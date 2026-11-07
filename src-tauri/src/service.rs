/*!
 * Windows service that performs privileged Driver Updater operations for the
 * medium-integrity UI. The pipe contract lives in `service_contract`.
 */

#[cfg(target_os = "windows")]
mod windows_service_impl {
    use std::ffi::OsString;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
    use windows_service::{define_windows_service, service_dispatcher};

    use crate::service_contract::SERVICE_NAME;

    define_windows_service!(ffi_service_main, service_main);

    pub fn run_service() {
        let _ = service_dispatcher::start(SERVICE_NAME, ffi_service_main);
    }

    fn service_main(_arguments: Vec<OsString>) {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_handler = Arc::clone(&stop);
        let Ok(status_handle) =
            service_control_handler::register(SERVICE_NAME, move |control| match control {
                ServiceControl::Stop | ServiceControl::Shutdown => {
                    stop_for_handler.store(true, Ordering::SeqCst);
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            })
        else {
            return;
        };

        let status = ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        };
        let _ = status_handle.set_service_status(status);

        crate::service::serve_pipe(Arc::clone(&stop));

        let _ = status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        });
    }
}

#[cfg(target_os = "windows")]
pub use windows_service_impl::run_service;

#[cfg(not(target_os = "windows"))]
pub fn run_service() {}

/// Serves requests until the service is stopped.
#[cfg(target_os = "windows")]
pub fn serve_pipe(stop: std::sync::Arc<std::sync::atomic::AtomicBool>) {
    use std::ffi::c_void;
    use std::sync::atomic::Ordering;
    use windows_sys::Win32::Foundation::{CloseHandle, LocalFree, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
    use windows_sys::Win32::Storage::FileSystem::{ReadFile, WriteFile};
    use windows_sys::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_MESSAGE,
        PIPE_TYPE_MESSAGE, PIPE_WAIT,
    };

    const PIPE_ACCESS_DUPLEX: u32 = 0x0000_0003;

    use crate::service_contract::{failure, validate_request, ServiceRequest, SERVICE_PIPE};

    let token = match crate::service_token::read() {
        Ok(token) => token,
        Err(_) => return,
    };

    // Authenticated users may open the pipe; the per-request token still
    // authenticates the caller and every operation is allowlisted.
    let sddl: Vec<u16> = "D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GRGW;;;AU)\0"
        .encode_utf16()
        .collect();
    let mut descriptor: *mut c_void = std::ptr::null_mut();
    let converted = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            std::ptr::null_mut(),
        )
    };
    if converted == 0 || descriptor.is_null() {
        return;
    }
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };

    while !stop.load(Ordering::SeqCst) {
        let name: Vec<u16> = SERVICE_PIPE
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let pipe = unsafe {
            CreateNamedPipeW(
                name.as_ptr(),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                1,
                64 * 1024,
                64 * 1024,
                0,
                &attributes,
            )
        };
        if pipe == INVALID_HANDLE_VALUE {
            break;
        }
        let connected = unsafe { ConnectNamedPipe(pipe, std::ptr::null_mut()) };
        if connected == 0 && unsafe { windows_sys::Win32::Foundation::GetLastError() } != 535 {
            unsafe {
                let _ = DisconnectNamedPipe(pipe);
                let _ = CloseHandle(pipe);
            }
            continue;
        }

        let mut buffer = vec![0u8; 64 * 1024];
        let mut read = 0u32;
        let ok = unsafe {
            ReadFile(
                pipe,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                &mut read,
                std::ptr::null_mut(),
            )
        };
        let response = if ok != 0 {
            match serde_json::from_slice::<ServiceRequest>(&buffer[..read as usize]) {
                Ok(request) => match validate_request(&request, &token) {
                    Ok(()) => handle_operation(request.operation),
                    Err(error) => failure(error),
                },
                Err(_) => failure("invalid service request"),
            }
        } else {
            failure("could not read service request")
        };
        let encoded = serde_json::to_vec(&response).unwrap_or_default();
        let mut written = 0u32;
        unsafe {
            let _ = WriteFile(
                pipe,
                encoded.as_ptr(),
                encoded.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            );
            let _ = DisconnectNamedPipe(pipe);
            let _ = CloseHandle(pipe);
        }
    }

    unsafe {
        let _ = LocalFree(descriptor);
    }
}

#[cfg(target_os = "windows")]
fn handle_operation(
    operation: crate::service_contract::ServiceOperation,
) -> crate::service_contract::ServiceResponse {
    use crate::service_contract::{failure, success, ServiceOperation};

    let result = match operation {
        ServiceOperation::Ping => Ok(serde_json::json!({ "service": "driver-updater" })),
        ServiceOperation::InstallDriverUpdate { update_id } => {
            crate::privileged::install_update(&update_id)
        }
        ServiceOperation::CreateRestorePoint { name } => {
            crate::privileged::create_restore_point(&name)
        }
        ServiceOperation::ListRestorePoints => crate::privileged::list_restore_points(),
        ServiceOperation::DeleteRestorePoint { id } => crate::privileged::delete_restore_point(&id),
        ServiceOperation::RestoreRestorePoint { id } => {
            crate::privileged::restore_restore_point(&id)
        }
        ServiceOperation::CreateDriverBackup { path } => {
            let root = crate::service_token::backup_root();
            match crate::service_contract::validate_backup_path(&path, &root) {
                Ok(()) => crate::privileged::create_driver_backup(&path),
                Err(error) => Err(error),
            }
        }
        ServiceOperation::RestoreDriverBackup { path, id } => {
            let root = crate::service_token::backup_root();
            match crate::service_contract::validate_backup_path(&path, &root) {
                Ok(()) => crate::privileged::restore_driver_backup(&path, &id),
                Err(error) => Err(error),
            }
        }
        ServiceOperation::DeleteDriverBackup { path, id } => {
            let root = crate::service_token::backup_root();
            match crate::service_contract::validate_backup_path(&path, &root) {
                Ok(()) => crate::privileged::delete_driver_backup(&path, &id),
                Err(error) => Err(error),
            }
        }
        ServiceOperation::InstallPackage {
            package_id,
            version,
        } => crate::privileged::install_package(&package_id, &version),
    };
    match result {
        Ok(value) => success(value),
        Err(error) => failure(error),
    }
}

/*!
 * Client side of the local Driver Updater service pipe.
 */

#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn call(
    operation: crate::service_contract::ServiceOperation,
) -> Result<serde_json::Value, String> {
    use windows_sys::Win32::Foundation::{
        CloseHandle, GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, ReadFile, WriteFile, OPEN_EXISTING,
    };

    use crate::service_contract::{
        ServiceRequest, ServiceResponse, SERVICE_PIPE, SERVICE_PROTOCOL_VERSION,
    };

    let token = crate::service_token::read()?;
    let request = ServiceRequest {
        version: SERVICE_PROTOCOL_VERSION,
        token,
        operation,
    };
    let encoded = serde_json::to_vec(&request).map_err(|error| error.to_string())?;
    let name: Vec<u16> = SERVICE_PIPE
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let pipe = unsafe {
        CreateFileW(
            name.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            0,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };
    if pipe == INVALID_HANDLE_VALUE {
        return Err("the Driver Updater service is not running".to_string());
    }
    let mut written = 0u32;
    let write_ok = unsafe {
        WriteFile(
            pipe,
            encoded.as_ptr(),
            encoded.len() as u32,
            &mut written,
            std::ptr::null_mut(),
        )
    };
    if write_ok == 0 {
        unsafe {
            let _ = CloseHandle(pipe);
        }
        return Err("could not send the service request".to_string());
    }
    let mut buffer = vec![0u8; crate::service_contract::MAX_REQUEST_BYTES];
    let mut read = 0u32;
    let read_ok = unsafe {
        ReadFile(
            pipe,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            &mut read,
            std::ptr::null_mut(),
        )
    };
    unsafe {
        let _ = CloseHandle(pipe);
    }
    if read_ok == 0 {
        return Err("could not read the service response".to_string());
    }
    let response: ServiceResponse =
        serde_json::from_slice(&buffer[..read as usize]).map_err(|error| error.to_string())?;
    if let Some(error) = response.error {
        return Err(error);
    }
    response
        .result
        .ok_or_else(|| "the service returned an empty response".to_string())
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn call(
    _operation: crate::service_contract::ServiceOperation,
) -> Result<serde_json::Value, String> {
    Err("the Driver Updater service is only available on Windows".to_string())
}

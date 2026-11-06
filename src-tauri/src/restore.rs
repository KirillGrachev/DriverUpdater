/*!
 * Real Windows System Restore integration.
 *
 * <p>Restore points are created and removed through `srclient.dll`; listing and
 * starting a restore use the native System Restore WMI provider directly. These
 * operations require an elevated process; the caller surfaces the resulting
 * error.</p>
 */

#[cfg(target_os = "windows")]
use std::mem::ManuallyDrop;

#[cfg(target_os = "windows")]
use windows::core::{s, w, BSTR, GUID};
#[cfg(target_os = "windows")]
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IDispatch,
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
#[cfg(target_os = "windows")]
use windows::Win32::System::Variant::{
    VariantChangeType, VariantToStringAlloc, VARIANT, VARIANT_NOVALUEPROP, VT_BSTR, VT_DATE, VT_I4,
    VT_I8, VT_UI4,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Wmi::{ISWbemLocator, ISWbemPropertySet, ISWbemServices};

use crate::models::RestorePoint;

#[cfg(target_os = "windows")]
const BEGIN_SYSTEM_CHANGE: u32 = 100;
#[cfg(target_os = "windows")]
const MODIFY_SETTINGS: u32 = 12;
#[cfg(target_os = "windows")]
const CLSID_SWBM_LOCATOR: GUID = GUID::from_u128(0x76a64158_cb41_11d1_8b02_00600806d9b6);
#[cfg(target_os = "windows")]
const WBEM_FLAG_RETURN_WBEM_COMPLETE: i32 = 0;

#[cfg(target_os = "windows")]
#[repr(C)]
struct RestorePointInfo {
    event_type: u32,
    restore_point_type: u32,
    sequence_number: i64,
    description: [u16; 256],
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct StateManagerStatus {
    status: u32,
    sequence_number: i64,
}

#[cfg(target_os = "windows")]
type SrSetRestorePoint = unsafe extern "system" fn(
    restore_point: *const RestorePointInfo,
    status: *mut StateManagerStatus,
) -> i32;
#[cfg(target_os = "windows")]
type SrRemoveRestorePointFn = unsafe extern "system" fn(sequence_number: u32) -> u32;

#[cfg(target_os = "windows")]
struct SrClient {
    set_restore_point: SrSetRestorePoint,
    remove_restore_point: SrRemoveRestorePointFn,
}

#[cfg(target_os = "windows")]
fn srclient() -> Result<&'static SrClient, String> {
    use std::sync::OnceLock;

    static CLIENT: OnceLock<Result<SrClient, String>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            // Resolve the restore APIs at runtime. The MSVC import library is
            // not linkable with every dependency combination, while the DLL is
            // present on every supported Windows release.
            let module = unsafe { LoadLibraryW(w!("srclient.dll")) }
                .map_err(|error| format!("srclient.dll could not be loaded: {error}"))?;
            let set_restore_point = unsafe { GetProcAddress(module, s!("SRSetRestorePointW")) }
                .ok_or_else(|| "SRSetRestorePointW is missing from srclient.dll".to_string())?;
            let remove_restore_point = unsafe {
                GetProcAddress(module, s!("SRRemoveRestorePoint"))
            }
            .ok_or_else(|| "SRRemoveRestorePoint is missing from srclient.dll".to_string())?;
            let set_restore_point: SrSetRestorePoint =
                unsafe { std::mem::transmute(set_restore_point) };
            let remove_restore_point: SrRemoveRestorePointFn =
                unsafe { std::mem::transmute(remove_restore_point) };
            Ok(SrClient {
                set_restore_point,
                remove_restore_point,
            })
        })
        .as_ref()
        .map_err(Clone::clone)
}

#[cfg(all(test, target_os = "windows"))]
#[test]
#[ignore = "requires the Windows client System Restore component"]
fn resolves_srclient_entry_points() {
    assert!(
        srclient().is_ok(),
        "srclient.dll must expose both restore entry points"
    );
}

#[cfg(target_os = "windows")]
fn now_iso() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0);
    unix_seconds_iso(seconds as i64)
}

#[cfg(target_os = "windows")]
fn unix_seconds_iso(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let time = seconds.rem_euclid(86_400);
    let (year, month, day) = crate::detection::days_to_civil(days.max(0) as u64);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3600,
        (time % 3600) / 60,
        time % 60
    )
}

/**
 * System Restore operations exposed to the repository.
 */
pub trait RestorePointProvider: Send + Sync {
    fn list(&self) -> Result<Vec<RestorePoint>, String>;
    fn create(&self, name: &str) -> Result<RestorePoint, String>;
    fn restore(&self, id: &str) -> Result<(), String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}

/**
 * Native Windows implementation.
 */
#[cfg(target_os = "windows")]
#[derive(Default)]
pub struct WindowsRestorePoints;

#[cfg(target_os = "windows")]
impl RestorePointProvider for WindowsRestorePoints {
    fn list(&self) -> Result<Vec<RestorePoint>, String> {
        with_wmi(|services| {
            let query = BSTR::from(
                "SELECT SequenceNumber, Description, CreationTime, RestorePointType FROM SystemRestore",
            );
            let language = BSTR::from("WQL");
            let points = unsafe { services.ExecQuery(&query, &language, 0, None::<&IDispatch>) }
                .map_err(|error| format!("WMI query failed: {error}"))?;
            let count =
                unsafe { points.Count() }.map_err(|error| format!("WMI count failed: {error}"))?;
            let mut result = Vec::new();
            for index in 0..count {
                let object = unsafe { points.ItemIndex(index) }
                    .map_err(|error| format!("WMI item failed: {error}"))?;
                let properties = unsafe { object.Properties_() }
                    .map_err(|error| format!("WMI properties failed: {error}"))?;
                let sequence = property_i64(&properties, "SequenceNumber")?;
                let description = property_string(&properties, "Description")?;
                let created_at = property_date(&properties, "CreationTime")?;
                let point_type = property_i64(&properties, "RestorePointType")? as i32;
                result.push(RestorePoint {
                    id: format!("rp-{sequence}"),
                    name: description,
                    kind: kind_for(point_type).to_string(),
                    created_at,
                    size_mb: 0,
                });
            }
            result.sort_by_key(|point| std::cmp::Reverse(sequence_number(&point.id).unwrap_or(0)));
            Ok(result)
        })
    }

    fn create(&self, name: &str) -> Result<RestorePoint, String> {
        let description = name.trim();
        if description.is_empty() || description.chars().count() > 200 {
            return Err("invalid restore point name".to_string());
        }
        let mut wide: Vec<u16> = description.encode_utf16().take(255).collect();
        wide.push(0);
        let mut spec = RestorePointInfo {
            event_type: BEGIN_SYSTEM_CHANGE,
            restore_point_type: MODIFY_SETTINGS,
            sequence_number: 0,
            description: [0; 256],
        };
        for (slot, unit) in spec.description.iter_mut().zip(wide) {
            *slot = unit;
        }
        let mut status = StateManagerStatus {
            status: 1,
            sequence_number: 0,
        };
        let client = srclient()?;
        let ok = unsafe { (client.set_restore_point)(&spec, &mut status) };
        if ok == 0 || status.status != 0 {
            return Err(format!(
                "System Restore failed with status {}",
                status.status
            ));
        }
        Ok(RestorePoint {
            id: format!("rp-{}", status.sequence_number),
            name: description.to_string(),
            kind: "manual".to_string(),
            created_at: now_iso(),
            size_mb: 0,
        })
    }

    fn restore(&self, id: &str) -> Result<(), String> {
        let sequence = sequence_number(id)?;
        with_wmi(|services| {
            let class = unsafe {
                services.Get(
                    &BSTR::from("SystemRestore"),
                    WBEM_FLAG_RETURN_WBEM_COMPLETE,
                    None::<&IDispatch>,
                )
            }
            .map_err(|error| format!("WMI class lookup failed: {error}"))?;
            let methods = unsafe { class.Methods_() }
                .map_err(|error| format!("WMI methods failed: {error}"))?;
            let method = unsafe { methods.Item(&BSTR::from("Restore"), 0) }
                .map_err(|error| format!("WMI Restore method failed: {error}"))?;
            let parameters = unsafe { method.InParameters() }
                .map_err(|error| format!("WMI parameters failed: {error}"))?;
            let properties = unsafe { parameters.Properties_() }
                .map_err(|error| format!("WMI parameters properties failed: {error}"))?;
            let value = bstr_variant(&sequence.to_string());
            unsafe {
                properties
                    .Item(&BSTR::from("SequenceNumber"), 0)
                    .and_then(|property| property.SetValue(&value))
            }
            .map_err(|error| format!("WMI parameter write failed: {error}"))?;
            let output = unsafe {
                services.ExecMethod(
                    &BSTR::from("SystemRestore"),
                    &BSTR::from("Restore"),
                    &parameters,
                    0,
                    None::<&IDispatch>,
                )
            }
            .map_err(|error| format!("System Restore call failed: {error}"))?;
            let output_properties = unsafe { output.Properties_() }
                .map_err(|error| format!("WMI result properties failed: {error}"))?;
            let return_value = property_i64(&output_properties, "ReturnValue")?;
            if return_value != 0 {
                return Err(format!("System Restore failed with {return_value}"));
            }
            Ok(())
        })
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        let sequence = u32::try_from(sequence_number(id)?)
            .map_err(|_| "invalid restore point id".to_string())?;
        let client = srclient()?;
        let result = unsafe { (client.remove_restore_point)(sequence) };
        if result != 0 {
            return Err(format!("SRRemoveRestorePoint failed with {result}"));
        }
        Ok(())
    }
}

/**
 * Non-Windows placeholder.
 */
#[cfg(not(target_os = "windows"))]
#[derive(Default)]
pub struct UnsupportedRestorePoints;

#[cfg(not(target_os = "windows"))]
impl RestorePointProvider for UnsupportedRestorePoints {
    fn list(&self) -> Result<Vec<RestorePoint>, String> {
        Err("System Restore is available only on Windows".to_string())
    }

    fn create(&self, _name: &str) -> Result<RestorePoint, String> {
        Err("System Restore is available only on Windows".to_string())
    }

    fn restore(&self, _id: &str) -> Result<(), String> {
        Err("System Restore is available only on Windows".to_string())
    }

    fn delete(&self, _id: &str) -> Result<(), String> {
        Err("System Restore is available only on Windows".to_string())
    }
}

/**
 * Returns the provider for the current target.
 */
pub fn default_restore_provider() -> Box<dyn RestorePointProvider> {
    #[cfg(target_os = "windows")]
    {
        Box::<WindowsRestorePoints>::default()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Box::<UnsupportedRestorePoints>::default()
    }
}

#[cfg(target_os = "windows")]
struct ComGuard(bool);

#[cfg(target_os = "windows")]
impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

#[cfg(target_os = "windows")]
fn with_wmi<T>(operation: impl FnOnce(&ISWbemServices) -> Result<T, String>) -> Result<T, String> {
    let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
        .ok()
        .is_ok();
    let _guard = ComGuard(initialized);
    let locator: ISWbemLocator = unsafe {
        CoCreateInstance(
            &CLSID_SWBM_LOCATOR,
            None::<&windows::core::IUnknown>,
            CLSCTX_INPROC_SERVER,
        )
    }
    .map_err(|error| format!("WMI locator failed: {error}"))?;
    let services = unsafe {
        locator.ConnectServer(
            &BSTR::from("."),
            &BSTR::from("root/default"),
            &BSTR::new(),
            &BSTR::new(),
            &BSTR::new(),
            &BSTR::new(),
            0,
            None::<&IDispatch>,
        )
    }
    .map_err(|error| format!("WMI connection failed: {error}"))?;
    operation(&services)
}

#[cfg(target_os = "windows")]
fn property_i64(properties: &ISWbemPropertySet, name: &str) -> Result<i64, String> {
    let value = unsafe { properties.Item(&BSTR::from(name), 0) }
        .and_then(|property| unsafe { property.Value() })
        .map_err(|error| format!("WMI {name} failed: {error}"))?;
    variant_i64(&value)
}

#[cfg(target_os = "windows")]
fn property_string(properties: &ISWbemPropertySet, name: &str) -> Result<String, String> {
    let value = unsafe { properties.Item(&BSTR::from(name), 0) }
        .and_then(|property| unsafe { property.Value() })
        .map_err(|error| format!("WMI {name} failed: {error}"))?;
    variant_string(&value)
}

#[cfg(target_os = "windows")]
fn property_date(properties: &ISWbemPropertySet, name: &str) -> Result<String, String> {
    let value = unsafe { properties.Item(&BSTR::from(name), 0) }
        .and_then(|property| unsafe { property.Value() })
        .map_err(|error| format!("WMI {name} failed: {error}"))?;
    if let Some(date) = variant_date(&value) {
        return Ok(ole_date_iso(date));
    }
    variant_string(&value)
}

#[cfg(target_os = "windows")]
fn variant_i64(value: &VARIANT) -> Result<i64, String> {
    let inner = unsafe { &*value.Anonymous.Anonymous };
    match inner.vt {
        VT_I4 => Ok(unsafe { inner.Anonymous.lVal } as i64),
        VT_I8 => Ok(unsafe { inner.Anonymous.llVal }),
        VT_UI4 => Ok(unsafe { inner.Anonymous.ulVal } as i64),
        VT_BSTR => unsafe { inner.Anonymous.bstrVal.to_string() }
            .parse()
            .map_err(|_| "WMI value is not numeric".to_string()),
        _ => variant_string(value)?
            .parse()
            .map_err(|_| "WMI value is not numeric".to_string()),
    }
}

#[cfg(target_os = "windows")]
fn variant_string(value: &VARIANT) -> Result<String, String> {
    let inner = unsafe { &*value.Anonymous.Anonymous };
    if inner.vt == VT_BSTR {
        return Ok(unsafe { inner.Anonymous.bstrVal.to_string() });
    }
    let mut converted = VARIANT::default();
    unsafe {
        VariantChangeType(&mut converted, value, VARIANT_NOVALUEPROP, VT_BSTR)
            .map_err(|error| format!("WMI value conversion failed: {error}"))?;
        let text = VariantToStringAlloc(&converted)
            .map_err(|error| format!("WMI value formatting failed: {error}"))?;
        let result = text.to_string().map_err(|error| error.to_string())?;
        CoTaskMemFree(Some(text.0 as _));
        Ok(result)
    }
}

#[cfg(target_os = "windows")]
fn variant_date(value: &VARIANT) -> Option<f64> {
    let inner = unsafe { &*value.Anonymous.Anonymous };
    if inner.vt == VT_DATE {
        Some(unsafe { inner.Anonymous.date })
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
fn ole_date_iso(date: f64) -> String {
    let seconds = ((date - 25_569.0) * 86_400.0).round() as i64;
    unix_seconds_iso(seconds)
}

#[cfg(target_os = "windows")]
fn bstr_variant(value: &str) -> VARIANT {
    let mut variant = VARIANT::default();
    unsafe {
        let inner = &mut *variant.Anonymous.Anonymous;
        inner.vt = VT_BSTR;
        inner.Anonymous.bstrVal = ManuallyDrop::new(BSTR::from(value));
    }
    variant
}

#[cfg(target_os = "windows")]
fn kind_for(restore_point_type: i32) -> &'static str {
    match restore_point_type {
        10 => "auto",
        12 => "manual",
        _ => "manual",
    }
}

#[cfg(target_os = "windows")]
fn sequence_number(id: &str) -> Result<i64, String> {
    let value = id
        .strip_prefix("rp-")
        .ok_or_else(|| "invalid restore point id".to_string())?;
    value
        .parse::<i64>()
        .map_err(|_| "invalid restore point id".to_string())
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn formats_the_current_time_as_utc_iso() {
        let value = now_iso();
        assert_eq!(value.len(), 20);
        assert!(value.ends_with('Z'));
        assert_eq!(&value[4..5], "-");
        assert_eq!(&value[10..11], "T");
    }

    #[test]
    fn parses_only_numeric_restore_point_ids() {
        assert_eq!(sequence_number("rp-42").expect("id"), 42);
        assert!(sequence_number("42").is_err());
        assert!(sequence_number("rp-nope").is_err());
    }

    #[test]
    #[ignore = "queries the live System Restore provider"]
    fn lists_native_restore_points() {
        let points = WindowsRestorePoints.list().expect("restore point list");
        println!("native restore points: {}", points.len());
        for point in points.iter().take(5) {
            println!(
                "{} {} {} {}",
                point.id, point.kind, point.created_at, point.name
            );
        }
    }

    #[test]
    fn converts_ole_dates() {
        assert_eq!(ole_date_iso(25_569.0), "1970-01-01T00:00:00Z");
    }
}

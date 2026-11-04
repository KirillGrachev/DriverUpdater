//! At-rest protection for locally persisted application data.
//!
//! <p>The desktop app stores settings, driver cache and restore-point metadata
//! under the current Windows user profile. On Windows, every file is sealed with
//! DPAPI in the current-user scope, so copying the file to another account or
//! machine does not reveal its contents. The optional entropy binds ciphertext
//! to this application identifier.</p>
//!
//! <p>This is not a defence against malware already running as the same user:
//! DPAPI keys are available to that user by design. It does protect backups,
//! copied profiles and other users on the same PC.</p>

#[cfg(target_os = "windows")]
use std::ffi::c_void;

/// File prefix that distinguishes sealed data from a legacy plaintext file.
pub const MAGIC: &[u8; 8] = b"DRVUPD1\0";

#[cfg(target_os = "windows")]
const ENTROPY: &[u8] = b"dev.driverupdater.desktop";

/// Returns whether a persisted payload is already sealed.
pub fn is_sealed(payload: &[u8]) -> bool {
    payload.starts_with(MAGIC)
}

/// Encrypts bytes for storage under the current Windows user.
///
/// On non-Windows development builds the payload is returned unchanged so the
/// browser preview and unit tests can still exercise the storage layer.
pub fn seal(data: &[u8]) -> Result<Vec<u8>, String> {
    #[cfg(target_os = "windows")]
    {
        let ciphertext = protect(data)?;
        let mut payload = Vec::with_capacity(MAGIC.len() + ciphertext.len());
        payload.extend_from_slice(MAGIC);
        payload.extend_from_slice(&ciphertext);
        Ok(payload)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(data.to_vec())
    }
}

/// Decrypts bytes previously produced by [`seal`].
///
/// A legacy plaintext payload is returned as-is so the caller can migrate it.
pub fn open(payload: &[u8]) -> Result<Vec<u8>, String> {
    if !is_sealed(payload) {
        return Ok(payload.to_vec());
    }

    #[cfg(target_os = "windows")]
    {
        unprotect(&payload[MAGIC.len()..])
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(payload[MAGIC.len()..].to_vec())
    }
}

#[cfg(target_os = "windows")]
fn protect(data: &[u8]) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let input = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    let entropy = CRYPT_INTEGER_BLOB {
        cbData: ENTROPY.len() as u32,
        pbData: ENTROPY.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let ok = unsafe {
        CryptProtectData(
            &input,
            std::ptr::null(),
            &entropy,
            std::ptr::null(),
            std::ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if ok == 0 {
        return Err("DPAPI encryption failed".to_string());
    }

    let bytes =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe {
        LocalFree(output.pbData as *mut c_void);
    }
    Ok(bytes)
}

#[cfg(target_os = "windows")]
fn unprotect(data: &[u8]) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let input = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    let entropy = CRYPT_INTEGER_BLOB {
        cbData: ENTROPY.len() as u32,
        pbData: ENTROPY.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let ok = unsafe {
        CryptUnprotectData(
            &input,
            std::ptr::null_mut(),
            &entropy,
            std::ptr::null(),
            std::ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if ok == 0 {
        return Err("DPAPI decryption failed".to_string());
    }

    let bytes =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe {
        LocalFree(output.pbData as *mut c_void);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seals_and_opens_round_trip() {
        let plaintext = br#"{"hostname":"test-pc","drivers":[]}"#;
        let sealed = seal(plaintext).expect("seal");
        #[cfg(target_os = "windows")]
        {
            assert!(is_sealed(&sealed));
            assert_ne!(sealed.as_slice(), plaintext.as_slice());
        }
        let opened = open(&sealed).expect("open");
        assert_eq!(opened, plaintext);
    }

    #[test]
    fn accepts_legacy_plaintext_for_migration() {
        let plaintext = b"legacy";
        assert_eq!(open(plaintext).expect("legacy"), plaintext);
    }
}

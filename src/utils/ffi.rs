use crate::common_ffi::{pchar_to_str, pchar_to_string, PChar};

#[no_mangle]
pub unsafe extern "C" fn utils_compare_versions(file1: PChar, file2: PChar, out: *mut i8) -> bool {
    boolclosure! {{
        *out = super::version::compare_versions(pchar_to_str(file1)?, pchar_to_str(file2)?)? as i8;
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn utils_extract_file_version(file_name: PChar, out: *mut PChar) -> bool {
    boolclosure! {{
        use std::ffi::CString;
        let v = super::version::extract_version(pchar_to_str(file_name)?)?;
        *out = CString::new(v).unwrap().into_raw();
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn utils_log_info(text: PChar) {
    log::info!("{}", pchar_to_string(text).unwrap_or_default());
}

#[no_mangle]
pub unsafe extern "C" fn utils_log_warn(text: PChar) {
    log::warn!("{}", pchar_to_string(text).unwrap_or_default());
}

#[no_mangle]
pub unsafe extern "C" fn utils_log_error(text: PChar) {
    log::error!("{}", pchar_to_string(text).unwrap_or_default());
}

#[no_mangle]
pub unsafe extern "C" fn utils_log_debug(text: PChar) {
    log::debug!("{}", pchar_to_string(text).unwrap_or_default());
}
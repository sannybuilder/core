use crate::common_ffi::{pchar_to_str, PChar};

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
        *out = CString::into_raw(CString::new(v).unwrap());
        Some(())
    }}
}

use crate::common_ffi::*;

#[no_mangle]
pub extern "C" fn gxt_new() -> *mut GxtParser {
    ptr_new(GxtParser::new())
}

#[no_mangle]
pub unsafe extern "C" fn gxt_free(p: *mut GxtParser) {
    ptr_free(p)
}

#[no_mangle]
pub unsafe extern "C" fn gxt_load_file(p: *mut GxtParser, path: PChar) -> bool {
    boolclosure!({
        let path = pchar_to_str(path)?;
        (*p).load_file(path)?;
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn gxt_get(p: *mut GxtParser, key: PChar, out: *mut PChar) -> bool {
    boolclosure!({
        let key = pchar_to_str(key)?;
        let value = (*p).get(key)?;
        *out = CString::new(value).unwrap().into_raw();
        Some(())
    })
}

use libc::c_char;
use std::ffi::{CStr, CString};

pub type PChar = *const c_char;

pub fn pchar_to_string<'a>(s: PChar) -> String {
    unsafe { String::from(CStr::from_ptr(s).to_str().unwrap()) }
}

pub fn pchar_to_str<'a>(s: PChar) -> &'a str {
    unsafe { CStr::from_ptr(s).to_str().unwrap() }
}

#[allow(dead_code)]
pub fn string_to_pchar(s: String) -> PChar {
    CString::new(s).unwrap().into_raw()
}

#[allow(dead_code)]
pub fn str_to_pchar(s: &str) -> PChar {
    CString::new(s).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn str_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    CString::from_raw(s);
}

pub unsafe fn ptr_free<T>(ptr: *mut T) {
    if ptr.is_null() {
        return;
    }
    Box::from_raw(ptr);
}

pub fn ptr_new<T>(o: T) -> *const T {
    Box::into_raw(Box::new(o))
}

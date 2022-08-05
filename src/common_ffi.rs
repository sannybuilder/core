use libc::c_char;
pub type PChar = *const c_char;

pub fn pchar_to_string<'a>(s: PChar) -> Option<String> {
    if s.is_null() {
        None
    } else {
        unsafe { Some(String::from(std::ffi::CStr::from_ptr(s).to_string_lossy())) }
    }
}

pub fn pchar_to_str<'a>(s: PChar) -> Option<&'a str> {
    if s.is_null() {
        None
    } else {
        unsafe { Some(std::ffi::CStr::from_ptr(s).to_str().ok()?) }
    }
}

#[no_mangle]
pub unsafe extern "C" fn str_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    let _ = std::ffi::CString::from_raw(s);
}

pub unsafe fn ptr_free<T>(ptr: *mut T) {
    if ptr.is_null() {
        return;
    }
    Box::from_raw(ptr);
}

pub fn ptr_new<T>(o: T) -> *mut T {
    Box::into_raw(Box::new(o))
}

#[allow(unused_macros)]
macro_rules! pchar {
    ($name:expr) => {
        std::ffi::CString::new($name).unwrap().as_ptr()
    };
}

macro_rules! boolclosure {
    ($b:block) => {
        || -> Option<()> { $b }().is_some()
    };
}

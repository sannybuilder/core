use crate::common_ffi::{pchar_to_string, PChar};

#[no_mangle]
pub unsafe extern "C" fn v4_try_transform(input: PChar, out: *mut PChar) -> bool {
    boolclosure! {{
        let input = pchar_to_string(input)?;
        let result = super::transform(&input)?;
        *out = std::ffi::CString::new(result).unwrap().into_raw();
        Some(())
    }}
}

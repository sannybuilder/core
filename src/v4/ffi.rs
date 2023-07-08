use crate::common_ffi::{pchar_to_string, PChar};
use crate::legacy_ini::OpcodeTable;
use crate::namespaces::namespaces::Namespaces;

#[no_mangle]
pub unsafe extern "C" fn v4_try_transform(
    input: PChar,
    ns: *const Namespaces,
    legacy_ini: *const OpcodeTable,
    out: *mut PChar,
) -> bool {
    boolclosure! {{
        let input = pchar_to_string(input)?;
        let result = super::transform(&input, ns.as_ref()?, legacy_ini.as_ref()?)?;
        *out = std::ffi::CString::new(result).unwrap().into_raw();
        Some(())
    }}
}

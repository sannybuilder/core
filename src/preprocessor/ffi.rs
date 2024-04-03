use std::ffi::CString;

use super::{PreProcessorBuilder, Preprocessor};
use crate::common_ffi::*;

#[no_mangle]
pub extern "C" fn preprocessor_new(include: PChar, compiler_ini: PChar) -> *mut Preprocessor {
    let mut p = PreProcessorBuilder::new();

    if let Some(include_path) = pchar_to_str(include) {
        p.implicit_includes(vec![include_path.into()]);
    }
    if let Some(compiler_ini_path) = pchar_to_str(compiler_ini) {
        p.reserved_words(compiler_ini_path.into());
    }
    ptr_new(p.build())
}

#[no_mangle]
pub unsafe extern "C" fn preprocessor_free(p: *mut Preprocessor) {
    ptr_free(p)
}

#[no_mangle]
pub unsafe extern "C" fn preprocessor_parse_file(p: *mut Preprocessor, file: PChar) -> bool {
    boolclosure! {{
        let p = p.as_mut()?;
        let file = pchar_to_str(file)?;
        p.parse_file(file.into()).ok()
    }}
}

// #[no_mangle]
// pub unsafe extern "C" fn preprocessor_get_line(
//     p: *mut Preprocessor,
//     line_index: u32,
//     out_line: *mut PChar,
// ) -> bool {
//     boolclosure! {{
//         let p = p.as_mut()?;
//         *out_line = p.get_line(line_index as usize)?.as_ptr();
//         Some(())
//     }}
// }

// #[no_mangle]
// pub unsafe extern "C" fn preprocessor_get_line_count(
//     p: *mut Preprocessor,
//     out_count: *mut u32,
// ) -> bool {
//     boolclosure! {{
//         let p = p.as_mut()?;
//         *out_count = p.get_line_count() as u32;
//         Some(())
//     }}
// }

// #[no_mangle]
// pub unsafe extern "C" fn preprocessor_translate_line(
//     p: *mut Preprocessor,
//     line_index: u32,
//     out_index: *mut u32,
//     out_filename: *mut PChar,
// ) -> bool {
//     boolclosure! {{
//         let p = p.as_mut()?;
//         let (index, filename) = p.translate_line(line_index as usize)?;
//         *out_index = index as u32;
//         *out_filename = CString::new(filename.to_str()?).ok()?.into_raw();
//         Some(())
//     }}
// }

#[no_mangle]
pub unsafe extern "C" fn preprocessor_get_number_of_functions_this_scope(
    p: *mut Preprocessor,
    line_index: u32,
    out_count: *mut u32,
) -> bool {
    boolclosure! {{
        let p = p.as_mut()?;
        *out_count = p.get_number_of_functions_this_scope(line_index as usize) as u32;
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn preprocessor_get_function(
    p: *mut Preprocessor,
    line_index: u32,
    function_index: u32,
    out_signature: *mut PChar,
) -> bool {
    boolclosure! {{
        let p = p.as_mut()?;
        let f = p.get_function(line_index as usize, function_index as usize)?;
        *out_signature = CString::new(f.signature.clone()).ok()?.into_raw();
        Some(())
    }}
}

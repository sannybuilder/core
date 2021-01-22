use crate::common_ffi::*;
use crate::dictionary::ffi::*;
use std::ffi::CString;

use super::config::ConfigBuilder;

pub type ListNumByStr = Dict<CString, i32>;

#[no_mangle]
pub extern "C" fn list_num_by_str_new(
    duplicates: u8,
    hex_keys: bool,
    comments: PChar,
    delimiters: PChar,
    strip_whitespace: bool,
) -> *mut ListNumByStr {
    let mut builder = ConfigBuilder::new();

    builder
        .set_duplicates(duplicates.into())
        .set_case_format(CaseFormat::NoFormat)
        .set_strip_whitespace(strip_whitespace)
        .set_hex_keys(hex_keys);

    if let Some(comments) = pchar_to_string(comments) {
        builder.set_comments(comments);
    }
    if let Some(delimiters) = pchar_to_string(delimiters) {
        builder.set_delimiters(delimiters);
    }
    ptr_new(Dict::new(builder.build()))
}

#[no_mangle]
pub unsafe extern "C" fn list_num_by_str_load_file(
    list: *mut ListNumByStr,
    file_name: PChar,
) -> bool {
    boolclosure! {{
        list.as_mut()?.load_file(pchar_to_str(file_name)?)
    }}
}

#[no_mangle]
pub unsafe extern "C" fn list_num_by_str_get_entry(
    list: *mut ListNumByStr,
    index: usize,
    out_key: *mut PChar,
    out_value: *mut i32,
) -> bool {
    boolclosure! {{
        let (key, value) = list.as_mut()?.map.iter().nth(index)?;
        *out_key = key.as_ptr();
        *out_value = *value;
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn list_num_by_str_get_count(list: *mut ListNumByStr) -> usize {
    if let Some(ptr) = list.as_mut() {
        return ptr.map.len();
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn list_num_by_str_free(ptr: *mut ListNumByStr) {
    ptr_free(ptr);
}

#[test]
fn test_list_num_by_str_get_entry() {
    unsafe {
        let f = list_num_by_str_new(
            Duplicates::Replace.into(),
            true,
            pchar!(";"),
            pchar!(",="),
            true,
        );
        assert!(f.as_mut().is_some());
        assert_eq!(f.as_ref().unwrap().config.case_format, CaseFormat::NoFormat);

        let loaded = list_num_by_str_load_file(f, pchar!("src/dictionary/test/keywords-hex.txt"));
        assert!(loaded);

        let mut key = pchar!("");
        let mut value = 0;
        let res = list_num_by_str_get_entry(f, 1, &mut key, &mut value);
        assert!(res);
    }
}

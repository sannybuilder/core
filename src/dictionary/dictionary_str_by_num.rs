use crate::common_ffi::*;
use crate::dictionary::ffi::*;
use std::ffi::CString;

use super::config::ConfigBuilder;

pub type DictStrByNum = Dict<i32, CString>;

#[no_mangle]
pub extern "C" fn dictionary_str_by_num_new(
    duplicates: u8,
    hex_keys: bool,
    case_format: u8,
    comments: PChar,
    delimiters: PChar,
    strip_whitespace: bool,
) -> *mut DictStrByNum {
    let mut builder = ConfigBuilder::new();

    builder
        .set_duplicates(duplicates.into())
        .set_case_format(case_format.into())
        .set_strip_whitespace(strip_whitespace)
        .set_hex_keys(hex_keys);

    if let Some(comments) = pchar_to_string(comments) {
        builder.set_comments(comments);
    }
    if let Some(delimiters) = pchar_to_string(delimiters) {
        builder.set_delimiters(delimiters);
    }

    log::debug!("New instance with config {:?}", builder);
    ptr_new(Dict::new(builder.build()))
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_load_file(
    dict: *mut DictStrByNum,
    file_name: PChar,
) -> bool {
    boolclosure! {{
        let file_name = pchar_to_str(file_name)?;
        log::debug!("Loading file {}", file_name);
        let d = dict.as_mut()?;
        d.load_file(file_name);
        log::debug!("Loaded {} entries", d.map.len());
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_add(
    dict: *mut DictStrByNum,
    key: i32,
    value: PChar,
) -> bool {
    boolclosure! {{
        let d = dict.as_mut()?;
        let value = apply_format(pchar_to_str(value)?, &d.config.case_format)?;
        d.add(key, value);
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_find(
    dict: *mut DictStrByNum,
    key: i32,
    out: *mut PChar,
) -> bool {
    boolclosure! {{
       *out = dict.as_mut()?.map.get(&key)?.as_ptr();
       Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_get_entry(
    dict: *mut DictStrByNum,
    index: usize,
    out_key: *mut i32,
    out_value: *mut PChar,
) -> bool {
    boolclosure! {{
        let (key, value) = dict.as_mut()?.map.iter().nth(index)?;
        *out_key = *key;
        *out_value = value.as_ptr();
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_filter_by_name(
    ns: *mut DictStrByNum,
    needle: PChar,
    dict: *mut crate::dictionary::dictionary_str_by_num::DictStrByNum,
) -> bool {
    boolclosure! {{
        let needle = pchar_to_str(needle)?.to_ascii_lowercase();
        let d = ns.as_mut()?;

        for (key, value) in d.map.iter() {
            let name = value.to_str().ok()?.to_ascii_lowercase();
            if name.starts_with(&needle) {
                dict.as_mut()?.add(*key, value.clone());
            }

        }
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_get_count(dict: *mut DictStrByNum) -> usize {
    if let Some(ptr) = dict.as_mut() {
        return ptr.map.len();
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_free(ptr: *mut DictStrByNum) {
    ptr_free(ptr);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        unsafe {
            let f = dictionary_str_by_num_new(
                Duplicates::Replace.into(),
                false,
                CaseFormat::NoFormat.into(),
                pchar!(";"),
                pchar!(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded =
                dictionary_str_by_num_load_file(f, pchar!("src/dictionary/test/keywords.txt"));
            assert!(loaded);
            assert_eq!(dictionary_str_by_num_get_count(f), 2);
        }
    }

    #[test]
    fn test_dictionary_str_by_num_find_uppercase() {
        unsafe {
            let f = dictionary_str_by_num_new(
                Duplicates::Replace.into(),
                true,
                CaseFormat::UpperCase.into(),
                pchar!(";"),
                pchar!(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded =
                dictionary_str_by_num_load_file(f, pchar!("src/dictionary/test/keywords-hex.txt"));
            assert!(loaded);
            let mut s = pchar!("");
            assert!(dictionary_str_by_num_find(f, 2, &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "JUMP");

            assert!(!dictionary_str_by_num_find(f, 0, &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "JUMP");
        }
    }

    #[test]
    fn test_dictionary_str_by_num_find_lowercase() {
        unsafe {
            let f = dictionary_str_by_num_new(
                Duplicates::Replace.into(),
                true,
                CaseFormat::LowerCase.into(),
                pchar!(";"),
                pchar!(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded =
                dictionary_str_by_num_load_file(f, pchar!("src/dictionary/test/keywords-hex.txt"));
            assert!(loaded);

            let mut s = pchar!("");
            assert!(dictionary_str_by_num_find(f, 2, &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "jump");

            assert!(!dictionary_str_by_num_find(f, 0, &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_num_duplicates_ignore() {
        unsafe {
            let f = dictionary_str_by_num_new(
                Duplicates::Ignore.into(),
                true,
                CaseFormat::NoFormat.into(),
                pchar!(";"),
                pchar!(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded =
                dictionary_str_by_num_load_file(f, pchar!("src/dictionary/test/keywords-dups.txt"));
            assert!(loaded);
            let mut s = pchar!("");
            assert!(dictionary_str_by_num_find(f, 1, &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "wait");

            assert!(dictionary_str_by_num_find(f, 2, &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_num_duplicates_replace() {
        unsafe {
            let f = dictionary_str_by_num_new(
                Duplicates::Replace.into(),
                true,
                CaseFormat::NoFormat.into(),
                pchar!(";"),
                pchar!(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded =
                dictionary_str_by_num_load_file(f, pchar!("src/dictionary/test/keywords-dups.txt"));
            assert!(loaded);

            let mut s = pchar!("");
            assert!(dictionary_str_by_num_find(f, 1, &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "jump");

            assert!(dictionary_str_by_num_find(f, 2, &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "jump");
        }
    }
}

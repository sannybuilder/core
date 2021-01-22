use crate::common_ffi::*;
use crate::dictionary::ffi::*;
use std::ffi::CString;

use super::{dictionary_num_by_str::DictNumByStr, dictionary_str_by_num::DictStrByNum};

struct DictTwoWay {
    num: DictNumByStr,
    str: DictStrByNum,
}

#[no_mangle]
pub extern "C" fn dictionary_two_way_new(
    duplicates: u8,
    hex_keys: bool,
    case_format: u8,
    comments: PChar,
    delimiters: PChar,
    trim: bool,
) -> *mut DictTwoWay {
    let num = DictNumByStr::new(
        duplicates.into(),
        case_format.into(),
        pchar_to_string(comments).unwrap_or(String::new()),
        pchar_to_string(delimiters).unwrap_or(String::new()),
    );

    ptr_new(Dict::new(
        duplicates.into(),
        case_format.into(),
        pchar_to_string(comments).unwrap_or(String::new()),
        pchar_to_string(delimiters).unwrap_or(String::new()),
        trim,
        false,
    ))
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_load_file(
    dict: *mut DictStrByStr,
    file_name: PChar,
) -> bool {
    boolclosure! {{
        dict.as_mut()?.load_file(pchar_to_str(file_name)?)
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_add(
    dict: *mut DictStrByStr,
    key: PChar,
    value: PChar,
) -> bool {
    boolclosure! {{
       dict.as_mut()?.add_raw(pchar_to_str(key)?, pchar_to_str(value)?);
       Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_find(
    dict: *mut DictStrByStr,
    key: PChar,
    out: *mut PChar,
) -> bool {
    boolclosure! {{
        let d = dict.as_mut()?;
        let key = apply_format(pchar_to_str(key)?, &d.case_format)?;
        *out = d.map.get(&key)?.as_ptr();
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_get_entry(
    dict: *mut DictStrByStr,
    index: usize,
    out_key: *mut PChar,
    out_value: *mut PChar,
) -> bool {
    boolclosure! {{
      let (key, value) = dict.as_mut()?.map.iter().nth(index)?;
      *out_key = key.as_ptr();
      *out_value = value.as_ptr();
      Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_get_count(dict: *mut DictStrByStr) -> usize {
    if let Some(ptr) = dict.as_mut() {
        return ptr.map.len();
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_free(ptr: *mut DictStrByStr) {
    ptr_free(ptr);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_str_by_str_get_count() {
        unsafe {
            let f = dictionary_str_by_str_new(
                Duplicates::Replace.into(),
                CaseFormat::NoFormat.into(),
                pchar!(";"),
                pchar!(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded =
                dictionary_str_by_str_load_file(f, pchar!("src/dictionary/test/keywords.txt"));
            assert!(loaded);

            assert_eq!(dictionary_str_by_str_get_count(f), 2);
        }
    }

    #[test]
    fn test_dictionary_str_by_str_find_uppercase() {
        unsafe {
            let f = dictionary_str_by_str_new(
                Duplicates::Replace.into(),
                CaseFormat::UpperCase.into(),
                pchar!(";"),
                pchar!(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded =
                dictionary_str_by_str_load_file(f, pchar!("src/dictionary/test/keywords-hex.txt"));
            assert!(loaded);
            let mut s = pchar!("");
            assert!(dictionary_str_by_str_find(f, pchar!("0002"), &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "JUMP");

            assert!(!dictionary_str_by_str_find(f, pchar!("0000"), &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "JUMP");
        }
    }

    #[test]
    fn test_dictionary_str_by_str_find_lowercase() {
        unsafe {
            let f = dictionary_str_by_str_new(
                Duplicates::Replace.into(),
                CaseFormat::LowerCase.into(),
                pchar!(";"),
                pchar!(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded =
                dictionary_str_by_str_load_file(f, pchar!("src/dictionary/test/keywords-hex.txt"));
            assert!(loaded);

            let mut s = pchar!("");
            assert!(dictionary_str_by_str_find(f, pchar!("0002"), &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "jump");

            assert!(!dictionary_str_by_str_find(f, pchar!("0000"), &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_str_duplicates_ignore() {
        unsafe {
            let f = dictionary_str_by_str_new(
                Duplicates::Ignore.into(),
                CaseFormat::LowerCase.into(),
                pchar!(";"),
                pchar!(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded =
                dictionary_str_by_str_load_file(f, pchar!("src/dictionary/test/keywords-dups.txt"));
            assert!(loaded);

            let mut s = pchar!("");
            assert!(dictionary_str_by_str_find(f, pchar!("0001"), &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "wait");

            assert!(dictionary_str_by_str_find(f, pchar!("0002"), &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_str_duplicates_replace() {
        unsafe {
            let f = dictionary_str_by_str_new(
                Duplicates::Replace.into(),
                CaseFormat::NoFormat.into(),
                pchar!(";"),
                pchar!(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded =
                dictionary_str_by_str_load_file(f, pchar!("src/dictionary/test/keywords-dups.txt"));
            assert!(loaded);

            let mut s = pchar!("");
            assert!(dictionary_str_by_str_find(f, pchar!("0001"), &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "jump");

            assert!(dictionary_str_by_str_find(f, pchar!("0002"), &mut s));
            assert_eq!(pchar_to_str(s).unwrap(), "jump");
        }
    }
}

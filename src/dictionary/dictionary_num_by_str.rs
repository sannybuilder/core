use crate::common_ffi::*;
use crate::dictionary::ffi::*;
use std::ffi::CString;

pub type DictNumByStr = Dict<CString, i32>;

#[no_mangle]
pub extern "C" fn dictionary_num_by_str_new(
    duplicates: u8,
    hex_keys: bool,
    comments: PChar,
    delimiters: PChar,
    trim: bool,
) -> *mut DictNumByStr {
    ptr_new(Dict::new(
        duplicates.into(),
        CaseFormat::NoFormat,
        pchar_to_string(comments).unwrap_or(String::new()),
        pchar_to_string(delimiters).unwrap_or(String::new()),
        trim,
        hex_keys,
    ))
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_load_file(
    dict: *mut DictNumByStr,
    file_name: PChar,
) -> bool {
    boolclosure! {{
        dict.as_mut()?.load_file(pchar_to_str(file_name)?)
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_add(
    dict: *mut DictNumByStr,
    key: PChar,
    value: i32,
) -> bool {
    boolclosure! {{
        let d = dict.as_mut()?;
        let key = apply_format(pchar_to_str(key)?, &d.case_format)?;
        d.add(key, value);
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_find(
    dict: *mut DictNumByStr,
    key: PChar,
    out: *mut i32,
) -> bool {
    boolclosure! {{
        let key = CString::new(pchar_to_str(key)?).ok()?;
        *out = *dict.as_mut()?.map.get(&key)?;
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_get_entry(
    dict: *mut DictNumByStr,
    index: usize,
    out_key: *mut PChar,
    out_value: *mut i32,
) -> bool {
    boolclosure! {{
        let (key, value) = dict.as_mut()?.map.iter().nth(index)?;
        *out_key = key.as_ptr();
        *out_value = *value;
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_get_count(dict: *mut DictNumByStr) -> usize {
    if let Some(ptr) = dict.as_mut() {
        return ptr.map.len();
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_free(ptr: *mut DictNumByStr) {
    ptr_free(ptr);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_num_by_str_get_count() {
        unsafe {
            let f = dictionary_num_by_str_new(
                Duplicates::Replace.into(),
                false,
                pchar!(";"),
                pchar!(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded =
                dictionary_num_by_str_load_file(f, pchar!("src/dictionary/test/keywords.txt"));
            assert!(loaded);
            assert_eq!(dictionary_num_by_str_get_count(f), 2);
        }
    }

    #[test]
    fn test_dictionary_num_by_str_find() {
        unsafe {
            let f = dictionary_num_by_str_new(
                Duplicates::Replace.into(),
                true,
                pchar!(";"),
                pchar!(",="),
                true,
            );
            assert!(f.as_mut().is_some());

            let loaded =
                dictionary_num_by_str_load_file(f, pchar!("src/dictionary/test/keywords-hex.txt"));
            assert!(loaded);

            assert_eq!(dictionary_num_by_str_get_count(f), 23);
            let mut i = 0;
            assert!(dictionary_num_by_str_find(f, pchar!("wait"), &mut i));
            assert_eq!(i, 1);
            i = -1;
            assert!(!dictionary_num_by_str_find(f, pchar!(""), &mut i));
            assert_eq!(i, -1);
        }
    }

    #[test]
    fn test_dictionary_num_by_str_duplicates_ignore() {
        unsafe {
            let f = dictionary_num_by_str_new(
                Duplicates::Ignore.into(),
                true,
                pchar!(";"),
                pchar!(",="),
                true,
            );
            assert!(f.as_mut().is_some());

            let loaded =
                dictionary_num_by_str_load_file(f, pchar!("src/dictionary/test/keywords-dups.txt"));
            assert!(loaded);

            assert_eq!(dictionary_num_by_str_get_count(f), 2);

            let mut i = 0;
            assert!(dictionary_num_by_str_find(f, pchar!("wait"), &mut i));
            assert_eq!(i, 1);
            assert!(dictionary_num_by_str_find(f, pchar!("jump"), &mut i));
            assert_eq!(i, 1);
        }
    }

    #[test]
    fn test_dictionary_num_by_str_duplicates_replace() {
        unsafe {
            let f = dictionary_num_by_str_new(
                Duplicates::Replace.into(),
                true,
                pchar!(";"),
                pchar!(",="),
                true,
            );
            assert!(f.as_mut().is_some());

            let loaded =
                dictionary_num_by_str_load_file(f, pchar!("src/dictionary/test/keywords-dups.txt"));
            assert!(loaded);

            assert_eq!(dictionary_num_by_str_get_count(f), 2);

            let mut i = 0;
            assert!(dictionary_num_by_str_find(f, pchar!("wait"), &mut i));
            assert_eq!(i, 1);
            assert!(dictionary_num_by_str_find(f, pchar!("jump"), &mut i));
            assert_eq!(i, 2);
        }
    }
}

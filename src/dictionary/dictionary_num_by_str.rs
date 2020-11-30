use crate::common_ffi::*;
use crate::dictionary::ffi::*;

pub type DictNumByStr = Dict<String, i32>;

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
        CaseFormat::LowerCase,
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
        let key = apply_format_s(pchar_to_str(key)?, &d.case_format);
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
        let d = dict.as_mut()?;
        let key = apply_format_s(pchar_to_str(key)?, &d.case_format);
        *out = *d.map.get(&key)?;
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_free(ptr: *mut DictNumByStr) {
    ptr_free(ptr);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

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

            let mut i = 0;
            assert!(dictionary_num_by_str_find(f, pchar!("Wait"), &mut i));
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
            assert!(f.as_ref().is_some());

            assert_eq!(f.as_ref().unwrap().case_format, CaseFormat::LowerCase);

            let loaded =
                dictionary_num_by_str_load_file(f, pchar!("src/dictionary/test/keywords-dups.txt"));
            assert!(loaded);

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
            assert!(f.as_ref().is_some());

            let loaded =
                dictionary_num_by_str_load_file(f, pchar!("src/dictionary/test/keywords-dups.txt"));
            assert!(loaded);

            let mut i = 0;
            assert!(dictionary_num_by_str_find(f, pchar!("wait"), &mut i));
            assert_eq!(i, 1);
            assert!(dictionary_num_by_str_find(f, pchar!("jump"), &mut i));
            assert_eq!(i, 2);
        }
    }
}

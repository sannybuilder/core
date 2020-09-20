use crate::common_ffi::*;
use crate::dictionary::ffi::*;
use std::ffi::CString;

type DictNumByStr = Dict<CString, i32>;

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
        pchar_to_string(comments),
        pchar_to_string(delimiters),
        trim,
        hex_keys,
    ))
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_load_file(
    dict: *mut DictNumByStr,
    file_name: PChar,
) -> bool {
    if let Some(ptr) = dict.as_mut() {
        if let Ok(_) = ptr.load_file(pchar_to_str(file_name)) {
            return true;
        }
    }
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_find(
    dict: *mut DictNumByStr,
    name: PChar,
    out: *mut i32,
) -> bool {
    if let Some(ptr) = dict.as_mut() {
        if let Ok(name) = CString::new(pchar_to_str(name)) {
            if let Some(val) = ptr.map.get(&name) {
                *out = *val;
                return true;
            }
        }
    }
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_get_entry(
    dict: *mut DictNumByStr,
    index: usize,
    out_key: *mut PChar,
    out_value: *mut i32,
) -> bool {
    if let Some(ptr) = dict.as_mut() {
        if let Some((key, &value)) = ptr.map.iter().nth(index) {
            *out_value = value;
            *out_key = key.as_ptr();
            return true;
        }
    }
    return false;
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
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded = dictionary_num_by_str_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords.txt"),
            );
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
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );
            assert!(f.as_mut().is_some());

            let loaded = dictionary_num_by_str_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords-hex.txt"),
            );
            assert!(loaded);

            assert_eq!(dictionary_num_by_str_get_count(f), 23);
            let mut i = 0;
            assert!(dictionary_num_by_str_find(f, str_to_pchar("wait"), &mut i));
            assert_eq!(i, 1);
            i = -1;
            assert!(!dictionary_num_by_str_find(f, str_to_pchar(""), &mut i));
            assert_eq!(i, -1);
        }
    }

    #[test]
    fn test_dictionary_num_by_str_duplicates_ignore() {
        unsafe {
            let f = dictionary_num_by_str_new(
                Duplicates::Ignore.into(),
                true,
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );
            assert!(f.as_mut().is_some());

            let loaded = dictionary_num_by_str_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords-dups.txt"),
            );
            assert!(loaded);

            assert_eq!(dictionary_num_by_str_get_count(f), 2);

            let mut i = 0;
            assert!(dictionary_num_by_str_find(f, str_to_pchar("wait"), &mut i));
            assert_eq!(i, 1);
            assert!(dictionary_num_by_str_find(f, str_to_pchar("jump"), &mut i));
            assert_eq!(i, 1);
        }
    }

    #[test]
    fn test_dictionary_num_by_str_duplicates_replace() {
        unsafe {
            let f = dictionary_num_by_str_new(
                Duplicates::Replace.into(),
                true,
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );
            assert!(f.as_mut().is_some());

            let loaded = dictionary_num_by_str_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords-dups.txt"),
            );
            assert!(loaded);

            assert_eq!(dictionary_num_by_str_get_count(f), 2);

            let mut i = 0;
            assert!(dictionary_num_by_str_find(f, str_to_pchar("wait"), &mut i));
            assert_eq!(i, 1);
            assert!(dictionary_num_by_str_find(f, str_to_pchar("jump"), &mut i));
            assert_eq!(i, 2);
        }
    }
}

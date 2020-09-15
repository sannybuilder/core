use crate::common_ffi::*;
use crate::dictionary::ffi::*;
use std::ffi::CString;

type DictNumByStr = Dict<CString, i32>;
type PDict = *const DictNumByStr;
type PDictMut = *mut DictNumByStr;

#[no_mangle]
pub extern "C" fn dictionary_num_by_str_new(
    duplicates: u8,
    hex_keys: bool,
    comments: PChar,
    delimiters: PChar,
    trim: bool,
) -> PDict {
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
    dict: &mut DictNumByStr,
    file_name: PChar,
) -> bool {
    if let Ok(_) = dict.load_file(pchar_to_str(file_name)) {
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_find(
    dict: &DictNumByStr,
    name: PChar,
    out: *mut i32,
) -> bool {
    if let Ok(name) = CString::new(pchar_to_str(name)) {
        if let Some(val) = dict.map.get(&name) {
            *out = *val;
            return true;
        }
    }
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_get_entry(
    dict: &DictNumByStr,
    index: usize,
    out_key: *mut PChar,
    out_value: *mut i32,
) -> bool {
    if let Some((key, &value)) = dict.map.iter().nth(index) {
        *out_value = value;
        *out_key = key.as_ptr();
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_get_count(dict: &DictNumByStr) -> usize {
    dict.map.len()
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_num_by_str_free(ptr: PDictMut) {
    ptr_free(ptr);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_num_by_str_get_count() {
        let mut f = Dict::new(
            Duplicates::Replace,
            CaseFormat::NoFormat,
            String::from(";"),
            String::from(",="),
            true,
            false,
        );
        let content = f.load_file("src/test/keywords.txt");
        assert!(content.is_ok(), content.unwrap_err());

        unsafe {
            assert_eq!(dictionary_num_by_str_get_count(&f), 2);
        }
    }

    #[test]
    fn test_dictionary_num_by_str_find() {
        let mut f = Dict::new(
            Duplicates::Replace,
            CaseFormat::NoFormat,
            String::from(";"),
            String::from(",="),
            true,
            true,
        );
        let content = f.load_file("src/test/keywords-hex.txt");
        assert!(content.is_ok(), content.unwrap_err());

        unsafe {
            assert_eq!(dictionary_num_by_str_get_count(&f), 23);

            let mut i: i32 = 0;
            assert!(dictionary_num_by_str_find(&f, str_to_pchar("wait"), &mut i));
            assert_eq!(i, 1);
            i = -1;
            assert!(!dictionary_num_by_str_find(&f, str_to_pchar(""), &mut i));
            assert_eq!(i, -1);
        }
    }

    #[test]
    fn test_dictionary_num_by_str_find_in_uppercase() {
        let mut f = Dict::new(
            Duplicates::Replace,
            CaseFormat::UpperCase,
            String::from(";"),
            String::from(",="),
            true,
            true,
        );
        let content = f.load_file("src/test/keywords-hex.txt");
        assert!(content.is_ok(), content.unwrap_err());

        unsafe {
            assert_eq!(dictionary_num_by_str_get_count(&f), 23);

            let mut i: i32 = 0;
            assert!(dictionary_num_by_str_find(&f, str_to_pchar("WAIT"), &mut i));
            assert_eq!(i, 1);
            assert!(dictionary_num_by_str_find(&f, str_to_pchar("JUMP"), &mut i));
            assert_eq!(i, 2);
            i = -1;
            assert!(!dictionary_num_by_str_find(&f, str_to_pchar(""), &mut i));
            assert_eq!(i, -1);
        }
    }

    #[test]
    fn test_dictionary_num_by_str_duplicates_ignore() {
        let mut f = Dict::new(
            Duplicates::Ignore,
            CaseFormat::NoFormat,
            String::from(";"),
            String::from(",="),
            true,
            true,
        );
        let content = f.load_file("src/test/keywords-dups.txt");
        assert!(content.is_ok(), content.unwrap_err());
        unsafe {
            assert_eq!(dictionary_num_by_str_get_count(&f), 2);

            let mut i: i32 = 0;
            assert!(dictionary_num_by_str_find(&f, str_to_pchar("wait"), &mut i));
            assert_eq!(i, 1);
            assert!(dictionary_num_by_str_find(&f, str_to_pchar("jump"), &mut i));
            assert_eq!(i, 1);
        }
    }

    #[test]
    fn test_dictionary_num_by_str_duplicates_replace() {
        let mut f = Dict::new(
            Duplicates::Replace,
            CaseFormat::NoFormat,
            String::from(";"),
            String::from(",="),
            true,
            true,
        );
        let content = f.load_file("src/test/keywords-dups.txt");
        assert!(content.is_ok(), content.unwrap_err());
        unsafe {
            assert_eq!(dictionary_num_by_str_get_count(&f), 2);

            let mut i: i32 = 0;
            assert!(dictionary_num_by_str_find(&f, str_to_pchar("wait"), &mut i));
            assert_eq!(i, 1);
            assert!(dictionary_num_by_str_find(&f, str_to_pchar("jump"), &mut i));
            assert_eq!(i, 2);
        }
    }
}

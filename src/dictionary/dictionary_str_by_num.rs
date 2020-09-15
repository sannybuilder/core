use crate::common_ffi::*;
use crate::dictionary::ffi::*;
use std::ffi::CString;

type DictStrByNum = Dict<i32, CString>;
type PDict = *const DictStrByNum;
type PDictMut = *mut DictStrByNum;

#[no_mangle]
pub extern "C" fn dictionary_str_by_num_new(
    duplicates: u8,
    hex_keys: bool,
    case_format: u8,
    comments: PChar,
    delimiters: PChar,
    trim: bool,
) -> PDict {
    ptr_new(Dict::new(
        duplicates.into(),
        case_format.into(),
        pchar_to_string(comments),
        pchar_to_string(delimiters),
        trim,
        hex_keys,
    ))
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_load_file(
    dict: &mut DictStrByNum,
    file_name: PChar,
) -> bool {
    if let Ok(_) = dict.load_file(pchar_to_str(file_name)) {
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_find(
    dict: &DictStrByNum,
    id: i32,
    out: *mut PChar,
) -> bool {
    if let Some(val) = dict.map.get(&id) {
        *out = val.as_ptr();
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_get_entry(
    dict: &DictStrByNum,
    index: usize,
    out_key: *mut i32,
    out_value: *mut PChar,
) -> bool {
    if let Some((&key, value)) = dict.map.iter().nth(index) {
        *out_key = key;
        *out_value = value.as_ptr();
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_get_count(dict: &DictStrByNum) -> usize {
    dict.map.len()
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_free(ptr: PDictMut) {
    ptr_free(ptr);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
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
            assert_eq!(dictionary_str_by_num_get_count(&f), 2);
        }
    }

    #[test]
    fn test_dictionary_str_by_num_find_uppercase() {
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
            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_num_find(&f, 2, &mut s));
            assert_eq!(pchar_to_str(s), "JUMP");

            assert!(!dictionary_str_by_num_find(&f, 0, &mut s));
            assert_eq!(pchar_to_str(s), "JUMP");
        }
    }

    #[test]
    fn test_dictionary_str_by_num_find_lowercase() {
        let mut f = Dict::new(
            Duplicates::Replace,
            CaseFormat::LowerCase,
            String::from(";"),
            String::from(",="),
            true,
            true,
        );
        let content = f.load_file("src/test/keywords-hex.txt");
        assert!(content.is_ok(), content.unwrap_err());
        unsafe {
            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_num_find(&f, 2, &mut s));
            assert_eq!(pchar_to_str(s), "jump");

            assert!(!dictionary_str_by_num_find(&f, 0, &mut s));
            assert_eq!(pchar_to_str(s), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_num_duplicates_ignore() {
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
            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_num_find(&f, 1, &mut s));
            assert_eq!(pchar_to_str(s), "wait");

            assert!(dictionary_str_by_num_find(&f, 2, &mut s));
            assert_eq!(pchar_to_str(s), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_num_duplicates_replace() {
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
            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_num_find(&f, 1, &mut s));
            assert_eq!(pchar_to_str(s), "jump");

            assert!(dictionary_str_by_num_find(&f, 2, &mut s));
            assert_eq!(pchar_to_str(s), "jump");
        }
    }
}

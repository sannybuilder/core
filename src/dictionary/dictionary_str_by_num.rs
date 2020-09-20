use crate::common_ffi::*;
use crate::dictionary::ffi::*;
use std::ffi::CString;

type DictStrByNum = Dict<i32, CString>;

#[no_mangle]
pub extern "C" fn dictionary_str_by_num_new(
    duplicates: u8,
    hex_keys: bool,
    case_format: u8,
    comments: PChar,
    delimiters: PChar,
    trim: bool,
) -> *mut DictStrByNum {
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
    dict: *mut DictStrByNum,
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
pub unsafe extern "C" fn dictionary_str_by_num_find(
    dict: *mut DictStrByNum,
    id: i32,
    out: *mut PChar,
) -> bool {
    if let Some(ptr) = dict.as_mut() {
        if let Some(val) = ptr.map.get(&id) {
            *out = val.as_ptr();
            return true;
        }
    }
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_num_get_entry(
    dict: *mut DictStrByNum,
    index: usize,
    out_key: *mut i32,
    out_value: *mut PChar,
) -> bool {
    if let Some(ptr) = dict.as_mut() {
        if let Some((&key, value)) = ptr.map.iter().nth(index) {
            *out_key = key;
            *out_value = value.as_ptr();
            return true;
        }
    }
    return false;
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
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded = dictionary_str_by_num_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords.txt"),
            );
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
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded = dictionary_str_by_num_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords-hex.txt"),
            );
            assert!(loaded);
            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_num_find(f, 2, &mut s));
            assert_eq!(pchar_to_str(s), "JUMP");

            assert!(!dictionary_str_by_num_find(f, 0, &mut s));
            assert_eq!(pchar_to_str(s), "JUMP");
        }
    }

    #[test]
    fn test_dictionary_str_by_num_find_lowercase() {
        unsafe {
            let f = dictionary_str_by_num_new(
                Duplicates::Replace.into(),
                true,
                CaseFormat::LowerCase.into(),
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded = dictionary_str_by_num_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords-hex.txt"),
            );
            assert!(loaded);

            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_num_find(f, 2, &mut s));
            assert_eq!(pchar_to_str(s), "jump");

            assert!(!dictionary_str_by_num_find(f, 0, &mut s));
            assert_eq!(pchar_to_str(s), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_num_duplicates_ignore() {
        unsafe {
            let f = dictionary_str_by_num_new(
                Duplicates::Ignore.into(),
                true,
                CaseFormat::NoFormat.into(),
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded = dictionary_str_by_num_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords-dups.txt"),
            );
            assert!(loaded);
            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_num_find(f, 1, &mut s));
            assert_eq!(pchar_to_str(s), "wait");

            assert!(dictionary_str_by_num_find(f, 2, &mut s));
            assert_eq!(pchar_to_str(s), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_num_duplicates_replace() {
        unsafe {
            let f = dictionary_str_by_num_new(
                Duplicates::Replace.into(),
                true,
                CaseFormat::NoFormat.into(),
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded = dictionary_str_by_num_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords-dups.txt"),
            );
            assert!(loaded);

            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_num_find(f, 1, &mut s));
            assert_eq!(pchar_to_str(s), "jump");

            assert!(dictionary_str_by_num_find(f, 2, &mut s));
            assert_eq!(pchar_to_str(s), "jump");
        }
    }
}

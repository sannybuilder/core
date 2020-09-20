use crate::common_ffi::*;
use crate::dictionary::ffi::*;
use std::ffi::CString;

type DictStrByStr = Dict<CString, CString>;

#[no_mangle]
pub extern "C" fn dictionary_str_by_str_new(
    duplicates: u8,
    case_format: u8,
    comments: PChar,
    delimiters: PChar,
    trim: bool,
) -> *mut DictStrByStr {
    ptr_new(Dict::new(
        duplicates.into(),
        case_format.into(),
        pchar_to_string(comments),
        pchar_to_string(delimiters),
        trim,
        false,
    ))
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_load_file(
    dict: *mut DictStrByStr,
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
pub unsafe extern "C" fn dictionary_str_by_str_add(
    dict: *mut DictStrByStr,
    key: PChar,
    value: PChar,
) -> bool {
    if let Some(ptr) = dict.as_mut() {
        if let Ok(key) = CString::new(pchar_to_str(key)) {
            if let Ok(value) = CString::new(pchar_to_str(value)) {
                if ptr.should_add(&key) {
                    ptr.map.insert(key, value);
                    return true;
                }
            }
        }
    }
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_find(
    dict: *mut DictStrByStr,
    key: PChar,
    out: *mut PChar,
) -> bool {
    if let Some(ptr) = dict.as_mut() {
        if let Ok(name) = CString::new(pchar_to_str(key).to_ascii_lowercase()) {
            if let Some(val) = ptr.map.get(&name) {
                *out = val.as_ptr();
                return true;
            }
        }
    }
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_get_entry(
    dict: *mut DictStrByStr,
    index: usize,
    out_key: *mut PChar,
    out_value: *mut PChar,
) -> bool {
    if let Some(ptr) = dict.as_mut() {
        if let Some((key, value)) = ptr.map.iter().nth(index) {
            *out_key = key.as_ptr();
            *out_value = value.as_ptr();
            return true;
        }
    }
    return false;
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
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded = dictionary_str_by_str_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords.txt"),
            );
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
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded = dictionary_str_by_str_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords-hex.txt"),
            );
            assert!(loaded);
            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_str_find(f, str_to_pchar("0002"), &mut s));
            assert_eq!(pchar_to_str(s), "JUMP");

            assert!(!dictionary_str_by_str_find(f, str_to_pchar("0000"), &mut s));
            assert_eq!(pchar_to_str(s), "JUMP");
        }
    }

    #[test]
    fn test_dictionary_str_by_str_find_lowercase() {
        unsafe {
            let f = dictionary_str_by_str_new(
                Duplicates::Replace.into(),
                CaseFormat::LowerCase.into(),
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded = dictionary_str_by_str_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords-hex.txt"),
            );
            assert!(loaded);

            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_str_find(f, str_to_pchar("0002"), &mut s));
            assert_eq!(pchar_to_str(s), "jump");

            assert!(!dictionary_str_by_str_find(f, str_to_pchar("0000"), &mut s));
            assert_eq!(pchar_to_str(s), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_str_duplicates_ignore() {
        unsafe {
            let f = dictionary_str_by_str_new(
                Duplicates::Ignore.into(),
                CaseFormat::LowerCase.into(),
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded = dictionary_str_by_str_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords-dups.txt"),
            );
            assert!(loaded);

            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_str_find(f, str_to_pchar("0001"), &mut s));
            assert_eq!(pchar_to_str(s), "wait");

            assert!(dictionary_str_by_str_find(f, str_to_pchar("0002"), &mut s));
            assert_eq!(pchar_to_str(s), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_str_duplicates_replace() {
        unsafe {
            let f = dictionary_str_by_str_new(
                Duplicates::Replace.into(),
                CaseFormat::NoFormat.into(),
                str_to_pchar(";"),
                str_to_pchar(",="),
                true,
            );

            assert!(f.as_mut().is_some());
            let loaded = dictionary_str_by_str_load_file(
                f,
                str_to_pchar("src/dictionary/test/keywords-dups.txt"),
            );
            assert!(loaded);

            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_str_find(f, str_to_pchar("0001"), &mut s));
            assert_eq!(pchar_to_str(s), "jump");

            assert!(dictionary_str_by_str_find(f, str_to_pchar("0002"), &mut s));
            assert_eq!(pchar_to_str(s), "jump");
        }
    }
}

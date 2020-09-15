use crate::common_ffi::*;
use crate::dictionary::ffi::*;
use std::ffi::CString;

type DictStrByStr = Dict<CString, CString>;
type PDict = *const DictStrByStr;
type PDictMut = *mut DictStrByStr;

#[no_mangle]
pub extern "C" fn dictionary_str_by_str_new(
    duplicates: u8,
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
        false,
    ))
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_load_file(
    dict: &mut DictStrByStr,
    file_name: PChar,
) -> bool {
    if let Ok(_) = dict.load_file(pchar_to_str(file_name)) {
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_add(
    dict: &mut DictStrByStr,
    key: PChar,
    value: PChar,
) -> bool {
    if let Ok(key) = CString::new(pchar_to_str(key)) {
        if let Ok(value) = CString::new(pchar_to_str(value)) {
            if dict.should_add(&key) {
                dict.map.insert(key, value);
                return true;
            }
        }
    }
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_find(
    dict: &DictStrByStr,
    key: PChar,
    out: *mut PChar,
) -> bool {
    if let Ok(name) = CString::new(pchar_to_str(key).to_ascii_lowercase()) {
        if let Some(val) = dict.map.get(&name) {
            *out = val.as_ptr();
            return true;
        }
    }
    return false;
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_get_entry(
    dict: &DictStrByStr,
    index: usize,
    out_key: *mut PChar,
    out_value: *mut PChar,
) -> bool {
    if let Some((key, value)) = dict.map.iter().nth(index) {
        *out_key = key.as_ptr();
        *out_value = value.as_ptr();
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_get_count(dict: &DictStrByStr) -> usize {
    dict.map.len()
}

#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_free(ptr: PDictMut) {
    ptr_free(ptr);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dictionary_str_by_str_get_count() {
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
            assert_eq!(dictionary_str_by_str_get_count(&f), 2);
        }
    }

    #[test]
    fn test_dictionary_str_by_str_find_uppercase() {
        let mut f = Dict::new(
            Duplicates::Replace,
            CaseFormat::UpperCase,
            String::from(";"),
            String::from(",="),
            true,
            false,
        );
        let content = f.load_file("src/test/keywords-hex.txt");
        assert!(content.is_ok(), content.unwrap_err());
        unsafe {
            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_str_find(&f, str_to_pchar("0002"), &mut s));
            assert_eq!(pchar_to_str(s), "JUMP");

            assert!(!dictionary_str_by_str_find(
                &f,
                str_to_pchar("0000"),
                &mut s
            ));
            assert_eq!(pchar_to_str(s), "JUMP");
        }
    }

    #[test]
    fn test_dictionary_str_by_str_find_lowercase() {
        let mut f = Dict::new(
            Duplicates::Replace,
            CaseFormat::LowerCase,
            String::from(";"),
            String::from(",="),
            true,
            false,
        );
        let content = f.load_file("src/test/keywords-hex.txt");
        assert!(content.is_ok(), content.unwrap_err());
        unsafe {
            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_str_find(&f, str_to_pchar("0002"), &mut s));
            assert_eq!(pchar_to_str(s), "jump");

            assert!(!dictionary_str_by_str_find(
                &f,
                str_to_pchar("0000"),
                &mut s
            ));
            assert_eq!(pchar_to_str(s), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_str_duplicates_ignore() {
        let mut f = Dict::new(
            Duplicates::Ignore,
            CaseFormat::LowerCase,
            String::from(";"),
            String::from(",="),
            true,
            false,
        );
        let content = f.load_file("src/test/keywords-dups.txt");
        assert!(content.is_ok(), content.unwrap_err());
        unsafe {
            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_str_find(&f, str_to_pchar("0001"), &mut s));
            assert_eq!(pchar_to_str(s), "wait");

            assert!(dictionary_str_by_str_find(&f, str_to_pchar("0002"), &mut s));
            assert_eq!(pchar_to_str(s), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_str_duplicates_replace() {
        let mut f = Dict::new(
            Duplicates::Replace,
            CaseFormat::NoFormat,
            String::from(";"),
            String::from(",="),
            true,
            false,
        );
        let content = f.load_file("src/test/keywords-dups.txt");
        assert!(content.is_ok(), content.unwrap_err());
        unsafe {
            let mut s = str_to_pchar("");
            assert!(dictionary_str_by_str_find(&f, str_to_pchar("0001"), &mut s));
            assert_eq!(pchar_to_str(s), "jump");

            assert!(dictionary_str_by_str_find(&f, str_to_pchar("0002"), &mut s));
            assert_eq!(pchar_to_str(s), "jump");
        }
    }
}

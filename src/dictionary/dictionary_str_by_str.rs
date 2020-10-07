use crate::common_ffi::*;
use crate::dictionary::ffi::*;
use std::ffi::CString;

pub type DictStrByStr = Dict<CString, CString>;

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
       let dict = dict.as_mut()?;
       let key = CString::new(pchar_to_str(key)?).ok()?;
       let value = CString::new(pchar_to_str(value)?).ok()?;
       dict.add(key, value);
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
        let key = CString::new(pchar_to_str(key)?.to_ascii_lowercase()).ok()?;
        *out = dict.as_mut()?.map.get(&key)?.as_ptr();
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

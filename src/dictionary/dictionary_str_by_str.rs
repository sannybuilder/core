use crate::common_ffi::*;
use crate::dictionary::ffi::*;
use std::ffi::CString;

use super::config::ConfigBuilder;

pub type DictStrByStr = Dict<CString, CString>;

#[no_mangle]
pub extern "C" fn dictionary_str_by_str_new(
    duplicates: u8,
    case_format: u8,
    comments: PChar,
    delimiters: PChar,
    strip_whitespace: bool,
) -> *mut DictStrByStr {
    let mut builder = ConfigBuilder::new();

    builder
        .set_duplicates(duplicates.into())
        .set_case_format(case_format.into())
        .set_strip_whitespace(strip_whitespace)
        .set_hex_keys(false);

    if let Some(comments) = pchar_to_string(comments) {
        builder.set_comments(comments);
    }
    if let Some(delimiters) = pchar_to_string(delimiters) {
        builder.set_delimiters(delimiters);
    }
    ptr_new(Dict::new(builder.build()))
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
       dict.as_mut()?.add_with_format(pchar_to_str(key)?, pchar_to_str(value)?);
       Some(())
    }}
}
#[no_mangle]
pub unsafe extern "C" fn dictionary_str_by_str_remove(dict: *mut DictStrByStr, key: PChar) -> bool {
    boolclosure! {{
       dict.as_mut()?.remove(pchar_to_str(key)?);
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
        let key = apply_format(pchar_to_str(key)?, &CaseFormat::LowerCase)?; // should match KeyValue for (CString, CString)
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
            let comments = std::ffi::CString::new(";").unwrap();
            let delimiter = std::ffi::CString::new(",=").unwrap();
            let f = dictionary_str_by_str_new(
                Duplicates::Replace.into(),
                CaseFormat::NoFormat.into(),
                comments.as_ptr(),
                delimiter.as_ptr(),
                true,
            );

            assert!(f.as_mut().is_some());
            let file = std::ffi::CString::new("src/dictionary/test/keywords.txt").unwrap();
            let loaded = dictionary_str_by_str_load_file(f, file.as_ptr());
            assert!(loaded);

            assert_eq!(dictionary_str_by_str_get_count(f), 2);
        }
    }

    #[test]
    fn test_dictionary_str_by_str_find_uppercase() {
        unsafe {
            let comments = std::ffi::CString::new(";").unwrap();
            let delimiter = std::ffi::CString::new(",=").unwrap();
            let f = dictionary_str_by_str_new(
                Duplicates::Replace.into(),
                CaseFormat::UpperCase.into(),
                comments.as_ptr(),
                delimiter.as_ptr(),
                true,
            );

            assert!(f.as_mut().is_some());
            let file = std::ffi::CString::new("src/dictionary/test/keywords-hex.txt").unwrap();
            let loaded = dictionary_str_by_str_load_file(f, file.as_ptr());
            assert!(loaded);

            let mut ptr = 0;
            use std::mem::transmute;

            let op = std::ffi::CString::new("0002").unwrap();
            assert!(dictionary_str_by_str_find(
                f,
                op.as_ptr(),
                transmute(&mut ptr)
            ));
            assert_eq!(pchar_to_str(transmute(ptr)).unwrap(), "JUMP");

            let op = std::ffi::CString::new("0000").unwrap();
            assert!(!dictionary_str_by_str_find(
                f,
                op.as_ptr(),
                transmute(&mut ptr)
            ));
            assert_eq!(pchar_to_str(transmute(ptr)).unwrap(), "JUMP");
        }
    }

    #[test]
    fn test_dictionary_str_by_str_find_lowercase() {
        unsafe {
            let comments = std::ffi::CString::new(";").unwrap();
            let delimiter = std::ffi::CString::new(",=").unwrap();
            let f = dictionary_str_by_str_new(
                Duplicates::Replace.into(),
                CaseFormat::LowerCase.into(),
                comments.as_ptr(),
                delimiter.as_ptr(),
                true,
            );

            assert!(f.as_mut().is_some());
            let file = std::ffi::CString::new("src/dictionary/test/keywords-hex.txt").unwrap();
            let loaded = dictionary_str_by_str_load_file(f, file.as_ptr());
            assert!(loaded);

            let mut ptr = 0;
            use std::mem::transmute;

            let op = std::ffi::CString::new("0002").unwrap();
            assert!(dictionary_str_by_str_find(
                f,
                op.as_ptr(),
                transmute(&mut ptr)
            ));
            assert_eq!(pchar_to_str(transmute(ptr)).unwrap(), "jump");

            let op = std::ffi::CString::new("0000").unwrap();
            assert!(!dictionary_str_by_str_find(
                f,
                op.as_ptr(),
                transmute(&mut ptr)
            ));
            assert_eq!(pchar_to_str(transmute(ptr)).unwrap(), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_str_duplicates_ignore() {
        unsafe {
            let comments = std::ffi::CString::new(";").unwrap();
            let delimiter = std::ffi::CString::new(",=").unwrap();
            let f = dictionary_str_by_str_new(
                Duplicates::Ignore.into(),
                CaseFormat::LowerCase.into(),
                comments.as_ptr(),
                delimiter.as_ptr(),
                true,
            );

            assert!(f.as_mut().is_some());
            let file = std::ffi::CString::new("src/dictionary/test/keywords-dups.txt").unwrap();
            let loaded = dictionary_str_by_str_load_file(f, file.as_ptr());
            assert!(loaded);

            let mut ptr = 0;
            use std::mem::transmute;

            let op = std::ffi::CString::new("0001").unwrap();
            assert!(dictionary_str_by_str_find(
                f,
                op.as_ptr(),
                transmute(&mut ptr)
            ));
            assert_eq!(pchar_to_str(transmute(ptr)).unwrap(), "wait");

            let op = std::ffi::CString::new("0002").unwrap();
            assert!(dictionary_str_by_str_find(
                f,
                op.as_ptr(),
                transmute(&mut ptr)
            ));
            assert_eq!(pchar_to_str(transmute(ptr)).unwrap(), "jump");
        }
    }

    #[test]
    fn test_dictionary_str_by_str_duplicates_replace() {
        unsafe {
            let comments = std::ffi::CString::new(";").unwrap();
            let delimiter = std::ffi::CString::new(",=").unwrap();
            let f = dictionary_str_by_str_new(
                Duplicates::Replace.into(),
                CaseFormat::NoFormat.into(),
                comments.as_ptr(),
                delimiter.as_ptr(),
                true,
            );

            assert!(f.as_mut().is_some());
            let file = std::ffi::CString::new("src/dictionary/test/keywords-dups.txt").unwrap();
            let loaded = dictionary_str_by_str_load_file(f, file.as_ptr());
            assert!(loaded);

            let mut ptr = 0;
            use std::mem::transmute;

            let op = std::ffi::CString::new("0001").unwrap();
            assert!(dictionary_str_by_str_find(
                f,
                op.as_ptr(),
                transmute(&mut ptr)
            ));
            assert_eq!(pchar_to_str(transmute(ptr)).unwrap(), "jump");

            let op = std::ffi::CString::new("0002").unwrap();
            assert!(dictionary_str_by_str_find(
                f,
                op.as_ptr(),
                transmute(&mut ptr)
            ));
            assert_eq!(pchar_to_str(transmute(ptr)).unwrap(), "jump");
        }
    }
}

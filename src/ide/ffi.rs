use libc::c_char;

use super::model_list::ModelList;

use crate::common_ffi::*;

#[no_mangle]
pub extern "C" fn model_list_new() -> *mut ModelList {
    ptr_new(ModelList::new())
}

#[no_mangle]
pub unsafe extern "C" fn model_list_free(list: *mut ModelList) {
    ptr_free(list)
}

#[no_mangle]
pub unsafe extern "C" fn model_list_load_from_file(list: *mut ModelList, file_name: PChar) -> bool {
    boolclosure!({
        list.as_mut()?.load_from_file(pchar_to_str(file_name)?);
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn model_list_get_by_id(
    list: *mut ModelList,
    id: i32,
    out: *mut PChar,
) -> bool {
    boolclosure!({
        *out = list.as_mut()?.find_by_id(id)?.as_ptr();
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn model_list_get_by_name(
    list: *mut ModelList,
    name: PChar,
    out_id: *mut i32,
    out_type: *mut u8,
) -> bool {
    boolclosure!({
        let name = pchar_to_str(name)?;
        let model = list.as_mut()?.find_by_name(&name)?;
        *out_id = model.id;
        *out_type = model.r#type as u8;
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn model_list_filter_names(
    list: *mut ModelList,
    needle: PChar,
    dict: *mut crate::dictionary::dictionary_str_by_num::DictStrByNum,
) -> bool {
    boolclosure!({
        let needle = pchar_to_str(needle)?;
        let results = list.as_mut()?.filter_by_name(&needle);
        for (name, id) in results {
            dict.as_mut()?.add(id, name);
        }
        Some(())
    })
}

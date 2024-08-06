use super::source_map::SourceMap;
use crate::common_ffi::*;

#[no_mangle]
pub extern "C" fn source_map_new() -> *mut SourceMap {
    ptr_new(SourceMap::new())
}

#[no_mangle]
pub unsafe extern "C" fn source_map_free(map: *mut SourceMap) {
    ptr_free(map)
}

#[no_mangle]
pub unsafe extern "C" fn source_map_clear(map: *mut SourceMap) -> bool {
    boolclosure!({
        let map = map.as_mut()?;
        (*map).clear();
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn source_map_add(
    map: *mut SourceMap,
    path: PChar,
    line: u32,
    offset: u32,
) -> bool {
    boolclosure!({
        let path = pchar_to_str(path)?;
        (*map).add(path, line, offset);
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn source_map_get_offset(
    map: *mut SourceMap,
    path: PChar,
    line: u32,
    out: *mut u32,
) -> bool {
    boolclosure!({
        let path = pchar_to_str(path)?;
        *out = (*map).get_offset(path, line)?;
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn source_map_get_line(
    map: *mut SourceMap,
    path: PChar,
    offset: u32,
    out: *mut u32,
) -> bool {
    boolclosure!({
        let path = pchar_to_str(path)?;
        *out = (*map).get_line(path, offset)?;
        Some(())
    })
}


#[no_mangle]
pub unsafe extern "C" fn source_map_adjust_offset_by(map: *mut SourceMap, delta: u32) {
    (*map).adjust_offset_by(delta);
}

#[no_mangle]
pub unsafe extern "C" fn source_map_dump(map: *mut SourceMap, path: PChar) -> bool {
    boolclosure!({
        let path = pchar_to_str(path)?;
        let mut file = std::fs::File::create(path).ok()?;
        let map = map.as_ref()?;
        serde_json::to_writer(&mut file, map).ok()?;
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn source_map_new_local_variable_scope(
    map: *mut SourceMap,
    file_name: PChar,
    line: u32,
    var_name: PChar,
    var_index: i32,
) -> bool {
    boolclosure!({
        let file_name = pchar_to_str(file_name)?;
        let name = pchar_to_str(var_name)?;
        (*map).new_local_variable_scope(file_name, line, name, var_index);
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn source_map_find_local_variable_index(
    map: *mut SourceMap,
    file_name: PChar,
    line: u32,
    var_name: PChar,
    out: *mut i32,
) -> bool {
    boolclosure!({
        let file_name = pchar_to_str(file_name)?;
        let name = pchar_to_str(var_name)?;
        *out = (*map).find_local_variable_index(file_name, line, name)?;
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn source_map_new_global_variable(
    map: *mut SourceMap,
    name: PChar,
    var_index: i32,
) -> bool {
    boolclosure!({
        let name = pchar_to_str(name)?;
        (*map).new_global_variable(name, var_index);
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn source_map_find_global_variable_index(
    map: *mut SourceMap,
    name: PChar,
    out: *mut i32,
) -> bool {
    boolclosure!({
        let name = pchar_to_str(name)?;
        *out = (*map).find_global_variable_index(name)?;
        Some(())
    })
}

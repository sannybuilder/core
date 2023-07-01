use std::ffi::CString;

use super::table::*;
use crate::common_ffi::*;

#[no_mangle]
pub extern "C" fn legacy_ini_new(game: u8) -> *mut OpcodeTable {
    log::debug!("New legacy ini for game {:?}", Game::from(game));
    ptr_new(OpcodeTable::new(game.into()))
}

#[no_mangle]
pub unsafe extern "C" fn legacy_ini_free(table: *mut OpcodeTable) {
    log::debug!("Close legacy ini");
    ptr_free(table)
}

#[no_mangle]
pub unsafe extern "C" fn legacy_ini_load_file(table: *mut OpcodeTable, path: PChar) -> bool {
    boolclosure!({
        {
            let path = pchar_to_str(path)?;
            (*table).load_from_file(path);
            log::debug!(
                "File {path} loaded. Max opcode: {:04X}, Count: {}",
                (*table).get_max_opcode(),
                (*table).len()
            );
            Some(())
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn legacy_ini_get_param_real_index(
    table: *mut OpcodeTable,
    opcode: u16,
    index: u8,
) -> u8 {
    (*table).get_param_real_index(opcode, index as _)
}

#[no_mangle]
pub unsafe extern "C" fn legacy_ini_get_max_opcode(table: *mut OpcodeTable) -> u16 {
    (*table).get_max_opcode()
}

#[no_mangle]
pub unsafe extern "C" fn legacy_ini_get_params_count(table: *mut OpcodeTable, opcode: u16) -> u8 {
    (*table).get_params_count(opcode)
}

#[no_mangle]
pub unsafe extern "C" fn legacy_ini_get_param_type(
    table: *mut OpcodeTable,
    opcode: u16,
    index: u8,
) -> u8 {
    (*table).get_param_type(opcode, index as _) as u8
}

#[no_mangle]
pub unsafe extern "C" fn legacy_ini_does_word_exist(
    table: *mut OpcodeTable,
    opcode: u16,
    index: u8,
) -> bool {
    (*table).does_word_exist(opcode, index as _)
}

#[no_mangle]
pub unsafe extern "C" fn legacy_ini_get_word(
    table: *mut OpcodeTable,
    opcode: u16,
    index: u8,
    out: *mut PChar,
) -> bool {
    boolclosure! {{
        let word = (*table).get_word(opcode, index as _)?;
        *out = CString::into_raw(CString::new(word).unwrap());
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn legacy_ini_parse_line(table: *mut OpcodeTable, line: PChar) -> bool {
    boolclosure! {{
        (*table).parse_line(pchar_to_str(line)?);
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn legacy_ini_get_publisher(
    table: *mut OpcodeTable,
    out: *mut PChar,
) -> bool {
    boolclosure! {{
        let publisher = (*table).get_publisher()?;
        *out = CString::into_raw(CString::new(publisher).unwrap());
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn legacy_ini_get_date(table: *mut OpcodeTable, out: *mut PChar) -> bool {
    boolclosure! {{
        let date = (*table).get_date()?;
        *out = CString::into_raw(CString::new(date).unwrap());
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn legacy_ini_is_variadic_opcode(table: *mut OpcodeTable, opcode: u16) -> bool {
    (*table).is_variadic_opcode(opcode)
}

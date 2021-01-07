use std::ffi::CString;

use crate::{
    common_ffi::{pchar_to_str, pchar_to_string, ptr_free, ptr_new, PChar},
    language_service::server::LanguageServer,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolType {
    Number = 0,
    String = 1,
    Var = 2,
    Label = 3,
    ModelName = 4,
}
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub line_number: u32,
    pub _type: SymbolType,
}

pub struct DocumentInfo {
    pub is_active: bool,
}

pub struct SymbolInfoMap {
    pub file_name: Option<String>,
    pub line_number: u32,
    pub _type: SymbolType,
    pub value: Option<String>,
}
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Status {
    Disabled = 0,
    Idle = 1,
    Scanning = 2,
    PendingScan = 3,
    PendingRescan = 4,
    Unknown = 254,
    Error = 255,
}

#[derive(Debug, Clone)]
pub enum Source {
    Memory,
    File(String),
}

pub type EditorHandle = u32;
pub type StatusChangeCallback = extern "C" fn(EditorHandle, Status);

#[no_mangle]
pub extern "C" fn language_service_new(cb: StatusChangeCallback) -> *mut LanguageServer {
    ptr_new(LanguageServer::new(cb))
}

#[no_mangle]
pub unsafe extern "C" fn language_service_free(server: *mut LanguageServer) {
    ptr_free(server);
}

#[no_mangle]
pub unsafe extern "C" fn language_service_client_connect_with_file(
    server: *mut LanguageServer,
    file_name: PChar,
    handle: EditorHandle,
    static_constants_file: PChar,
) -> bool {
    boolclosure! {{
        server.as_mut()?.connect(Source::File(pchar_to_string(file_name)?), handle, pchar_to_str(static_constants_file)?);
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn language_service_client_connect_in_memory(
    server: *mut LanguageServer,
    handle: EditorHandle,
    static_constants_file: PChar,
) -> bool {
    boolclosure! {{
        server.as_mut()?.connect(Source::Memory, handle, pchar_to_str(static_constants_file)?);
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn language_service_client_notify_on_change(
    server: *mut LanguageServer,
    handle: EditorHandle,
    text: PChar,
) -> bool {
    boolclosure! {{
        server.as_mut()?.message_queue.send((handle, pchar_to_string(text)?)).ok()
    }}
}

#[no_mangle]
pub unsafe extern "C" fn language_service_client_disconnect(
    server: *mut LanguageServer,
    handle: EditorHandle,
) -> bool {
    boolclosure! {{
        server.as_mut()?.disconnect(handle);
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn language_service_find(
    server: *mut LanguageServer,
    symbol: PChar,
    handle: EditorHandle,
    out_value: *mut SymbolInfo,
) -> bool {
    boolclosure! {{
        let server = server.as_mut()?;
        let s = server.find(pchar_to_str(symbol)?, handle)?;
        out_value.as_mut()?._type = s._type;
        out_value.as_mut()?.line_number = s.line_number;
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn language_service_is_enabled(
    server: *mut LanguageServer,
    handle: EditorHandle,
) -> bool {
    boolclosure! {{
        server.as_mut()?.get_document_info(handle).is_active.then_some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn language_service_filter_constants_by_name(
    server: *mut LanguageServer,
    handle: EditorHandle,
    needle: PChar,
    dict: *mut crate::dictionary::dictionary_str_by_str::DictStrByStr,
) -> bool {
    boolclosure! {{
        let items = server.as_mut()?.filter_constants_by_name(pchar_to_str(needle)?, handle)?;
        for item in items {
            dict.as_mut()?.add(CString::new(item.0).ok()?, CString::new(item.1).ok()?)
        }
        Some(())
    }}
}

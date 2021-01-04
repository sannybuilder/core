use crate::{
    common_ffi::{pchar_to_str, pchar_to_string, ptr_free, ptr_new, PChar},
    language_service::server::LanguageServer,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum SymbolType {
    Number = 0,
    String = 1,
    Var = 2,
    Label = 3,
    ModelName = 4,
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
}
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Status {
    Disabled = 0,
    Ready = 1,
    Scanning = 2,
    PendingUpdate = 3,
    Unknown = 254,
    Error = 255,
}

#[derive(Debug, Clone)]
pub enum Source {
    Memory,
    File(String),
}

pub type EditorHandle = u32;
pub type NotifyCallback = extern "C" fn(EditorHandle);
pub type StatusChangeCallback = extern "C" fn(EditorHandle, Status);

#[no_mangle]
pub extern "C" fn language_service_new(
    cb1: NotifyCallback,
    cb2: StatusChangeCallback,
) -> *mut LanguageServer {
    ptr_new(LanguageServer::new(cb1, cb2))
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
pub unsafe extern "C" fn language_service_info(
    server: *mut LanguageServer,
    handle: EditorHandle,
    out_value: *mut DocumentInfo,
) -> bool {
    boolclosure! {{
        let info = out_value.as_mut()?;
        info.is_active = server.as_mut()?.get_document_info(handle).is_active;
        Some(())
    }}
}

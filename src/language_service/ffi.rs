use crate::{
    common_ffi::{pchar_to_str, ptr_free, ptr_new, PChar},
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
    pub file_name: String,
    pub line_number: u32,
    pub _type: SymbolType,
}
#[derive(Clone, Copy)]
pub enum Status {
    Ready = 1,
    Scanning = 2,
}

pub type EditorHandle = u32;
pub type NotifyCallback = extern "C" fn(EditorHandle);

pub type StatusChangeCallback = extern "C" fn(EditorHandle, i32);

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
pub unsafe extern "C" fn language_service_client_connect(
    server: *mut LanguageServer,
    file_name: PChar,
    handle: EditorHandle,
    static_constants_file: PChar,
) -> bool {
    boolclosure! {{
        server.as_mut()?.connect(pchar_to_str(file_name)?, handle, pchar_to_str(static_constants_file)?);
        Some(())
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
    file_name: PChar,
    out_value: *mut SymbolInfo,
) -> bool {
    boolclosure! {{
        let server = server.as_mut()?;
        let s = server.find(pchar_to_str(symbol)?, handle, pchar_to_str(file_name)?)?;
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

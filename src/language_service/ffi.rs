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
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SymbolInfo {
    pub line_number: u32,
    pub _type: SymbolType,
}

pub struct SymbolInfoMap {
    pub file_name: String,
    pub line_number: u32,
    pub _type: SymbolType,
}

pub type EditorHandle = u32;
pub type NotifyCallback = extern "C" fn(EditorHandle);

#[no_mangle]
pub extern "C" fn language_service_new(cb: NotifyCallback) -> *mut LanguageServer {
    ptr_new(LanguageServer::new(cb))
}

#[no_mangle]
pub unsafe extern "C" fn language_service_free(server: *mut LanguageServer) {
    ptr_free(server);
}

#[no_mangle]
pub unsafe extern "C" fn language_service_client_file_open(
    server: *mut LanguageServer,
    file_name: PChar,
    handle: EditorHandle,
) -> bool {
    boolclosure! {{
        server.as_mut()?.open(pchar_to_str(file_name)?, handle);
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn language_service_client_file_close(
    server: *mut LanguageServer,
    file_name: PChar,
    handle: EditorHandle,
) -> bool {
    boolclosure! {{
        server.as_mut()?.close(pchar_to_str(file_name)?, handle);
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn language_service_find(
    server: *mut LanguageServer,
    symbol: PChar,
    editor: EditorHandle,
    file_name: PChar,
    out_value: *mut SymbolInfo,
) -> bool {
    boolclosure! {{
        let server = server.as_mut()?;
        let s = server.find(pchar_to_str(symbol)?, editor, pchar_to_str(file_name)?)?;
        out_value.as_mut()?._type = s._type;
        out_value.as_mut()?.line_number = s.line_number;
        Some(())
    }}
}

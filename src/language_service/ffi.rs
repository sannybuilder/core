use std::ffi::CString;

use crate::{
    common_ffi::{pchar_to_str, pchar_to_string, ptr_free, ptr_new, PChar},
    language_service::server::LanguageServer,
    v4::helpers::token_str,
};

use super::symbol_table::SymbolType;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SymbolInfoRaw {
    pub _type: SymbolType,
    pub value: PChar,
    pub name_no_format: PChar,
    pub annotation: PChar,
}

pub struct DocumentInfo {
    pub is_active: bool,
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

#[no_mangle]
pub extern "C" fn language_service_new() -> *mut LanguageServer {
    ptr_new(LanguageServer::new())
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
    classes_file: PChar,
) -> bool {
    boolclosure! {{
        server.as_mut()?.connect(
            Source::File(pchar_to_string(file_name)?),
            handle,
            pchar_to_str(static_constants_file)?,
            pchar_to_str(classes_file)?
        );
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn language_service_client_connect_in_memory(
    server: *mut LanguageServer,
    handle: EditorHandle,
    static_constants_file: PChar,
    classes_file: PChar,
) -> bool {
    boolclosure! {{
        server.as_mut()?.connect(
            Source::Memory,
            handle,
            pchar_to_str(static_constants_file)?,
            pchar_to_str(classes_file)?
        );
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
    line_number: u32,
    out_value: *mut SymbolInfoRaw,
) -> bool {
    boolclosure! {{
        let server = server.as_mut()?;
        let s = server.find(pchar_to_str(symbol)?, handle, line_number)?;
        let out_value = out_value.as_mut()?;
        out_value._type = s._type;

        // don't return line numbers as they are not used on the client side
        // out_value.line_number = s.line_number;
        // out_value.end_line_number = s.end_line_number;

        out_value.value = CString::new(s.value.unwrap_or_default()).unwrap().into_raw();
        out_value.name_no_format = CString::new(s.name_no_format).unwrap().into_raw();
        out_value.annotation = CString::new(s.annotation.unwrap_or_default()).unwrap().into_raw();
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
    line_number: u32,
    dict: *mut crate::dictionary::dictionary_str_by_str::DictStrByStr,
) -> bool {
    boolclosure! {{
        let items = server.as_mut()?.filter_constants_by_name(pchar_to_str(needle)?, handle, line_number)?;
        for item in items {
            dict.as_mut()?.add(CString::new(item).ok()?, CString::new("").ok()?) // todo: use simple list instead of dict
        }
        Some(())
    }}
}

#[no_mangle]
pub unsafe extern "C" fn language_service_format_function_signature(
    server: *mut LanguageServer,
    value: PChar,
    out: *mut PChar,
) -> bool {
    boolclosure! {{
        let _server = server.as_mut()?;
        use crate::parser::{function_arguments_and_return_types, Span};

        let line = pchar_to_str(value)?;
        let (_, ref signature) = function_arguments_and_return_types(Span::from(line)).ok()?;

        let params = signature.0
            .iter()
            .map(|param|{
                let type_token = token_str(line, &param._type);
                let name_token = param.name.as_ref().map(|name| token_str(line, name));

                match name_token {
                    Some(name) => format!("\"{}: {}\"", name, type_token),
                    None => format!("\"{}\"", type_token),
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        // let return_types = signature.1
        //     .iter()
        //     .map(|ret_type| format!("\"{}\"", token_str(line, &ret_type.token)))
        //     .collect::<Vec<_>>()
        //     .join(", ");
        *out = CString::new(format!("{params}")).unwrap().into_raw();
        Some(())
    }}
}

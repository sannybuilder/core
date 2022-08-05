use super::service::UpdateService;
use crate::common_ffi::{pchar_to_str, ptr_free, ptr_new, PChar};

pub type StatusChangeCallback = extern "C" fn(PChar);

#[no_mangle]
pub extern "C" fn update_service_new(cb: StatusChangeCallback) -> *mut UpdateService {
    ptr_new(UpdateService::new(cb))
}

#[no_mangle]
pub unsafe extern "C" fn update_service_free(service: *mut UpdateService) {
    ptr_free(service);
}

#[no_mangle]
pub unsafe extern "C" fn update_service_check_version(
    service: *mut UpdateService,
    params: PChar,
) -> bool {
    boolclosure! {{
        service.as_mut()?.check_version(pchar_to_str(params)?)
    }}
}

#[no_mangle]
pub unsafe extern "C" fn update_service_auto_update(
    service: *mut UpdateService,
    params: PChar,
) -> bool {
    boolclosure! {{
        service.as_mut()?.auto_update(pchar_to_str(params)?)
    }}
}

#[no_mangle]
pub unsafe extern "C" fn update_service_download(
    service: *mut UpdateService,
    params: PChar,
) -> bool {
    boolclosure! {{
        service.as_mut()?.download(pchar_to_str(params)?)
    }}
}

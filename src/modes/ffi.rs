use std::path::Path;

use libc::c_char;

use super::modes::ModeManager;

use crate::common_ffi::*;

#[no_mangle]
pub extern "C" fn mode_manager_new() -> *mut ModeManager {
    ptr_new(ModeManager::new())
}

#[no_mangle]
pub unsafe extern "C" fn mode_manager_free(manager: *mut ModeManager) {
    ptr_free(manager)
}

#[no_mangle]
pub unsafe extern "C" fn mode_manager_load_from_dir(manager: *mut ModeManager, dir: PChar) -> bool {
    boolclosure!({
        let path = Path::new(pchar_to_str(dir)?);
        if !path.is_dir() {
            return None;
        }
        manager.as_mut()?.load_from_directory(path);
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn mode_manager_set_current_mode_by_id(
    manager: *mut ModeManager,
    mode_id: PChar,
) -> bool {
    boolclosure!({
        let id = pchar_to_str(mode_id)?;
        manager.as_mut()?.set_current_mode_by_id(&id);
        Some(())
    })
}

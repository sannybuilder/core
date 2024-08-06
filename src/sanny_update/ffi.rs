use crate::common_ffi::{pchar_to_str, PChar};

#[no_mangle]
pub unsafe extern "C" fn sb_download_update(channel: u8, local_version: PChar) -> bool {
    boolclosure!({
        super::download_update(channel.into(), pchar_to_str(local_version)?).then_some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn sb_update_exists(channel: u8, local_version: PChar) -> bool {
    boolclosure!({ super::has_update(channel.into(), pchar_to_str(local_version)?).then_some(()) })
}

#[no_mangle]
pub unsafe extern "C" fn sb_update_cleanup() {
    super::cleanup();
}

#[no_mangle]
pub unsafe extern "C" fn sb_update_get_remote_version(channel: u8, out: *mut PChar) -> bool {
    boolclosure!({
        let remote_version = super::get_latest_version_from_github(channel.into())?;
        *out = std::ffi::CString::new(remote_version).unwrap().into_raw();
        Some(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn sb_update_get_release_notes(channel: u8, out: *mut PChar) -> bool {
    boolclosure!({
        let release_notes = super::get_release_notes_from_github(channel.into())?;
        *out = std::ffi::CString::new(release_notes).unwrap().into_raw();
        Some(())
    })
}

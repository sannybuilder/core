use crate::common_ffi::{pchar_to_string, PChar};

#[no_mangle]
pub unsafe extern "C" fn v4_try_transform(input: PChar, game: u8, out: *mut PChar) -> bool {
    boolclosure! {{
        let input = pchar_to_string(input)?;
        let game = super::game::Game::try_from(game).ok()?;
        let result = super::transform(&input, game)?;
        *out = std::ffi::CString::new(result).unwrap().into_raw();
        Some(())
    }}
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Game {
    gta3,
    vc,
    sa,
    lcs,
    vcs,
    sa_mobile,
}

impl TryFrom<u8> for Game {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Game::gta3),
            1 => Ok(Game::vc),
            2 => Ok(Game::sa),
            3 => Ok(Game::lcs),
            4 => Ok(Game::vcs),
            5 => Ok(Game::sa_mobile),
            _ => Err(()),
        }
    }
}

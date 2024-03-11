#![feature(never_type)]
#![feature(hash_drain_filter)]

use ctor::ctor;
use simplelog::*;

#[macro_use]
pub mod common_ffi;
pub mod dictionary;
pub mod language_service;
pub mod legacy_ini;
pub mod namespaces;
pub mod parser;
pub mod sdk;
pub mod update_service;
pub mod utils;
pub mod v4;
pub mod source_map;

#[ctor]
fn main() {
    let config = ConfigBuilder::new()
        .set_level_padding(LevelPadding::Off)
        .set_time_to_local(true)
        .set_thread_level(LevelFilter::Off)
        .build();

    let cwd = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let _ = WriteLogger::init(
        LevelFilter::max(),
        config,
        std::fs::File::create(cwd.join("core.log")).unwrap(),
    );

    log::info!("core library loaded");
}

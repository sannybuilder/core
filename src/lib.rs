#![feature(never_type)]
#![feature(hash_drain_filter)]

use ctor::ctor;
use simplelog::*;

#[macro_use]
pub mod common_ffi;
pub mod dictionary;
pub mod language_service;
pub mod v4;
pub mod namespaces;
pub mod parser;
pub mod sdk;
pub mod update_service;
pub mod utils;

#[ctor]
fn main() {
    if cfg!(debug_assertions) {
        let config = ConfigBuilder::new()
            .set_level_padding(LevelPadding::Off)
            .set_time_to_local(true)
            .set_thread_level(LevelFilter::Off)
            .build();

        let _ = WriteLogger::init(
            LevelFilter::max(),
            config,
            std::fs::File::create("core.log").unwrap(),
        );
    }

    log::debug!("core library loaded");
}

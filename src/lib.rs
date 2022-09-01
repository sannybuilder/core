#![feature(never_type)]
#![feature(bool_to_option)]
#![feature(hash_drain_filter)]

use ctor::ctor;
use simplelog::*;

#[macro_use]
pub mod common_ffi;
pub mod dictionary;
pub mod language_service;
pub mod math;
pub mod namespaces;
pub mod parser;
pub mod sdk;
pub mod update_service;
pub mod utils;

#[ctor]
fn main() {
    if cfg!(debug_assertions) {
        let config = ConfigBuilder::new().set_time_to_local(true).build();

        let _ = WriteLogger::init(
            LevelFilter::max(),
            config,
            std::fs::File::create("core.log").unwrap(),
        );
    }

    log::debug!("core library loaded");
}

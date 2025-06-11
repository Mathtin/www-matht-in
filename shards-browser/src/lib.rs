mod utils;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use std::sync::OnceLock;

static STARTED: OnceLock<()> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
fn init_log() {
    use log::Level;
    console_log::init_with_level(Level::Trace).expect("error initializing log");
}

#[cfg(not(target_arch = "wasm32"))]
fn init_log() {
    env_logger::init();
}  

fn first_startup() {
    init_log();
    log::debug!("Shards browser starting!");

    #[cfg(target_arch = "wasm32")]
    utils::set_panic_hook();

    log::debug!("Shards browser started!");
}


#[wasm_bindgen]
pub fn start() {
    STARTED.get_or_init(first_startup);
    log::debug!("Start click");
}

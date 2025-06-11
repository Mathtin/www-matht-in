mod utils;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use std::sync::OnceLock;

static START_SUCCESS: OnceLock<bool> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
fn init_log() {
    use log::Level;
    console_log::init_with_level(Level::Trace).expect("error initializing log");
}

#[cfg(not(target_arch = "wasm32"))]
fn init_log() {
    env_logger::init();
}  

fn first_startup() -> bool {
    init_log();
    log::debug!("Shards browser starting!");

    #[cfg(target_arch = "wasm32")]
    utils::set_panic_hook();

    log::debug!("Shards browser started!");

    return true;
}


#[wasm_bindgen]
pub fn start() {
    let res = START_SUCCESS.get_or_init(first_startup);
    log::debug!("Start result: {}", res);
}

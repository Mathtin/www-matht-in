mod log;
mod time;
mod utils;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use std::sync::OnceLock;

static START_SUCCESS: OnceLock<bool> = OnceLock::new();

fn first_startup() -> bool {
    log::init_log();
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

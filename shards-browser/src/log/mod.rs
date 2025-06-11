mod format;
#[cfg(target_arch = "wasm32")]
mod web_logger;
#[cfg(not(target_arch = "wasm32"))]
mod env_logger;

pub use log::*;

#[cfg(target_arch = "wasm32")]
pub use web_logger::init_log;

#[cfg(not(target_arch = "wasm32"))]
pub use env_logger::init_log;


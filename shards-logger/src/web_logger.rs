use super::format::{LOG_LEVEL, format_message};
use super::{Level, Log, Metadata, Record};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;


type RecordFormatter = &'static (dyn Fn(&Record) -> String + Send + Sync);

/// Logs messages to the Web browser's console
///
/// Error and warning messages will be logged with `console.error()` and `console.warn()`, respectively.
/// All other messages will be logged with `console.log()`.
struct ConsoleLogger {
    formatter: RecordFormatter,
    log_level: Level,
}

const DEFAULT_LOGGER: ConsoleLogger = ConsoleLogger {
    formatter: &format_message,
    log_level: LOG_LEVEL,
};

impl Default for ConsoleLogger {
    fn default() -> Self {
        DEFAULT_LOGGER
    }
}

impl Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.log_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = (self.formatter)(record);
            match record.level() {
                Level::Error => error(&msg),
                Level::Warn => warn(&msg),
                _ => log(&msg),
            }
        }
    }

    fn flush(&self) {}
}


// Bindings to console functions
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=console)]
    fn log(text: &str);

    #[wasm_bindgen(js_namespace=console)]
    fn warn(text: &str);

    #[wasm_bindgen(js_namespace=console)]
    fn error(text: &str);
}


pub fn init_log() {
    console_error_panic_hook::set_once();
    log::set_logger(&DEFAULT_LOGGER).expect("error initializing log");
    log::set_max_level(LOG_LEVEL.to_level_filter());
}

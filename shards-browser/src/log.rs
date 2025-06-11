pub use log::*;

use crate::time;

#[cfg(debug_assertions)]
const LOG_LEVEL: Level = Level::Debug;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: Level = Level::Info;

const DATE_FORMAT: &[time::BorrowedFormatItem<'_>] = time::format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
);

use wasm_bindgen::prelude::*;

const DEFAULT_LOGGER: ConsoleLogger = ConsoleLogger {
    formatter: &format_message,
    log_level: LOG_LEVEL,
};

fn format_message(record: &Record) -> String {
    let now = time::OffsetDateTime::now_utc()
        .format(&DATE_FORMAT)
        .expect("log format");
    if record.level() >= Level::Debug {
        format!("{} [{}]: {}", &now, record.level(), record.args())
    } else {
        format!("{}", record.args())
    }
}

type RecordFormatter = &'static (dyn Fn(&Record) -> String + Send + Sync);

/// Logs messages to the Web browser's console
///
/// Error and warning messages will be logged with `console.error()` and `console.warn()`, respectively.
/// All other messages will be logged with `console.log()`.
struct ConsoleLogger {
    formatter: RecordFormatter,
    log_level: Level,
}

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


#[cfg(target_arch = "wasm32")]
pub fn init_log() {
    log::set_logger(&DEFAULT_LOGGER).expect("error initializing log");
    log::set_max_level(LOG_LEVEL.to_level_filter());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init_log() {
    env_logger::builder()
        .format(format_message)
        .init();
}

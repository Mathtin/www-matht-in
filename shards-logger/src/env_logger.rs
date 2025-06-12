use super::format::{LOG_LEVEL, format_message};
use std::io::Write;

pub fn init_log() {
    env_logger::builder()
        .format(|buf, record| {
            writeln!(buf, "{}", format_message(record))
        })
        .filter_level(LOG_LEVEL.to_level_filter())
        .init();
}

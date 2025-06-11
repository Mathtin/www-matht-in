use crate::time;
use super::{Level, Record};

#[cfg(debug_assertions)]
pub const LOG_LEVEL: Level = Level::Debug;

#[cfg(not(debug_assertions))]
pub const LOG_LEVEL: Level = Level::Info;

const DATE_FORMAT: &[time::BorrowedFormatItem<'_>] = time::format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
);

pub fn format_message(record: &Record) -> String {
    let now = time::OffsetDateTime::now_utc()
        .format(&DATE_FORMAT)
        .expect("log format");
    if record.level() >= Level::Debug {
        format!("{} [{}]: {}", &now, record.level(), record.args())
    } else {
        format!("{}", record.args())
    }
}

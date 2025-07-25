use super::{Level, Record};
use time::{
    OffsetDateTime, format_description::BorrowedFormatItem,
    macros::format_description,
};

#[cfg(debug_assertions)]
pub const LOG_LEVEL: Level = Level::Debug;

#[cfg(not(debug_assertions))]
pub const LOG_LEVEL: Level = Level::Info;

const DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3][offset_hour sign:mandatory]"
);

pub fn format_message(record: &Record) -> String {
    let now = OffsetDateTime::now_local()
        .expect("can't get local time zone")
        .format(&DATE_FORMAT)
        .expect("log format");
    format!("{} [{}] {}", &now, record.level(), record.args())
}

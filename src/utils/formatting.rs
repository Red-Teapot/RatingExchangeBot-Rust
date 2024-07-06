use time::macros::format_description;
use time::{format_description, OffsetDateTime};

use super::{timestamp, TimestampStyle};

const DATETIME_FORMAT: &[format_description::FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]");

pub fn format_utc(date_time: impl Into<OffsetDateTime>) -> String {
    let offset_date_time: OffsetDateTime = date_time.into();
    offset_date_time
        .format(DATETIME_FORMAT)
        .expect("Hard-coded format should be correct")
}

pub fn format_local(date_time: impl Into<OffsetDateTime>) -> String {
    timestamp(date_time.into(), TimestampStyle::ShortDateTime)
}

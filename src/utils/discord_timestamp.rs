use std::fmt::Display;

use time::OffsetDateTime;

pub enum TimestampStyle {
    /// Short time, e.g. `16:20`
    ShortTime,
    /// Long time, e.g. `16:20:30`
    LongTime,
    /// Short date, e.g. `20/04/2021`
    ShortDate,
    /// Long date, e.g. `20 April 2021`
    LongDate,
    /// Short date/time, e.g. `20 April 2021 16:20`
    ShortDateTime,
    /// Long date/time, e.g. `Tuesday, 20 April 2021 16:20`
    LongDateTime,
    /// Relative time, e.g. `2 months ago`
    RelativeTime,
}

impl Display for TimestampStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.suffix())
    }
}

impl TimestampStyle {
    pub fn suffix(&self) -> &'static str {
        use TimestampStyle::*;

        match self {
            ShortTime => "t",
            LongTime => "T",
            ShortDate => "d",
            LongDate => "D",
            ShortDateTime => "f",
            LongDateTime => "F",
            RelativeTime => "R",
        }
    }
}

pub fn timestamp(datetime: OffsetDateTime, style: TimestampStyle) -> String {
    let unix_timestamp = datetime.unix_timestamp();
    format!("<t:{unix_timestamp}:{style}>")
}

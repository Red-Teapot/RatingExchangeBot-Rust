use sqlx::Type;
use std::ops::Add;
use time::{Duration, OffsetDateTime, PrimitiveDateTime, UtcOffset};

#[derive(Copy, Clone, Debug, Type)]
#[sqlx(transparent)]
pub struct UtcDateTime(PrimitiveDateTime);

impl UtcDateTime {
    pub fn assume_utc(datetime: PrimitiveDateTime) -> UtcDateTime {
        UtcDateTime(datetime)
    }
}

impl From<OffsetDateTime> for UtcDateTime {
    fn from(value: OffsetDateTime) -> Self {
        let value_utc = value.to_offset(UtcOffset::UTC);
        UtcDateTime(PrimitiveDateTime::new(value_utc.date(), value_utc.time()))
    }
}

impl From<UtcDateTime> for OffsetDateTime {
    fn from(value: UtcDateTime) -> Self {
        value.0.assume_utc()
    }
}

impl Add<Duration> for UtcDateTime {
    type Output = UtcDateTime;

    fn add(self, rhs: Duration) -> Self::Output {
        UtcDateTime(self.0 + rhs)
    }
}

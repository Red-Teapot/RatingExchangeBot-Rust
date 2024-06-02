use std::str::FromStr;

use lazy_regex::regex_captures;
use time::{Date, Duration, Month, OffsetDateTime, Time, UtcOffset};

use crate::commands::CommandError;

use super::super::user_err;

const EXAMPLE_1: &str = "2023-06-24 15:33:40 UTC+7";
const EXAMPLE_2: &str = "15:33 UTC";

fn invalid_argument(message: String) -> CommandError {
    user_err(&format!(
        "{message}\nDatetime examples: `{EXAMPLE_1}`, `{EXAMPLE_2}`."
    ))
}

#[derive(PartialEq, Eq, Debug)]
pub struct HumanDateTime {
    date: Option<Date>,
    time: Option<Time>,
    utc_offset: UtcOffset,
}

impl HumanDateTime {
    pub fn materialize(&self, mut base_date: OffsetDateTime) -> OffsetDateTime {
        base_date = base_date.to_offset(self.utc_offset);

        match (self.date, self.time) {
            (Some(date), Some(time)) => {
                base_date = base_date.replace_date(date).replace_time(time);
            }

            (Some(date), None) => {
                base_date = base_date.replace_date(date);
            }

            (None, Some(time)) => {
                if time <= base_date.time() {
                    base_date += Duration::days(1);
                }

                base_date = base_date.replace_time(time);
            }

            (None, None) => panic!("HumanDateTime must have either date or time"),
        }

        base_date
    }
}

impl FromStr for HumanDateTime {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut date = None;
        let mut time = None;
        let mut utc_offset = None;

        let tokens = s.split_whitespace().filter(|s| !s.is_empty());
        for token in tokens {
            if let Some((_, year, month, day)) =
                regex_captures!(r"^(\d{4})-(\d{2})-(\d{2})$", token)
            {
                if date.is_some() {
                    return Err(invalid_argument(format!("Duplicate date: `{token}`.")));
                }

                let year = year
                    .parse()
                    .map_err(|_| invalid_argument(format!("Invalid year: `{year}`.")))?;
                let month: u8 = month
                    .parse()
                    .map_err(|_| invalid_argument(format!("Invalid month: `{month}`.")))?;
                let day = day
                    .parse()
                    .map_err(|_| invalid_argument(format!("Invalid day: `{day}`.")))?;

                date = Some(
                    Date::from_calendar_date(
                        year,
                        Month::try_from(month)
                            .map_err(|_| invalid_argument(format!("Invalid month: `{month}`.")))?,
                        day,
                    )
                    .map_err(|_| invalid_argument(format!("Invalid date: `{token}`.")))?,
                );
            } else if let Some((_, hour, minute, _, second)) =
                regex_captures!(r"^(\d{2}):(\d{2})(:(\d{2}))?$", token)
            {
                if time.is_some() {
                    return Err(invalid_argument(format!("Duplicate time: `{token}`.")));
                }

                let hour = hour
                    .parse()
                    .map_err(|_| invalid_argument(format!("Invalid hour: `{hour}`.")))?;
                let minute = minute
                    .parse()
                    .map_err(|_| invalid_argument(format!("Invalid minute: `{minute}`.")))?;
                let second = if second.is_empty() {
                    0
                } else {
                    second
                        .parse()
                        .map_err(|_| invalid_argument(format!("Invalid second: `{second}`.")))?
                };

                time = Some(
                    Time::from_hms(hour, minute, second)
                        .map_err(|_| invalid_argument(format!("Invalid time: `{token}`.")))?,
                );
            } else if let Some((_, _, sign, hour, _, minute)) =
                regex_captures!(r"^UTC(([+-])(\d{1,2})(:(\d{2}))?)?$", token)
            {
                if utc_offset.is_some() {
                    return Err(invalid_argument(format!(
                        "Duplicate UTC offset: `{token}`."
                    )));
                }

                utc_offset =
                    if sign.is_empty() {
                        Some(UtcOffset::UTC)
                    } else {
                        let sign = if sign == "+" { 1 } else { -1 };

                        let hour: i8 = hour
                            .parse()
                            .map_err(|_| invalid_argument(format!("Invalid hour: `{hour}`.")))?;
                        let minute = if minute.is_empty() {
                            0
                        } else {
                            minute.parse().map_err(|_| {
                                invalid_argument(format!("Invalid minute: `{minute}`."))
                            })?
                        };

                        Some(UtcOffset::from_hms(hour * sign, minute, 0).map_err(|_| {
                            invalid_argument(format!("Invalid UTC offset: `{token}`."))
                        })?)
                    };
            } else {
                return Err(invalid_argument(format!("Invalid token: `{token}`.")));
            }
        }

        let utc_offset = match utc_offset {
            Some(offset) => offset,
            None => {
                return Err(invalid_argument("No UTC offset is provided.".to_string()));
            }
        };

        if let (None, None) = (date, time) {
            return Err(invalid_argument(
                "Neither date nor time is provided.".to_string(),
            ));
        }

        Ok(HumanDateTime {
            date,
            time,
            utc_offset,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use time::{
        macros::{date, datetime, offset, time},
        UtcOffset,
    };

    use crate::commands::arguments::{
        human_datetime::{EXAMPLE_1, EXAMPLE_2},
        HumanDateTime,
    };

    #[test]
    fn example_1() {
        assert_eq!(
            HumanDateTime::from_str(EXAMPLE_1).unwrap(),
            HumanDateTime {
                date: Some(date!(2023 - 06 - 24)),
                time: Some(time!(15:33:40)),
                utc_offset: offset!(+7:00),
            }
        );
    }

    #[test]
    fn example_2() {
        assert_eq!(
            HumanDateTime::from_str(EXAMPLE_2).unwrap(),
            HumanDateTime {
                date: None,
                time: Some(time!(15:33:00)),
                utc_offset: UtcOffset::UTC,
            }
        );
    }

    #[test]
    fn date_hms_offset() {
        assert_eq!(
            HumanDateTime::from_str("2023-02-15 14:37:22 UTC+7").unwrap(),
            HumanDateTime {
                date: Some(date!(2023 - 02 - 15)),
                time: Some(time!(14:37:22)),
                utc_offset: offset!(+7:00),
            }
        );
    }

    #[test]
    fn date_hm_offset() {
        assert_eq!(
            HumanDateTime::from_str("2023-02-15 14:37 UTC-2:30").unwrap(),
            HumanDateTime {
                date: Some(date!(2023 - 02 - 15)),
                time: Some(time!(14:37:00)),
                utc_offset: offset!(-2:30),
            }
        );
    }

    #[test]
    fn hms_offset() {
        assert_eq!(
            HumanDateTime::from_str("00:30:59 UTC+12").unwrap(),
            HumanDateTime {
                date: None,
                time: Some(time!(00:30:59)),
                utc_offset: offset!(+12:00),
            }
        );
    }

    #[test]
    fn hm_offset() {
        assert_eq!(
            HumanDateTime::from_str("00:59 UTC-10:30").unwrap(),
            HumanDateTime {
                date: None,
                time: Some(time!(00:59:00)),
                utc_offset: offset!(-10:30),
            }
        );
    }

    #[test]
    fn date_utc() {
        assert_eq!(
            HumanDateTime::from_str("1987-02-18 UTC").unwrap(),
            HumanDateTime {
                date: Some(date!(1987 - 02 - 18)),
                time: None,
                utc_offset: UtcOffset::UTC,
            }
        );
    }

    #[test]
    fn hms_utc() {
        assert_eq!(
            HumanDateTime::from_str("07:23:12 UTC").unwrap(),
            HumanDateTime {
                date: None,
                time: Some(time!(07:23:12)),
                utc_offset: UtcOffset::UTC,
            }
        );
    }

    #[test]
    fn only_offset() {
        assert!(HumanDateTime::from_str("UTC+2").is_err());
    }

    #[test]
    fn materialize_date_time_offset() {
        assert_eq!(
            HumanDateTime {
                date: Some(date!(2023 - 04 - 13)),
                time: Some(time!(18:06:30)),
                utc_offset: offset!(+7:45),
            }
            .materialize(datetime!(2022-12-12 07:59:30 -4)),
            datetime!(2023-04-13 18:06:30 +7:45)
        )
    }

    #[test]
    fn materialize_date_time_utc() {
        assert_eq!(
            HumanDateTime {
                date: Some(date!(2023 - 04 - 13)),
                time: Some(time!(18:06:30)),
                utc_offset: UtcOffset::UTC,
            }
            .materialize(datetime!(2022-12-12 07:59:30 UTC)),
            datetime!(2023-04-13 18:06:30 UTC)
        )
    }

    #[test]
    fn materialize_date_offset() {
        assert_eq!(
            HumanDateTime {
                date: Some(date!(2023 - 04 - 13)),
                time: None,
                utc_offset: offset!(+8),
            }
            .materialize(datetime!(2022-12-12 07:59:30 +3)),
            datetime!(2023-04-13 12:59:30 +8)
        )
    }

    #[test]
    fn materialize_date_utc() {
        assert_eq!(
            HumanDateTime {
                date: Some(date!(2023 - 04 - 13)),
                time: None,
                utc_offset: UtcOffset::UTC,
            }
            .materialize(datetime!(2022-12-12 07:59:30 UTC)),
            datetime!(2023-04-13 07:59:30 UTC)
        )
    }

    #[test]
    fn materialize_time_offset() {
        assert_eq!(
            HumanDateTime {
                date: None,
                time: Some(time!(02:00:23)),
                utc_offset: offset!(-10:30),
            }
            .materialize(datetime!(2023-04-13 07:59:30 UTC)),
            datetime!(2023-04-13 02:00:23 -10:30)
        )
    }

    #[test]
    fn materialize_time_utc() {
        assert_eq!(
            HumanDateTime {
                date: None,
                time: Some(time!(13:02:00)),
                utc_offset: UtcOffset::UTC,
            }
            .materialize(datetime!(2023-04-21 18:22:34 UTC)),
            datetime!(2023-04-22 13:02:00 UTC)
        )
    }
}

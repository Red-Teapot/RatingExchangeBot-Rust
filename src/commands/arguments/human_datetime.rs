use std::str::FromStr;

use lazy_regex::regex_captures;
use time::{Date, Month, Time, UtcOffset};

use crate::commands::CommandError;

const EXAMPLE_1: &str = "2023-06-24 15:33:40 UTC+7";
const EXAMPLE_2: &str = "15:33";

fn invalid_argument(message: String) -> CommandError {
    super::invalid_argument(format!(
        "{message}\nDatetime examples: `{EXAMPLE_1}`, `{EXAMPLE_2}`."
    ))
}

#[derive(PartialEq, Eq, Debug)]
pub struct HumanDateTime {
    date: Option<Date>,
    time: Option<Time>,
    utc_offset: Option<UtcOffset>,
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

    use time::{Date, Month, Time, UtcOffset};

    use crate::commands::arguments::{
        human_datetime::{EXAMPLE_1, EXAMPLE_2},
        HumanDateTime,
    };

    #[test]
    fn example_1() {
        assert_eq!(
            HumanDateTime::from_str(EXAMPLE_1).unwrap(),
            HumanDateTime {
                date: Some(Date::from_calendar_date(2023, Month::June, 24).unwrap()),
                time: Some(Time::from_hms(15, 33, 40).unwrap()),
                utc_offset: Some(UtcOffset::from_hms(7, 0, 0).unwrap()),
            }
        );
    }

    #[test]
    fn example_2() {
        assert_eq!(
            HumanDateTime::from_str(EXAMPLE_2).unwrap(),
            HumanDateTime {
                date: None,
                time: Some(Time::from_hms(15, 33, 0).unwrap()),
                utc_offset: None,
            }
        );
    }

    #[test]
    fn date_hms_offset() {
        assert_eq!(
            HumanDateTime::from_str("2023-02-15 14:37:22 UTC+7").unwrap(),
            HumanDateTime {
                date: Some(Date::from_calendar_date(2023, Month::February, 15).unwrap()),
                time: Some(Time::from_hms(14, 37, 22).unwrap()),
                utc_offset: Some(UtcOffset::from_hms(7, 0, 0).unwrap()),
            }
        );
    }

    #[test]
    fn date_hm_offset() {
        assert_eq!(
            HumanDateTime::from_str("2023-02-15 14:37 UTC-2:30").unwrap(),
            HumanDateTime {
                date: Some(Date::from_calendar_date(2023, Month::February, 15).unwrap()),
                time: Some(Time::from_hms(14, 37, 0).unwrap()),
                utc_offset: Some(UtcOffset::from_hms(-2, -30, 0).unwrap()),
            }
        );
    }

    #[test]
    fn date_hms() {
        assert_eq!(
            HumanDateTime::from_str("2005-12-31 00:59:40").unwrap(),
            HumanDateTime {
                date: Some(Date::from_calendar_date(2005, Month::December, 31).unwrap()),
                time: Some(Time::from_hms(0, 59, 40).unwrap()),
                utc_offset: None,
            }
        );
    }

    #[test]
    fn date_hm() {
        assert_eq!(
            HumanDateTime::from_str("2023-01-01 23:59").unwrap(),
            HumanDateTime {
                date: Some(Date::from_calendar_date(2023, Month::January, 1).unwrap()),
                time: Some(Time::from_hms(23, 59, 0).unwrap()),
                utc_offset: None,
            }
        );
    }

    #[test]
    fn hms_offset() {
        assert_eq!(
            HumanDateTime::from_str("00:30:59 UTC+12").unwrap(),
            HumanDateTime {
                date: None,
                time: Some(Time::from_hms(0, 30, 59).unwrap()),
                utc_offset: Some(UtcOffset::from_hms(12, 0, 0).unwrap()),
            }
        );
    }

    #[test]
    fn hm_offset() {
        assert_eq!(
            HumanDateTime::from_str("00:59 UTC-10:30").unwrap(),
            HumanDateTime {
                date: None,
                time: Some(Time::from_hms(0, 59, 0).unwrap()),
                utc_offset: Some(UtcOffset::from_hms(-10, 30, 0).unwrap()),
            }
        );
    }

    #[test]
    fn hms() {
        assert_eq!(
            HumanDateTime::from_str("12:32:47").unwrap(),
            HumanDateTime {
                date: None,
                time: Some(Time::from_hms(12, 32, 47).unwrap()),
                utc_offset: None,
            }
        );
    }

    #[test]
    fn hm() {
        assert_eq!(
            HumanDateTime::from_str("20:14").unwrap(),
            HumanDateTime {
                date: None,
                time: Some(Time::from_hms(20, 14, 0).unwrap()),
                utc_offset: None,
            }
        );
    }

    #[test]
    fn date_utc() {
        assert_eq!(
            HumanDateTime::from_str("1987-02-18 UTC").unwrap(),
            HumanDateTime {
                date: Some(Date::from_calendar_date(1987, Month::February, 18).unwrap()),
                time: None,
                utc_offset: Some(UtcOffset::UTC),
            }
        );
    }

    #[test]
    fn hms_utc() {
        assert_eq!(
            HumanDateTime::from_str("07:23:12 UTC").unwrap(),
            HumanDateTime {
                date: None,
                time: Some(Time::from_hms(7, 23, 12).unwrap()),
                utc_offset: Some(UtcOffset::UTC),
            }
        );
    }

    #[test]
    fn only_offset() {
        assert!(HumanDateTime::from_str("UTC+2").is_err());
    }
}

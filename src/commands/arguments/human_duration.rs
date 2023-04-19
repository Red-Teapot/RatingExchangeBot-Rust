use std::str::FromStr;

use time::Duration;

use crate::commands::CommandError;

const EXAMPLE_1: &str = "1 day 3 hours 2 minutes 59 seconds";
const EXAMPLE_2: &str = "1d 3h 2m 59s";

fn invalid_argument(message: String) -> CommandError {
    super::invalid_argument(format!(
        "{message}\nDuration examples: `{EXAMPLE_1}`, `{EXAMPLE_2}`."
    ))
}

pub struct HumanDuration(Duration);

impl From<HumanDuration> for Duration {
    fn from(value: HumanDuration) -> Self {
        value.0
    }
}

impl FromStr for HumanDuration {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.to_owned();

        if let Some(c) = s
            .chars()
            .find(|c| !(c.is_ascii_alphanumeric() || c.is_ascii_whitespace()))
        {
            return Err(invalid_argument(format!(
                "Invalid character in duration: `{}`.",
                c.escape_default()
            )));
        }

        s.make_ascii_lowercase();

        let mut tokens = s
            .split_ascii_whitespace()
            .filter(|s| !s.is_empty())
            .flat_map(|s| {
                if let Some((first_non_digit, _)) =
                    s.char_indices().find(|(_i, c)| !c.is_ascii_digit())
                {
                    if first_non_digit > 0 {
                        let prefix = &s[0..first_non_digit];
                        let suffix = &s[first_non_digit..];
                        vec![prefix, suffix]
                    } else {
                        vec![s]
                    }
                } else {
                    vec![s]
                }
            });

        let mut duration = Duration::ZERO;

        while let Some(count) = tokens.next() {
            let unit = tokens
                .next()
                .ok_or(invalid_argument("Unexpected end of duration.".to_string()))?;
            let count: u32 = count
                .parse()
                .map_err(|_| invalid_argument(format!("Expected a number, got `{count}`.")))?;

            match unit {
                unit if "days".starts_with(unit) => duration += Duration::days(count as _),

                unit if "hours".starts_with(unit) => duration += Duration::hours(count as _),

                unit if "minutes".starts_with(unit) => duration += Duration::minutes(count as _),

                unit if "seconds".starts_with(unit) => duration += Duration::seconds(count as _),

                unit => return Err(invalid_argument(format!("Unknown time unit: `{unit}`."))),
            }
        }

        Ok(HumanDuration(duration))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use time::Duration;

    use crate::commands::arguments::human_duration::{HumanDuration, EXAMPLE_1, EXAMPLE_2};

    #[test]
    fn simple() {
        assert_eq!(
            HumanDuration::from_str(" 1 day 3h 20 min 30s ").unwrap().0,
            Duration::days(1) + Duration::hours(3) + Duration::minutes(20) + Duration::seconds(30)
        );
    }

    #[test]
    fn example_1() {
        assert_eq!(
            HumanDuration::from_str(EXAMPLE_1).unwrap().0,
            Duration::days(1) + Duration::hours(3) + Duration::minutes(2) + Duration::seconds(59)
        );
    }

    #[test]
    fn example_2() {
        assert_eq!(
            HumanDuration::from_str(EXAMPLE_2).unwrap().0,
            Duration::days(1) + Duration::hours(3) + Duration::minutes(2) + Duration::seconds(59)
        );
    }
}

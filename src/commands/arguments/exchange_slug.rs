use std::{fmt::Display, str::FromStr};

use crate::commands::CommandError;

use super::super::user_err;

pub struct ExchangeSlug(String);

impl FromStr for ExchangeSlug {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let is_valid = s
            .chars()
            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_'));

        if is_valid {
            Ok(ExchangeSlug(s.to_string()))
        } else {
            Err(user_err(&format!("Invalid exchange slug: `{}`.\nIt can only contain a-z, A-Z, 0-9, a dash (-) or an underscore (_).", s.escape_default())))
        }
    }
}

impl From<String> for ExchangeSlug {
    fn from(value: String) -> Self {
        ExchangeSlug::from_str(&value).unwrap()
    }
}

impl From<&str> for ExchangeSlug {
    fn from(value: &str) -> Self {
        ExchangeSlug::from_str(value).unwrap()
    }
}

impl From<ExchangeSlug> for String {
    fn from(value: ExchangeSlug) -> Self {
        value.0
    }
}

impl Display for ExchangeSlug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for ExchangeSlug {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::commands::arguments::exchange_slug::ExchangeSlug;

    #[test]
    fn simple() {
        assert!(ExchangeSlug::from_str("SomeTest_1-2").is_ok());
    }

    #[test]
    fn alphabet_caps() {
        assert!(ExchangeSlug::from_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ").is_ok());
    }

    #[test]
    fn alphabet_small() {
        assert!(ExchangeSlug::from_str("abcdefghijklmnopqrstuvwxyz").is_ok());
    }

    #[test]
    fn digits() {
        assert!(ExchangeSlug::from_str("0123456789").is_ok());
    }

    #[test]
    fn space_before_after() {
        assert!(ExchangeSlug::from_str(" AlmostValidButContainsSpaces   ").is_ok());
    }

    #[test]
    fn space_in_middle() {
        assert!(ExchangeSlug::from_str("Almost ValidBut ContainsSpaces").is_err());
    }

    #[test]
    fn special_char() {
        assert!(ExchangeSlug::from_str("Foo!Bar").is_err());
    }
}

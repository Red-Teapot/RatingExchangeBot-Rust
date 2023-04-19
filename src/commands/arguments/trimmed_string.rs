use std::{convert::Infallible, fmt::Display, str::FromStr};

/// A string that has no leading or trailing whitespaces.
///
/// Implemented `From*` traits trim the strings.
pub struct TrimmedString(String);

impl FromStr for TrimmedString {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TrimmedString(s.trim().to_owned()))
    }
}

impl From<String> for TrimmedString {
    fn from(value: String) -> Self {
        TrimmedString::from_str(&value).unwrap()
    }
}

impl From<&str> for TrimmedString {
    fn from(value: &str) -> Self {
        TrimmedString::from_str(value).unwrap()
    }
}

impl From<TrimmedString> for String {
    fn from(value: TrimmedString) -> Self {
        value.0
    }
}

impl Display for TrimmedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for TrimmedString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::arguments::trimmed_string::TrimmedString;

    #[test]
    fn trimmed() {
        assert_eq!(TrimmedString::from("test foo bar").as_ref(), "test foo bar");
    }

    #[test]
    fn untrimmed() {
        assert_eq!(
            TrimmedString::from("  test foo  \t bar   ").as_ref(),
            "test foo  \t bar"
        );
    }
}

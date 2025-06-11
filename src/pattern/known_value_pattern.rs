use known_values::KnownValue;

use crate::Envelope;
use super::match_pattern::MatchPattern;

/// Pattern for matching known values.
#[derive(Debug, Clone)]
pub enum KnownValuePattern {
    /// Matches any known value.
    Any,
    /// Matches the specific known value.
    KnownValue(KnownValue),
    /// Matches the specific known value name.
    Name(String),
    /// Matches the regex for a known value name.
    NameRegex(regex::Regex),
    /// Matches the Unit known value.
    Unit,
}

impl MatchPattern for KnownValuePattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        if let Ok(value) = envelope.extract_subject::<KnownValue>() {
            match self {
                KnownValuePattern::Any => true,
                KnownValuePattern::KnownValue(expected) => value == *expected,
                KnownValuePattern::Name(name) => value.to_string() == *name,
                KnownValuePattern::NameRegex(regex) => regex.is_match(&value.to_string()),
                KnownValuePattern::Unit => value.to_string() == "unit",
            }
        } else {
            false
        }
    }
}

use std::ops::RangeInclusive;

use dcbor::{Date, prelude::*};

use crate::{
    Envelope, Pattern,
    pattern::{
        Compilable, Matcher, Path, compile_as_atomic, leaf::LeafPattern,
        vm::Instr,
    },
};

/// Pattern for matching dates.
#[derive(Debug, Clone)]
pub enum DatePattern {
    /// Matches any date.
    Any,
    /// Matches a specific date.
    Date(Date),
    /// Matches dates within a range (inclusive).
    Range(RangeInclusive<Date>),
    /// Matches dates that are on or after the specified date.
    Earliest(Date),
    /// Matches dates that are on or before the specified date.
    Latest(Date),
    /// Matches a date by its ISO-8601 string representation.
    Iso8601(String),
    /// Matches dates whose ISO-8601 string representation matches the given
    /// regex pattern.
    Regex(regex::Regex),
}

impl PartialEq for DatePattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DatePattern::Any, DatePattern::Any) => true,
            (DatePattern::Date(a), DatePattern::Date(b)) => a == b,
            (DatePattern::Range(a), DatePattern::Range(b)) => a == b,
            (DatePattern::Earliest(a), DatePattern::Earliest(b)) => a == b,
            (DatePattern::Latest(a), DatePattern::Latest(b)) => a == b,
            (DatePattern::Iso8601(a), DatePattern::Iso8601(b)) => a == b,
            (DatePattern::Regex(a), DatePattern::Regex(b)) => {
                a.as_str() == b.as_str()
            }
            _ => false,
        }
    }
}

impl Eq for DatePattern {}

impl std::hash::Hash for DatePattern {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DatePattern::Any => {
                0u8.hash(state);
            }
            DatePattern::Date(date) => {
                1u8.hash(state);
                date.hash(state);
            }
            DatePattern::Range(range) => {
                2u8.hash(state);
                range.start().hash(state);
                range.end().hash(state);
            }
            DatePattern::Earliest(date) => {
                3u8.hash(state);
                date.hash(state);
            }
            DatePattern::Latest(date) => {
                4u8.hash(state);
                date.hash(state);
            }
            DatePattern::Iso8601(iso_string) => {
                5u8.hash(state);
                iso_string.hash(state);
            }
            DatePattern::Regex(regex) => {
                6u8.hash(state);
                // Regex does not implement Hash, so we hash its pattern string.
                regex.as_str().hash(state);
            }
        }
    }
}

impl DatePattern {
    /// Creates a new `DatePattern` that matches any date.
    pub fn any() -> Self { DatePattern::Any }

    /// Creates a new `DatePattern` that matches a specific date.
    pub fn date(date: Date) -> Self { DatePattern::Date(date) }

    /// Creates a new `DatePattern` that matches dates within a range
    /// (inclusive).
    pub fn range(range: RangeInclusive<Date>) -> Self {
        DatePattern::Range(range)
    }

    /// Creates a new `DatePattern` that matches dates that are on or after the
    /// specified date.
    pub fn earliest(date: Date) -> Self { DatePattern::Earliest(date) }

    /// Creates a new `DatePattern` that matches dates that are on or before the
    /// specified date.
    pub fn latest(date: Date) -> Self { DatePattern::Latest(date) }

    /// Creates a new `DatePattern` that matches a date by its ISO-8601 string
    /// representation.
    pub fn iso8601(iso_string: impl Into<String>) -> Self {
        DatePattern::Iso8601(iso_string.into())
    }

    /// Creates a new `DatePattern` that matches dates whose ISO-8601 string
    /// representation matches the given regex pattern.
    pub fn regex(regex: regex::Regex) -> Self { DatePattern::Regex(regex) }
}

impl Matcher for DatePattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        // Check if the envelope subject contains a date (CBOR tag 1)
        if let Some(cbor) = envelope.subject().as_leaf() {
            if let CBORCase::Tagged(tag, _) = cbor.as_case() {
                // Check if this is a date tag (tag 1)
                if tag.value() == 1 {
                    // Try to extract the date
                    if let Ok(date) = Date::try_from(cbor.clone()) {
                        match self {
                            DatePattern::Any => vec![vec![envelope.clone()]],
                            DatePattern::Date(expected_date) => {
                                if date == *expected_date {
                                    vec![vec![envelope.clone()]]
                                } else {
                                    vec![]
                                }
                            }
                            DatePattern::Range(range) => {
                                if range.contains(&date) {
                                    vec![vec![envelope.clone()]]
                                } else {
                                    vec![]
                                }
                            }
                            DatePattern::Earliest(earliest) => {
                                if date >= *earliest {
                                    vec![vec![envelope.clone()]]
                                } else {
                                    vec![]
                                }
                            }
                            DatePattern::Latest(latest) => {
                                if date <= *latest {
                                    vec![vec![envelope.clone()]]
                                } else {
                                    vec![]
                                }
                            }
                            DatePattern::Iso8601(expected_string) => {
                                let date_string = date.to_string();
                                if date_string == *expected_string {
                                    vec![vec![envelope.clone()]]
                                } else {
                                    vec![]
                                }
                            }
                            DatePattern::Regex(regex) => {
                                let date_string = date.to_string();
                                if regex.is_match(&date_string) {
                                    vec![vec![envelope.clone()]]
                                } else {
                                    vec![]
                                }
                            }
                        }
                    } else {
                        // Tagged with date tag but couldn't be parsed as date
                        vec![]
                    }
                } else {
                    // Not a date tag
                    vec![]
                }
            } else {
                // Not tagged
                vec![]
            }
        } else {
            // Not a leaf CBOR value
            vec![]
        }
    }
}

impl Compilable for DatePattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        compile_as_atomic(
            &Pattern::Leaf(LeafPattern::Date(self.clone())),
            code,
            literals,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bc_envelope::Envelope;

    #[test]
    fn test_date_pattern_any() {
        // Create a date envelope
        let date = Date::from_ymd(2023, 12, 25);
        let envelope = Envelope::new(date.clone());

        let pattern = DatePattern::any();
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with non-date envelope
        let text_envelope = Envelope::new("test");
        let paths = pattern.paths(&text_envelope);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_date_pattern_specific() {
        // Create a date envelope
        let date = Date::from_ymd(2023, 12, 25);
        let envelope = Envelope::new(date.clone());

        // Test matching date
        let pattern = DatePattern::date(date.clone());
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test non-matching date
        let different_date = Date::from_ymd(2023, 12, 24);
        let pattern = DatePattern::date(different_date);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_date_pattern_range() {
        let date = Date::from_ymd(2023, 12, 25);
        let envelope = Envelope::new(date.clone());

        // Test date within range
        let start = Date::from_ymd(2023, 12, 20);
        let end = Date::from_ymd(2023, 12, 30);
        let pattern = DatePattern::range(start..=end);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test date outside range
        let start = Date::from_ymd(2023, 12, 26);
        let end = Date::from_ymd(2023, 12, 30);
        let pattern = DatePattern::range(start..=end);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());

        // Test range boundaries (inclusive)
        let start = Date::from_ymd(2023, 12, 25);
        let end = Date::from_ymd(2023, 12, 25);
        let pattern = DatePattern::range(start..=end);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn test_date_pattern_iso8601() {
        // Test date-only string (midnight)
        let date = Date::from_ymd(2023, 12, 25);
        let envelope = Envelope::new(date.clone());

        let pattern = DatePattern::iso8601("2023-12-25");
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test non-matching string
        let pattern = DatePattern::iso8601("2023-12-24");
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());

        // Test date with time
        let date_with_time = Date::from_ymd_hms(2023, 12, 25, 15, 30, 45);
        let envelope_with_time = Envelope::new(date_with_time.clone());

        let pattern = DatePattern::iso8601("2023-12-25T15:30:45Z");
        let paths = pattern.paths(&envelope_with_time);
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn test_date_pattern_regex() {
        let date = Date::from_ymd(2023, 12, 25);
        let envelope = Envelope::new(date.clone());

        // Test regex that matches year 2023
        let regex = regex::Regex::new(r"^2023-.*").unwrap();
        let pattern = DatePattern::regex(regex);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test regex that matches December
        let regex = regex::Regex::new(r".*-12-.*").unwrap();
        let pattern = DatePattern::regex(regex);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test regex that doesn't match
        let regex = regex::Regex::new(r"^2024-.*").unwrap();
        let pattern = DatePattern::regex(regex);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());

        // Test regex with date-time
        let date_with_time = Date::from_ymd_hms(2023, 12, 25, 15, 30, 45);
        let envelope_with_time = Envelope::new(date_with_time.clone());

        let regex = regex::Regex::new(r".*T15:30:45Z$").unwrap();
        let pattern = DatePattern::regex(regex);
        let paths = pattern.paths(&envelope_with_time);
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn test_date_pattern_with_non_date_tagged_cbor() {
        // Create a non-date tagged CBOR value (e.g., tag 100)
        let tagged_cbor = CBOR::to_tagged_value(100, "not a date");
        let envelope = Envelope::new(tagged_cbor);

        // Should not match any date pattern
        let pattern = DatePattern::any();
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());
    }
}

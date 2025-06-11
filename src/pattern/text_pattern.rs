use super::{Matcher, Path};
use crate::Envelope;

/// Pattern for matching text values.
#[derive(Debug, Clone)]
pub enum TextPattern {
    /// Matches any text.
    Any,
    /// Matches the specific text.
    Exact(String),
    /// Matches the regex for a text.
    Regex(regex::Regex),
}

impl TextPattern {
    /// Creates a new `TextPattern` that matches any text.
    pub fn any() -> Self { TextPattern::Any }

    /// Creates a new `TextPattern` that matches the specific text.
    pub fn exact<T: Into<String>>(value: T) -> Self {
        TextPattern::Exact(value.into())
    }

    /// Creates a new `TextPattern` that matches the regex for a text.
    pub fn regex(regex: regex::Regex) -> Self { TextPattern::Regex(regex) }
}

impl Matcher for TextPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let is_hit =
            envelope
                .extract_subject::<String>()
                .ok()
                .map_or(false, |value| match self {
                    TextPattern::Any => true,
                    TextPattern::Exact(want) => value == *want,
                    TextPattern::Regex(regex) => regex.is_match(&value),
                });

        if is_hit {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}

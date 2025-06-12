// filepath: /Users/wolf/Dropbox/DevProjects/BlockchainCommons/bc-rust/bc-envelope/src/pattern/tag_pattern.rs
use dcbor::prelude::*;

use crate::Envelope;
use super::matcher::Matcher;

/// Pattern for matching tag values.
#[derive(Debug, Clone)]
pub enum TagPattern {
    /// Matches any tag.
    Any,
    /// Matches the specific tag.
    Tag(Tag),
    /// Matches the specific tag name.
    Name(String),
    /// Matches the regex for a tag.
    NameRegex(regex::Regex),
}

impl MatchPattern for TagPattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        // Check if the envelope contains a tagged value
        if let Some(CBORCase::Tagged(tag, _)) = envelope.subject().case() {
            match self {
                TagPattern::Any => true,
                TagPattern::Tag(expected_tag) => expected_tag.value() == tag.value(),
                TagPattern::Name(name) => format!("{}", tag) == *name,
                TagPattern::NameRegex(regex) => regex.is_match(&format!("{}", tag)),
            }
        } else {
            false
        }
    }
}

use dcbor::prelude::*;

use crate::Envelope;
use super::match_pattern::MatchPattern;

/// Pattern for matching byte string values.
#[derive(Debug, Clone)]
pub enum ByteStringPattern {
    /// Matches any byte string.
    Any,
    /// Matches the specific byte string.
    Exact(Vec<u8>),
    /// Matches the regex for a byte string.
    Regex(regex::bytes::Regex),
}

impl MatchPattern for ByteStringPattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        if let Ok(bytes) = envelope.extract_subject::<ByteString>() {
            match self {
                ByteStringPattern::Any => true,
                ByteStringPattern::Exact(value) => value == bytes.as_ref(),
                ByteStringPattern::Regex(regex) => regex.is_match(&bytes),
            }
        } else {
            false
        }
    }
}

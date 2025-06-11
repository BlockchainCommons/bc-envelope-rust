use bc_components::{Digest, DigestProvider};
use crate::Envelope;
use super::matcher::Matcher;

/// Pattern for matching envelopes by their digest.
#[derive(Debug, Clone)]
pub enum DigestPattern {
    /// Matches the exact digest.
    Digest(Digest),
    /// Matches the hexadecimal prefix of a digest.
    HexPrefix(String),
    /// Matches the binary regular expression for a digest.
    BinaryRegex(regex::bytes::Regex),
}

impl MatchPattern for DigestPattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        let digest = envelope.digest();
        match self {
            DigestPattern::Digest(pattern_digest) => *pattern_digest == *digest,
            DigestPattern::HexPrefix(prefix) => {
                hex::encode(digest.as_ref()).starts_with(prefix)
            }
            DigestPattern::BinaryRegex(regex) => regex.is_match(digest.as_bytes()),
        }
    }
}

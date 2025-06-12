use bc_components::{Digest, DigestProvider};

use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for matching envelopes by their digest.
#[derive(Debug, Clone)]
pub enum DigestPattern {
    /// Matches the exact digest.
    Digest(Digest),
    /// Matches the hexadecimal prefix of a digest (case insensitive).
    HexPrefix(String),
    /// Matches the binary regular expression for a digest.
    BinaryRegex(regex::bytes::Regex),
}

impl DigestPattern {
    /// Creates a new `DigestPattern` that matches the exact digest.
    pub fn digest(digest: Digest) -> Self { DigestPattern::Digest(digest) }

    /// Creates a new `DigestPattern` that matches the hexadecimal prefix of a
    /// digest.
    pub fn hex_prefix(prefix: impl Into<String>) -> Self {
        DigestPattern::HexPrefix(prefix.into())
    }

    /// Creates a new `DigestPattern` that matches the binary regex for a
    /// digest.
    pub fn binary_regex(regex: regex::bytes::Regex) -> Self {
        DigestPattern::BinaryRegex(regex)
    }
}

impl Matcher for DigestPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let digest = envelope.digest();
        let is_hit = match self {
            DigestPattern::Digest(pattern_digest) => *pattern_digest == *digest,
            DigestPattern::HexPrefix(prefix) => hex::encode(digest.as_bytes())
                .to_lowercase()
                .starts_with(&prefix.to_lowercase()),
            DigestPattern::BinaryRegex(regex) => {
                regex.is_match(digest.as_bytes())
            }
        };

        if is_hit {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}

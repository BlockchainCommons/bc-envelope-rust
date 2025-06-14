use bc_components::{Digest, DigestProvider};

use crate::{
    Envelope, Pattern,
    pattern::{
        Compilable, Matcher, Path, compile_as_atomic,
        structure::StructurePattern, vm::Instr,
    },
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

impl PartialEq for DigestPattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DigestPattern::Digest(a), DigestPattern::Digest(b)) => a == b,
            (DigestPattern::HexPrefix(a), DigestPattern::HexPrefix(b)) => {
                a.eq_ignore_ascii_case(b)
            }
            (DigestPattern::BinaryRegex(a), DigestPattern::BinaryRegex(b)) => {
                a.as_str() == b.as_str()
            }
            _ => false,
        }
    }
}

impl Eq for DigestPattern {}

impl std::hash::Hash for DigestPattern {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DigestPattern::Digest(a) => {
                0u8.hash(state);
                a.hash(state);
            }
            DigestPattern::HexPrefix(prefix) => {
                1u8.hash(state);
                prefix.to_lowercase().hash(state);
            }
            DigestPattern::BinaryRegex(regex) => {
                2u8.hash(state);
                // Regex does not implement Hash, so we hash its pattern string.
                regex.as_str().hash(state);
            }
        }
    }
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
            DigestPattern::HexPrefix(prefix) => {
                hex::encode(digest.data()).starts_with(&prefix.to_lowercase())
            }
            DigestPattern::BinaryRegex(regex) => regex.is_match(digest.data()),
        };

        if is_hit {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}

impl Compilable for DigestPattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        compile_as_atomic(
            &Pattern::Structure(StructurePattern::Digest(self.clone())),
            code,
            literals,
        );
    }
}

use super::{AndPattern, NotPattern, OrPattern, SearchPattern, SequencePattern};
use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for combining and modifying other patterns.
#[derive(Debug, Clone)]
pub enum MetaPattern {
    /// Matches if all contained patterns match.
    And(AndPattern),
    /// Matches if any contained pattern matches.
    Or(OrPattern),
    /// Matches if the inner pattern does not match.
    Not(NotPattern),
    /// Searches the entire envelope tree for matches.
    Search(SearchPattern),
    /// Matches a sequence of patterns.
    Sequence(SequencePattern),
    // Matches with repetition.
    // Repeat(RepeatPattern),
}

impl MetaPattern {
    pub fn and(pattern: AndPattern) -> Self { MetaPattern::And(pattern) }

    pub fn or(pattern: OrPattern) -> Self { MetaPattern::Or(pattern) }

    pub fn search(pattern: SearchPattern) -> Self {
        MetaPattern::Search(pattern)
    }

    pub fn sequence(pattern: SequencePattern) -> Self {
        MetaPattern::Sequence(pattern)
    }

    pub fn not(pattern: NotPattern) -> Self {
        MetaPattern::Not(pattern)
    }
}

impl Matcher for MetaPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        match self {
            MetaPattern::And(pattern) => pattern.paths(envelope),
            MetaPattern::Or(pattern) => pattern.paths(envelope),
            MetaPattern::Not(pattern) => pattern.paths(envelope),
            MetaPattern::Search(pattern) => pattern.paths(envelope),
            MetaPattern::Sequence(pattern) => pattern.paths(envelope),
        }
    }
}

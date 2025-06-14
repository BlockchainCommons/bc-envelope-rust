use super::{
    AndPattern, CapturePattern, NotPattern, OrPattern, RepeatPattern,
    SearchPattern, SequencePattern,
};
use crate::{
    Envelope, Pattern,
    pattern::{Compilable, Matcher, Path, vm::Instr},
};

/// Pattern for combining and modifying other patterns.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
    /// Matches with repetition.
    Repeat(RepeatPattern),
    /// Captures a pattern match.
    Capture(CapturePattern),
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

    pub fn not(pattern: NotPattern) -> Self { MetaPattern::Not(pattern) }
}

impl Matcher for MetaPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        match self {
            MetaPattern::And(pattern) => pattern.paths(envelope),
            MetaPattern::Or(pattern) => pattern.paths(envelope),
            MetaPattern::Not(pattern) => pattern.paths(envelope),
            MetaPattern::Search(pattern) => pattern.paths(envelope),
            MetaPattern::Sequence(pattern) => pattern.paths(envelope),
            MetaPattern::Repeat(pattern) => pattern.paths(envelope),
            MetaPattern::Capture(pattern) => pattern.paths(envelope),
        }
    }
}

impl Compilable for MetaPattern {
    fn compile(&self, code: &mut Vec<Instr>, lits: &mut Vec<Pattern>) {
        match self {
            MetaPattern::And(pattern) => pattern.compile(code, lits),
            MetaPattern::Or(pattern) => pattern.compile(code, lits),
            MetaPattern::Not(pattern) => pattern.compile(code, lits),
            MetaPattern::Search(pattern) => pattern.compile(code, lits),
            MetaPattern::Sequence(pattern) => pattern.compile(code, lits),
            MetaPattern::Repeat(pattern) => pattern.compile(code, lits),
            MetaPattern::Capture(pattern) => pattern.compile(code, lits),
        }
    }
}

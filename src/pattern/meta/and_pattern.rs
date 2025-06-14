use crate::{
    Envelope,
    pattern::{Matcher, Path, Pattern},
};

/// A pattern that matches if all contained patterns match.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AndPattern {
    pub patterns: Vec<Pattern>,
}

impl AndPattern {
    /// Creates a new `AndPattern` with the given patterns.
    pub fn new(patterns: Vec<Pattern>) -> Self { AndPattern { patterns } }

    /// Compile into byte-code (AND = all must match).
    pub fn compile(
        &self,
        code: &mut Vec<crate::pattern::vm::Instr>,
        lits: &mut Vec<Pattern>,
    ) {
        // Each pattern must match at this position
        for pattern in &self.patterns {
            pattern.compile(code, lits);
        }
    }
}

impl Matcher for AndPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if self
            .patterns
            .iter()
            .all(|pattern| pattern.matches(envelope))
        {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}

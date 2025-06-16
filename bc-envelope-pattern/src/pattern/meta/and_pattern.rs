use crate::{
    Envelope,
    pattern::{Compilable, Matcher, Path, Pattern, vm::Instr},
};

/// A pattern that matches if all contained patterns match.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AndPattern {
    pub patterns: Vec<Pattern>,
}

impl AndPattern {
    /// Creates a new `AndPattern` with the given patterns.
    pub fn new(patterns: Vec<Pattern>) -> Self { AndPattern { patterns } }
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

impl Compilable for AndPattern {
    /// Compile into byte-code (AND = all must match).
    fn compile(&self, code: &mut Vec<Instr>, lits: &mut Vec<Pattern>) {
        // Each pattern must match at this position
        for pattern in &self.patterns {
            pattern.compile(code, lits);
        }
    }
}

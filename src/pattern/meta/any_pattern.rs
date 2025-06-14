use crate::{
    pattern::{compile_as_atomic, meta::MetaPattern, vm::Instr, Compilable, Matcher, Path, Pattern}, Envelope
};

/// A pattern that matches if any contained pattern matches.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AnyPattern;

impl AnyPattern {
    /// Creates a new `AnyPattern`.
    pub fn new() -> Self {
        AnyPattern
    }
}

impl Default for AnyPattern {
    fn default() -> Self {
        AnyPattern
    }
}

impl Matcher for AnyPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        // Always return a path containing the envelope itself.
        vec![vec![envelope.clone()]]
    }
}

impl Compilable for AnyPattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        compile_as_atomic(
            &Pattern::Meta(MetaPattern::Any(self.clone())),
            code,
            literals,
        );
    }
}

use crate::{
    pattern::{compile_as_atomic, meta::MetaPattern, vm::Instr, Compilable, Matcher, Path, Pattern}, Envelope
};

/// A pattern that matches if any contained pattern matches.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct NonePattern;

impl NonePattern {
    /// Creates a new `NonePattern`.
    pub fn new() -> Self {
        NonePattern
    }
}

impl Default for NonePattern {
    fn default() -> Self {
        NonePattern
    }
}

impl Matcher for NonePattern {
    fn paths(&self, _envelope: &Envelope) -> Vec<Path> {
        // Never matches any element.
        Vec::new()
    }
}

impl Compilable for NonePattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        compile_as_atomic(
            &Pattern::Meta(MetaPattern::None(self.clone())),
            code,
            literals,
        );
    }
}

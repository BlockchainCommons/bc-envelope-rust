use crate::{
    pattern::{compile_as_atomic, leaf::LeafPattern, vm::Instr, Compilable, Matcher, Path}, Envelope, Pattern
};

/// Pattern for matching boolean values.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum BoolPattern {
    /// Matches any boolean value.
    Any,
    /// Matches the specific boolean value.
    Exact(bool),
}

impl BoolPattern {
    /// Creates a new `BoolPattern` that matches any boolean value.
    pub fn any() -> Self { BoolPattern::Any }

    /// Creates a new `BoolPattern` that matches the specific boolean value.
    pub fn exact(value: bool) -> Self { BoolPattern::Exact(value) }
}

impl Matcher for BoolPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let is_hit =
            envelope
                .extract_subject::<bool>()
                .ok()
                .is_some_and(|value| match self {
                    BoolPattern::Any => true,
                    BoolPattern::Exact(want) => value == *want,
                });

        if is_hit {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}

impl Compilable for BoolPattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        compile_as_atomic(
            &Pattern::Leaf(LeafPattern::Bool(
                self.clone(),
            )),
            code,
            literals,
        );
    }
}

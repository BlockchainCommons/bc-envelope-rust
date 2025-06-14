use crate::{
    Envelope, Pattern,
    pattern::{
        Matcher, Path,
        structure::StructurePattern,
        vm::{Axis, Instr},
    },
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum WrappedPattern {
    /// Matches any wrapped envelope.
    Any,
    /// Matches a wrapped envelope, and adds the unwrapped envelope to the path.
    Unwrap,
}

impl WrappedPattern {
    /// Creates a new `WrappedPattern` that matches any wrapped envelope.
    pub fn any() -> Self { WrappedPattern::Any }

    /// Creates a new `WrappedPattern` that matches a wrapped envelope and
    /// unwraps it.
    pub fn unwrap() -> Self { WrappedPattern::Unwrap }
}

impl WrappedPattern {
    /// Emit predicate + descent so the VM makes progress.
    pub(crate) fn compile(
        &self,
        code: &mut Vec<Instr>,
        lits: &mut Vec<Pattern>,
    ) {
        // 1) atomic predicate “is wrapped”
        let idx = lits.len();
        lits.push(Pattern::Structure(StructurePattern::wrapped(self.clone())));
        code.push(Instr::MatchPredicate(idx));

        // 2) then move into inner envelope
        code.push(Instr::PushAxis(Axis::Wrapped));
    }
}

impl Matcher for WrappedPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if envelope.subject().is_wrapped() {
            match self {
                WrappedPattern::Any => {
                    vec![vec![envelope.clone()]]
                }
                WrappedPattern::Unwrap => {
                    if let Ok(unwrapped) = envelope.subject().unwrap_envelope()
                    {
                        vec![vec![unwrapped]]
                    } else {
                        vec![]
                    }
                }
            }
        } else {
            vec![]
        }
    }
}

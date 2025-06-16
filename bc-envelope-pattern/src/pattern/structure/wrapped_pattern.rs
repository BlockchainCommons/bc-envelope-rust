use crate::{
    Envelope, Pattern,
    pattern::{
        Compilable, Matcher, Path,
        structure::StructurePattern,
        vm::{Axis, Instr},
    },
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum WrappedPattern {
    /// Matches any wrapped envelope.
    Any,
}

impl WrappedPattern {
    /// Creates a new `WrappedPattern` that matches any wrapped envelope.
    pub fn any() -> Self { WrappedPattern::Any }
}

impl Compilable for WrappedPattern {
    /// Emit predicate + descent so the VM makes progress.
    fn compile(&self, code: &mut Vec<Instr>, lits: &mut Vec<Pattern>) {
        // println!("Compiling WrappedPattern: {:?}", self);
        // 1) atomic predicate “is wrapped”
        let idx = lits.len();
        lits.push(Pattern::Structure(StructurePattern::wrapped(self.clone())));
        code.push(Instr::MatchStructure(idx));

        // 2) then move into inner envelope
        code.push(Instr::PushAxis(Axis::Wrapped));
    }
}

impl Matcher for WrappedPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        // println!("Matching WrappedPattern: {:?}", self);
        let subject = envelope.subject();
        if subject.is_wrapped() {
            match self {
                WrappedPattern::Any => {
                    vec![vec![envelope.clone()]]
                }
            }
        } else {
            vec![]
        }
    }
}

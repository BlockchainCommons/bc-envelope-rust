use crate::{
    Envelope,
    pattern::{Matcher, Path, vm::Instr, Pattern},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SubjectPattern {
    Any,
}

impl SubjectPattern {
    pub fn any() -> Self { SubjectPattern::Any }

    pub fn compile(&self, code: &mut Vec<Instr>, _lits: &mut Vec<Pattern>) {
        match self {
            SubjectPattern::Any => {
                code.push(Instr::NavigateSubject);
            }
        }
    }
}

impl Matcher for SubjectPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let subject = envelope.subject();
        match self {
            SubjectPattern::Any => {
                vec![vec![subject.clone()]]
            }
        }
    }
}

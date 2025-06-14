use crate::{
    Envelope,
    pattern::{Compilable, Matcher, Path, Pattern, vm::Instr},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SubjectPattern {
    Any,
}

impl SubjectPattern {
    pub fn any() -> Self { SubjectPattern::Any }
}

impl Compilable for SubjectPattern {
    fn compile(&self, code: &mut Vec<Instr>, _lits: &mut Vec<Pattern>) {
        match self {
            SubjectPattern::Any => {
                code.push(Instr::NavigateSubject);
            }
        }
    }
}

impl Matcher for SubjectPattern {
    fn paths(&self, env: &Envelope) -> Vec<Path> {
        // `Envelope::subject()` always returns an `Envelope`.
        // If `env` is already that subject (i.e. not a NODE) the caller
        // gets back an empty path, otherwise it gets the real subject.
        let subj = env.subject();
        if &subj == env {
            vec![vec![]]           // identity case: no additional envelope
        } else {
            vec![vec![subj]]       // normal case: yield the subject envelope
        }
    }
}

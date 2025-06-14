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
        if env.is_node() {
            vec![vec![env.subject().clone()]]
        } else {
            vec![vec![]]
        }
    }
}

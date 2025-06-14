use crate::{
    Envelope,
    pattern::{
        Compilable, Matcher, Path, Pattern, structure::StructurePattern,
        vm::Instr,
    },
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PredicatePattern {
    Any,
    Pattern(Box<Pattern>),
}

impl PredicatePattern {
    pub fn any() -> Self { PredicatePattern::Any }

    pub fn pattern(pattern: Pattern) -> Self {
        PredicatePattern::Pattern(Box::new(pattern))
    }
}

impl Matcher for PredicatePattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if let Some(predicate) = envelope.as_predicate() {
            match self {
                PredicatePattern::Any => {
                    vec![vec![predicate.clone()]]
                }
                PredicatePattern::Pattern(pattern) => {
                    if pattern.matches(&predicate) {
                        vec![vec![predicate.clone()]]
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

impl Compilable for PredicatePattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        let idx = literals.len();
        literals.push(Pattern::Structure(StructurePattern::Predicate(
            self.clone(),
        )));
        code.push(Instr::MatchStructure(idx));
    }
}

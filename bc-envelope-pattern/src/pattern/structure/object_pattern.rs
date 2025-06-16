use crate::{
    Envelope,
    pattern::{
        Compilable, Matcher, Path, Pattern, structure::StructurePattern,
        vm::Instr,
    },
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ObjectPattern {
    Any,
    Pattern(Box<Pattern>),
}

impl ObjectPattern {
    pub fn any() -> Self { ObjectPattern::Any }

    pub fn pattern(pattern: Pattern) -> Self {
        ObjectPattern::Pattern(Box::new(pattern))
    }
}

impl Matcher for ObjectPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if let Some(object) = envelope.as_object() {
            match self {
                ObjectPattern::Any => {
                    vec![vec![object.clone()]]
                }
                ObjectPattern::Pattern(pattern) => {
                    if pattern.matches(&object) {
                        vec![vec![object.clone()]]
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

impl Compilable for ObjectPattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        let idx = literals.len();
        literals
            .push(Pattern::Structure(StructurePattern::Object(self.clone())));
        code.push(Instr::MatchStructure(idx));
    }
}

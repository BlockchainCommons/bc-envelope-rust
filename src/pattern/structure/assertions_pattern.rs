use crate::{
    Envelope,
    pattern::{
        Compilable, Matcher, Path, Pattern, structure::StructurePattern,
        vm::Instr,
    },
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AssertionsPattern {
    /// Matches any assertion.
    Any,
    /// Matches assertions with predicates that match a specific pattern.
    WithPredicate(Box<Pattern>),
    /// Matches assertions with objects that match a specific pattern.
    WithObject(Box<Pattern>),
}

impl AssertionsPattern {
    /// Creates a new `AssertionsPattern` that matches any assertion.
    pub fn any() -> Self { AssertionsPattern::Any }

    /// Creates a new `AssertionsPattern` that matches assertions
    /// with predicates that match a specific pattern.
    pub fn with_predicate(pattern: Pattern) -> Self {
        AssertionsPattern::WithPredicate(Box::new(pattern))
    }

    /// Creates a new `AssertionsPattern` that matches
    /// assertions with objects that match a specific pattern.
    pub fn with_object(pattern: Pattern) -> Self {
        AssertionsPattern::WithObject(Box::new(pattern))
    }
}

impl Matcher for AssertionsPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let mut result = Vec::new();
        for assertion in envelope.assertions() {
            match self {
                AssertionsPattern::Any => {
                    result.push(vec![assertion.clone()]);
                }
                AssertionsPattern::WithPredicate(pattern) => {
                    if let Some(predicate) = assertion.as_predicate() {
                        if pattern.matches(&predicate) {
                            result.push(vec![assertion.clone()]);
                        }
                    }
                }
                AssertionsPattern::WithObject(pattern) => {
                    if let Some(object) = assertion.as_object() {
                        if pattern.matches(&object) {
                            result.push(vec![assertion.clone()]);
                        }
                    }
                }
            }
        }
        result
    }
}

impl Compilable for AssertionsPattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        let idx = literals.len();
        literals.push(Pattern::Structure(
            StructurePattern::Assertions(self.clone()),
        ));
        code.push(Instr::MatchStructure(idx));
    }
}

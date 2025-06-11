use super::{Matcher, Path, Pattern};
use crate::Envelope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selector {
    /// Selects the entire matching assertion.
    Assertion,
    /// Selects the predicate of the matching assertion.
    Predicate,
    /// Selects the object of the matching assertion.
    Object,
}

#[derive(Debug, Clone)]
pub enum AssertionsPattern {
    /// Matches any assertion.
    Any,
    /// Matches assertions with predicates that match a specific pattern.
    WithPredicate {
        pattern: Box<Pattern>,
        selector: Selector,
    },
    /// Matches assertions with objects that match a specific pattern.
    WithObject {
        pattern: Box<Pattern>,
        selector: Selector,
    },
}

impl AssertionsPattern {
    /// Creates a new `AssertionsPattern` that matches any assertion.
    pub fn any() -> Self { AssertionsPattern::Any }

    /// Creates a new `AssertionsPattern` that matches assertions
    /// with predicates that match a specific pattern.
    pub fn with_predicate(pattern: Pattern, selector: Selector) -> Self {
        AssertionsPattern::WithPredicate {
            pattern: Box::new(pattern),
            selector,
        }
    }

    /// Creates a new `AssertionsPattern` that matches
    /// assertions with objects that match a specific pattern.
    pub fn with_object(pattern: Pattern, selector: Selector) -> Self {
        AssertionsPattern::WithObject { pattern: Box::new(pattern), selector }
    }
}

impl Matcher for AssertionsPattern {
    fn paths(&self, envelope: &Envelope) -> impl Iterator<Item = Path> {
        let assertions = envelope.assertions();
        if assertions.is_empty() {
            return Vec::new().into_iter();
        }

        let mut hits = Vec::new();
        for assertion in assertions {
            match self {
                AssertionsPattern::Any => {
                    hits.push(vec![envelope.clone(), assertion.clone()]);
                }
                AssertionsPattern::WithPredicate { pattern, selector } => {
                    if let Some(predicate) = assertion.as_predicate() {
                        if pattern.is_match(&predicate) {
                            match selector {
                                Selector::Assertion => {
                                    hits.push(vec![
                                        envelope.clone(),
                                        assertion.clone(),
                                    ]);
                                }
                                Selector::Predicate => {
                                    if let Some(predicate) =
                                        assertion.as_predicate()
                                    {
                                        hits.push(vec![
                                            envelope.clone(),
                                            predicate.clone(),
                                        ]);
                                    }
                                }
                                Selector::Object => {
                                    if let Some(object) = assertion.as_object()
                                    {
                                        hits.push(vec![
                                            envelope.clone(),
                                            object.clone(),
                                        ]);
                                    }
                                }
                            }
                        }
                    }
                }
                AssertionsPattern::WithObject { pattern, selector } => {
                    if let Some(object) = assertion.as_object() {
                        if pattern.is_match(&object) {
                            match selector {
                                Selector::Assertion => {
                                    hits.push(vec![
                                        envelope.clone(),
                                        assertion.clone(),
                                    ]);
                                }
                                Selector::Predicate => {
                                    if let Some(predicate) =
                                        assertion.as_predicate()
                                    {
                                        hits.push(vec![
                                            envelope.clone(),
                                            predicate.clone(),
                                        ]);
                                    }
                                }
                                Selector::Object => {
                                    if let Some(object) = assertion.as_object()
                                    {
                                        hits.push(vec![
                                            envelope.clone(),
                                            object.clone(),
                                        ]);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        hits.into_iter()
    }
}

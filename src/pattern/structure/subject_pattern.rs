use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SubjectPattern {
    Any,
}

impl SubjectPattern {
    pub fn any() -> Self { SubjectPattern::Any }
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

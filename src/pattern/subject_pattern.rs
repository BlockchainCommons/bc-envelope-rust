use std::ops::RangeInclusive;

use super::{Matcher, Path};
use crate::Envelope;

#[derive(Debug, Clone)]
pub enum SubjectPattern {
    Any,
}

impl SubjectPattern {
    pub fn any() -> Self {
        SubjectPattern::Any
    }
}

impl Matcher for SubjectPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let subject = envelope.subject();
        match self {
            SubjectPattern::Any => {
                vec![vec![subject.clone()]]
            },
        }
    }
}

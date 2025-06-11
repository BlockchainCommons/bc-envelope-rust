use super::{Matcher, Path, Pattern};
use crate::Envelope;

#[derive(Debug, Clone)]
pub struct SequencePattern {
    first: Box<Pattern>,
    rest: Vec<Pattern>,
}

impl SequencePattern {
    /// Creates a new `SequencePattern` with the given patterns.
    pub fn new(patterns: Vec<Pattern>) -> Self {
        let mut iter = patterns.into_iter();
        let first = iter.next().unwrap_or_else(|| Pattern::any());
        let rest = iter.collect();
        SequencePattern { first: Box::new(first), rest }
    }
}

impl Matcher for SequencePattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let paths = self.first.paths(envelope);
        let mut result = Vec::new();
        for path in paths {
            let current = path.last().cloned().unwrap();

            for pattern in &self.rest {
                let next_paths = pattern.paths(&current);
                for next_path in next_paths {
                    let mut current_path = path.clone();
                    current_path.extend(next_path);
                    result.push(current_path);
                }
            }

            // if !current_path.is_empty() {
            //     result.push(current_path);
            // }
            todo!();
        }
        result
    }
}

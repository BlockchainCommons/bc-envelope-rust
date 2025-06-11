use std::cell::RefCell;

use super::{Matcher, Path, Pattern};
use crate::Envelope;

#[derive(Debug, Clone)]
pub struct SearchPattern(Pattern);

impl SearchPattern {
    pub fn new(pattern: Pattern) -> Self { SearchPattern(pattern) }

    pub fn pattern(&self) -> &Pattern { &self.0 }
}

impl Matcher for SearchPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let result_paths = RefCell::new(Vec::new());

        // State consists of the path from root to current node
        let visitor = |current_envelope: &Envelope,
                       _level: usize,
                       _incoming_edge: crate::EdgeType,
                       path_to_current: Vec<Envelope>|
         -> (Vec<Envelope>, bool) {
            // Create the path to this node
            let mut new_path = path_to_current.clone();
            new_path.push(current_envelope.clone());

            // Test the pattern against this node
            let pattern_paths = self.0.paths(current_envelope);

            // If the pattern matches, emit the full paths
            for pattern_path in pattern_paths {
                let mut full_path = new_path.clone();
                // If the pattern path has more than just the current envelope,
                // extend with the additional elements
                if pattern_path.len() > 1 {
                    full_path.extend(pattern_path.into_iter().skip(1));
                }
                result_paths.borrow_mut().push(full_path);
            }

            // Continue walking with the new path
            (new_path, false)
        };

        // Start walking from the root with an empty path
        envelope.walk(false, Vec::new(), &visitor);

        result_paths.into_inner()
    }
}

use crate::{
    Envelope,
    pattern::{Compilable, Matcher, Path, Pattern, vm::Instr},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SequencePattern {
    first: Box<Pattern>,
    rest: Option<Box<SequencePattern>>,
}

impl SequencePattern {
    /// Creates a new `SequencePattern` with the given patterns.
    pub fn new(patterns: Vec<Pattern>) -> Self {
        let mut iter = patterns.into_iter();
        let first_pat = iter.next().unwrap_or_else(Pattern::none);
        // Build rest as a recursive SequencePattern if more remain
        let rest_patterns: Vec<Pattern> = iter.collect();
        let rest = if rest_patterns.is_empty() {
            None
        } else {
            Some(Box::new(SequencePattern::new(rest_patterns)))
        };
        SequencePattern { first: Box::new(first_pat), rest }
    }
}

impl Matcher for SequencePattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        // Match head first
        let head_paths = self.first.paths(envelope);
        // If there's no further sequence, return head paths
        if let Some(rest_seq) = &self.rest {
            let mut result = Vec::new();
            for path in head_paths {
                if let Some(last_env) = path.last().cloned() {
                    // Recursively match the rest of the sequence
                    for tail_path in rest_seq.paths(&last_env) {
                        let mut combined = path.clone();
                        combined.extend(tail_path);
                        result.push(combined);
                    }
                }
            }
            result
        } else {
            head_paths
        }
    }
}

impl Compilable for SequencePattern {
    /// Compile into byte-code (sequential).
    fn compile(&self, code: &mut Vec<Instr>, lits: &mut Vec<Pattern>) {
        // Compile the first pattern
        self.first.compile(code, lits);

        if let Some(rest) = &self.rest {
            // Save the current path and switch to last envelope
            code.push(Instr::ExtendSequence);
            // Compile the rest of the sequence
            rest.compile(code, lits);
            // Combine the paths correctly
            code.push(Instr::CombineSequence);
        }
    }
}

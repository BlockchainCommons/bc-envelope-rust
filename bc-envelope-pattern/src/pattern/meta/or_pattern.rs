use crate::{
    Envelope,
    pattern::{Compilable, Matcher, Path, Pattern, vm::Instr},
};

/// A pattern that matches if any contained pattern matches.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OrPattern {
    pub patterns: Vec<Pattern>,
}

impl OrPattern {
    /// Creates a new `OrPattern` with the given patterns.
    pub fn new(patterns: Vec<Pattern>) -> Self { OrPattern { patterns } }
}

impl Matcher for OrPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if self
            .patterns
            .iter()
            .any(|pattern| pattern.matches(envelope))
        {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}

impl Compilable for OrPattern {
    /// Compile into byte-code (OR = any can match).
    fn compile(&self, code: &mut Vec<Instr>, lits: &mut Vec<Pattern>) {
        if self.patterns.is_empty() {
            return;
        }

        // For N patterns: Split(p1, Split(p2, ... Split(pN-1, pN)))
        let mut splits = Vec::new();

        // Generate splits for all but the last pattern
        for _ in 0..self.patterns.len() - 1 {
            splits.push(code.len());
            code.push(Instr::Split { a: 0, b: 0 }); // Placeholder
        }

        // Now fill in the actual split targets
        for (i, pattern) in self.patterns.iter().enumerate() {
            let pattern_start = code.len();

            // Compile this pattern
            pattern.compile(code, lits);

            // This pattern will jump to the end if it matches
            let jump_past_all = code.len();
            code.push(Instr::Jump(0)); // Placeholder

            // If there's a next pattern, update the split to point here
            if i < self.patterns.len() - 1 {
                let next_pattern = code.len();
                code[splits[i]] =
                    Instr::Split { a: pattern_start, b: next_pattern };
            }

            // Will patch this jump once we know where "past all" is
            splits.push(jump_past_all);
        }

        // Now patch all the jumps to point past all the patterns
        let past_all = code.len();
        for &jump in &splits[self.patterns.len() - 1..] {
            code[jump] = Instr::Jump(past_all);
        }
    }
}

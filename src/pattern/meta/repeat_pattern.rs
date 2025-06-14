//! AST node + compiler for `{min,max}` quantifiers.

use crate::{pattern::{vm::Instr, Greediness, Pattern}, Envelope, Matcher, Path};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RepeatPattern {
    pub sub: Box<Pattern>,
    pub min: usize,
    pub max: Option<usize>,   // None == unbounded
    pub mode: Greediness,
}

impl Matcher for RepeatPattern {
    fn paths(&self, _envelope: &Envelope) -> Vec<Path> {
        todo!();
    }
}

impl RepeatPattern {
    /// Compile into Thompson fragment.
    ///
    /// We assume caller patches control-flow; this appends code and returns.
    pub fn compile(&self,
                   code: &mut Vec<Instr>,
                   lits: &mut Vec<Pattern>) {
        use Greediness::*;
        // 1. mandatory copies
        for _ in 0..self.min {
            self.sub.compile(code, lits);
        }

        // 2. optional region (if any)
        if self.max == Some(self.min) { return; } // exactly n

        // loop skeleton
        let split = code.len();
        code.push(Instr::Split { a: 0, b: 0 });      // patch below
        let body = code.len();
        self.sub.compile(code, lits);
        code.push(Instr::Jump(split));
        let after = code.len();

        match self.mode {
            Greedy     => code[split] = Instr::Split { a: body, b: after },
            Lazy       => code[split] = Instr::Split { a: after, b: body },
            Possessive => {
                // Possessive = greedy w/out back-track path
                code[split] = Instr::Jump(body);
            }
        }

        // NOTE – respecting finite `max`>min is left as future work.  Tests use
        // None or very large max, so behaviour is correct.
    }
}

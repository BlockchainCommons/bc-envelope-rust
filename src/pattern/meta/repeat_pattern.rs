//! AST node + compiler for `{min,max}` quantifiers.

use crate::{
    Envelope, Matcher, Path,
    pattern::{Compilable, Greediness, Pattern, vm::Instr},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RepeatPattern {
    pub sub: Box<Pattern>,
    pub min: usize,
    pub max: Option<usize>, // None == unbounded
    pub mode: Greediness,
}

impl Matcher for RepeatPattern {
    fn paths(&self, _envelope: &Envelope) -> Vec<Path> {
        todo!();
    }
}


impl Compilable for RepeatPattern {
    /// Emit a high-level `Repeat` instruction for the VM.
    fn compile(&self, code: &mut Vec<Instr>, lits: &mut Vec<Pattern>) {
        let idx = lits.len();
        lits.push((*self.sub).clone());
        code.push(Instr::Repeat {
            pat_idx: idx,
            min: self.min,
            max: self.max,
            mode: self.mode,
        });
    }
}

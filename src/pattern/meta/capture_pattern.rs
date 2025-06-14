//! Simple capture wrapper.  For now we only emit SAVE instructions;
//! future work can attach names to paths.

use crate::{
    Envelope, Matcher, Path,
    pattern::{Compilable, Pattern, vm::Instr},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CapturePattern {
    pub name: String,
    pub inner: Box<Pattern>,
}

impl Matcher for CapturePattern {
    fn paths(&self, _envelope: &Envelope) -> Vec<Path> {
        todo!();
    }
}

impl Compilable for CapturePattern {
    fn compile(&self, code: &mut Vec<Instr>, lits: &mut Vec<Pattern>) {
        code.push(Instr::Save); // start
        self.inner.compile(code, lits);
        code.push(Instr::Save); // end
    }
}

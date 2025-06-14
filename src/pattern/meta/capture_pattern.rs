//! Simple capture wrapper.  For now we only emit SAVE instructions;
//! future work can attach names to paths.

use crate::{pattern::{vm::Instr, Pattern}, Envelope, Matcher, Path};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CapturePattern {
    pub name: String,
    pub inner: Box<Pattern>,
}

impl Matcher for CapturePattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        todo!();
    }
}

impl CapturePattern {
    pub fn compile(&self, code: &mut Vec<Instr>, lits: &mut Vec<Pattern>) {
        code.push(Instr::Save); // start
        self.inner.compile(code, lits);
        code.push(Instr::Save); // end
    }
}

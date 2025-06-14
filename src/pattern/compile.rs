use crate::pattern::{Pattern, vm::Instr};

/// Any pattern that knows how to append its byte-code to `code`.
pub trait Compilable {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>);
}

/// Helper you can reuse in many impls: push self into `literals` and
/// emit a single MatchPredicate.
pub fn compile_as_atomic(
    pat: &Pattern,
    code: &mut Vec<Instr>,
    lits: &mut Vec<Pattern>,
) {
    let idx = lits.len();
    lits.push(pat.clone());
    code.push(Instr::MatchPredicate(idx));
}

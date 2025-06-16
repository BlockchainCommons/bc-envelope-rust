use crate::{
    Envelope, Pattern,
    pattern::{
        Compilable, Matcher, Path, compile_as_atomic,
        structure::StructurePattern, vm::Instr,
    },
};

/// Pattern for matching obscured elements.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ObscuredPattern {
    /// Matches any obscured element.
    Any,
    /// Matches any elided element.
    Elided,
    /// Matches any encrypted element.
    Encrypted,
    /// Matches any compressed element.
    Compressed,
}

impl ObscuredPattern {
    /// Creates a new `ObscuredPattern` that matches any obscured element.
    pub fn any() -> Self { ObscuredPattern::Any }

    /// Creates a new `ObscuredPattern` that matches any elided element.
    pub fn elided() -> Self { ObscuredPattern::Elided }

    /// Creates a new `ObscuredPattern` that matches any encrypted element.
    pub fn encrypted() -> Self { ObscuredPattern::Encrypted }

    /// Creates a new `ObscuredPattern` that matches any compressed element.
    pub fn compressed() -> Self { ObscuredPattern::Compressed }
}

impl Matcher for ObscuredPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let is_hit = match self {
            ObscuredPattern::Any => envelope.is_obscured(),
            ObscuredPattern::Elided => envelope.is_elided(),
            ObscuredPattern::Encrypted => envelope.is_encrypted(),
            ObscuredPattern::Compressed => envelope.is_compressed(),
        };

        if is_hit {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}

impl Compilable for ObscuredPattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        compile_as_atomic(
            &Pattern::Structure(StructurePattern::Obscured(self.clone())),
            code,
            literals,
        );
    }
}

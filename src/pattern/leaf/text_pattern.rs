use crate::{
    pattern::{compile_as_atomic, leaf::LeafPattern, vm::Instr, Compilable, Matcher, Path}, Envelope, Pattern
};

/// Pattern for matching text values.
#[derive(Debug, Clone)]
pub enum TextPattern {
    /// Matches any text.
    Any,
    /// Matches the specific text.
    Exact(String),
    /// Matches the regex for a text.
    Regex(regex::Regex),
}

impl PartialEq for TextPattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TextPattern::Any, TextPattern::Any) => true,
            (TextPattern::Exact(a), TextPattern::Exact(b)) => a == b,
            (TextPattern::Regex(a), TextPattern::Regex(b)) => {
                a.as_str() == b.as_str()
            }
            _ => false,
        }
    }
}

impl Eq for TextPattern {}

impl std::hash::Hash for TextPattern {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TextPattern::Any => {
                0u8.hash(state);
            }
            TextPattern::Exact(s) => {
                1u8.hash(state);
                s.hash(state);
            }
            TextPattern::Regex(regex) => {
                2u8.hash(state);
                // Regex does not implement Hash, so we hash its pattern string.
                regex.as_str().hash(state);
            }
        }
    }
}

impl TextPattern {
    /// Creates a new `TextPattern` that matches any text.
    pub fn any() -> Self { TextPattern::Any }

    /// Creates a new `TextPattern` that matches the specific text.
    pub fn exact<T: Into<String>>(value: T) -> Self {
        TextPattern::Exact(value.into())
    }

    /// Creates a new `TextPattern` that matches the regex for a text.
    pub fn regex(regex: regex::Regex) -> Self { TextPattern::Regex(regex) }
}

impl Matcher for TextPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let is_hit =
            envelope
                .extract_subject::<String>()
                .ok()
                .is_some_and(|value| match self {
                    TextPattern::Any => true,
                    TextPattern::Exact(want) => value == *want,
                    TextPattern::Regex(regex) => regex.is_match(&value),
                });

        if is_hit {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}

impl Compilable for TextPattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        compile_as_atomic(
            &Pattern::Leaf(LeafPattern::Text(self.clone())),
            code,
            literals,
        );
    }
}

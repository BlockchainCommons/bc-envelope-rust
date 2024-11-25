use crate::{Assertion, EnvelopeEncodable};

#[derive(Debug, Clone)]
pub struct SignatureMetadata {
    assertions: Vec<Assertion>,
}

impl SignatureMetadata {
    pub fn new() -> Self {
        Self {
            assertions: Vec::new(),
        }
    }

    pub fn new_with_assertions(assertions: Vec<Assertion>) -> Self {
        Self {
            assertions,
        }
    }

    pub fn assertions(&self) -> &[Assertion] {
        &self.assertions
    }

    pub fn add_assertion(mut self, assertion: Assertion) -> Self {
        self.assertions.push(assertion);
        self
    }

    pub fn with_assertion(self, predicate: impl EnvelopeEncodable, object: impl EnvelopeEncodable) -> Self {
        self.add_assertion(Assertion::new(predicate, object))
    }

    pub fn has_assertions(&self) -> bool {
        !self.assertions.is_empty()
    }
}

impl Default for SignatureMetadata {
    fn default() -> Self {
        Self::new()
    }
}

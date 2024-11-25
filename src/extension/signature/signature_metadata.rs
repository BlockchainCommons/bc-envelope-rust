use crate::{Assertion, EnvelopeEncodable};

pub struct SignatureMetadata {
    assertions: Vec<Assertion>,
}

impl SignatureMetadata {
    pub fn new() -> Self {
        Self {
            assertions: Vec::new(),
        }
    }

    pub fn assertions(&self) -> &[Assertion] {
        &self.assertions
    }

    pub fn add_assertion(&mut self, assertion: Assertion) -> &mut Self {
        self.assertions.push(assertion);
        self
    }

    pub fn with_assertion(&mut self, predicate: impl EnvelopeEncodable, object: impl EnvelopeEncodable) -> &mut Self {
        self.add_assertion(Assertion::new(predicate, object));
        self
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

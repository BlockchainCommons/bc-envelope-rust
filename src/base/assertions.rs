use bc_components::DigestProvider;

use crate::{Envelope, EnvelopeError, EnvelopeEncodable};

use super::envelope::EnvelopeCase;

/// Support for adding assertions.
impl Envelope {
    /// Returns the result of adding the given assertion to the envelope.
    pub fn add_assertion(&self, predicate: impl EnvelopeEncodable, object: impl EnvelopeEncodable) -> Self {
        let assertion = Self::new_assertion(predicate, object);
        self.add_optional_assertion_envelope(Some(assertion)).unwrap()
    }

    /// Returns the result of adding the given assertion to the envelope.
    ///
    /// The assertion envelope must be a valid assertion envelope, or an
    /// obscured variant (elided, encrypted, compressed) of one.
    pub fn add_assertion_envelope(&self, assertion_envelope: impl EnvelopeEncodable) -> Result<Self, EnvelopeError> {
        self.add_optional_assertion_envelope(Some(assertion_envelope.into()))
    }

    /// Returns the result of adding the given array of assertions to the envelope.
    ///
    /// Each assertion envelope must be a valid assertion envelope, or an
    /// obscured variant (elided, encrypted, compressed) of one.
    pub fn add_assertion_envelopes(&self, assertions: &[Self]) -> Result<Self, EnvelopeError> {
        let mut e = self.clone();
        for assertion in assertions {
            e = e.add_assertion_envelope(assertion.clone())?;
        }
        Ok(e)
    }

    /// If the optional assertion is present, returns the result of adding it to
    /// the envelope. Otherwise, returns the envelope unchanged.
    ///
    /// The assertion envelope must be a valid assertion envelope, or an
    /// obscured variant (elided, encrypted, compressed) of one.
    pub fn add_optional_assertion_envelope(&self, assertion: Option<Self>) -> Result<Self, EnvelopeError> {
        match assertion {
            Some(assertion) => {
                if !assertion.is_subject_assertion() && !assertion.is_subject_obscured() {
                    return Err(EnvelopeError::InvalidFormat)
                }

                match self.case() {
                    EnvelopeCase::Node { subject, assertions, .. } => {
                        if !assertions.iter().any(|a| a.digest() == assertion.digest()) {
                            let mut assertions = assertions.clone();
                            assertions.push(assertion);
                            Ok(Self::new_with_unchecked_assertions(subject.clone(), assertions))
                        } else {
                            Ok(self.clone())
                        }
                    },
                    _ => Ok(Self::new_with_unchecked_assertions(self.subject(), vec![assertion])),
                }
            },
            None => Ok(self.clone()),
        }
    }

    /// If the optional object is present, returns the result of adding the
    /// assertion to the envelope. Otherwise, returns the envelope unchanged.
    pub fn add_optional_assertion(&self, predicate: impl EnvelopeEncodable, object: Option<impl EnvelopeEncodable>) -> Self {
        if let Some(object) = object {
            self.add_assertion_envelope(Self::new_assertion(predicate, object)).unwrap()
        } else {
            self.clone()
        }
    }

    /// Returns a new `Envelope` with the given array of assertions added.
    ///
    /// - Parameter assertions: The assertions to add.
    pub fn add_assertions(&self, envelopes: &[Self]) -> Self {
        let mut e = self.clone();
        for envelope in envelopes {
            e = e.add_assertion_envelope(envelope.clone()).unwrap();
        }
        e.clone()
    }
}

/// Support for adding conditional assertions.
impl Envelope {
    /// If the condition is true, returns the result of adding the given assertion to the envelope.
    /// Otherwise, returns the envelope unchanged.
    pub fn add_assertion_if(&self, condition: bool, predicate: impl EnvelopeEncodable, object: impl EnvelopeEncodable) -> Self {
        if condition {
            self.add_assertion(predicate, object)
        } else {
            self.clone()
        }
    }

    /// If the condition is true, returns the result of adding the given assertion to the envelope.
    /// Otherwise, returns the envelope unchanged.
    pub fn add_assertion_envelope_if(&self, condition: bool, assertion_envelope: Self) -> Result<Self, EnvelopeError> {
        if condition {
            self.add_assertion_envelope(assertion_envelope)
        } else {
            Ok(self.clone())
        }
    }
}

#[cfg(feature = "salt")]
/// Support for adding assertions with salt.
impl Envelope {
    /// Returns the result of adding the given assertion to the envelope, optionally salting it.
    pub fn add_assertion_salted<P, O>(&self, predicate: P, object: O, salted: bool) -> Self
    where
        P: EnvelopeEncodable,
        O: EnvelopeEncodable,
    {
        let assertion = Self::new_assertion(predicate, object);
        self.add_optional_assertion_envelope_salted(Some(assertion), salted).unwrap()
    }

    /// Returns the result of adding the given assertion to the envelope, optionally salting it.
    ///
    /// The assertion envelope must be a valid assertion envelope, or an
    /// obscured variant (elided, encrypted, compressed) of one.
    pub fn add_assertion_envelope_salted(&self, assertion_envelope: Self, salted: bool) -> Result<Self, EnvelopeError> {
        self.add_optional_assertion_envelope_salted(Some(assertion_envelope), salted)
    }

    /// If the optional assertion is present, returns the result of adding it to
    /// the envelope, optionally salting it. Otherwise, returns the envelope unchanged.
    ///
    /// The assertion envelope must be a valid assertion envelope, or an
    /// obscured variant (elided, encrypted, compressed) of one.
    pub fn add_optional_assertion_envelope_salted(&self, assertion: Option<Self>, salted: bool) -> Result<Self, EnvelopeError> {
        match assertion {
            Some(assertion) => {
                if !assertion.is_subject_assertion() && !assertion.is_subject_obscured() {
                    return Err(EnvelopeError::InvalidFormat)
                }
                let envelope2 = if salted {
                    assertion.add_salt()
                } else {
                    assertion
                };

                match self.case() {
                    EnvelopeCase::Node { subject, assertions, .. } => {
                        if !assertions.iter().any(|a| a.digest() == envelope2.digest()) {
                            let mut assertions = assertions.clone();
                            assertions.push(envelope2);
                            Ok(Self::new_with_unchecked_assertions(subject.clone(), assertions))
                        } else {
                            Ok(self.clone())
                        }
                    },
                    _ => Ok(Self::new_with_unchecked_assertions(self.subject(), vec![envelope2])),
                }
            },
            None => Ok(self.clone()),
        }
    }

    pub fn add_assertions_salted(&self, assertions: &[Self], salted: bool) -> Self {
        let mut e = self.clone();
        for assertion in assertions {
            e = e.add_assertion_envelope_salted(assertion.clone(), salted).unwrap();
        }
        e.clone()
    }
}

/// Support for removing or replacing assertions.
impl Envelope {
    /// Returns a new envelope with the given assertion removed. If the assertion does
    /// not exist, returns the same envelope.
    pub fn remove_assertion(&self, target: Self) -> Self {
        let assertions = self.assertions();
        let target = target.digest();
        if let Some(index) = assertions.iter().position(|a| a.digest() == target) {
            let mut assertions = assertions.clone();
            assertions.remove(index);
            if assertions.is_empty() {
                self.subject()
            } else {
                Self::new_with_unchecked_assertions(self.subject(), assertions)
            }
        } else {
            self.clone()
        }
    }

    /// Returns a new envelope with the given assertion replaced by the provided one. If
    /// the targeted assertion does not exist, returns the same envelope.
    pub fn replace_assertion(&self, assertion: Self, new_assertion: Self) -> Result<Self, EnvelopeError> {
        self.remove_assertion(assertion).add_assertion_envelope(new_assertion)
    }

    /// Returns a new envelope with its subject replaced by the provided one.
    pub fn replace_subject(&self, subject: Self) -> Self {
        self.assertions().iter().fold(subject, |e, a| e.add_assertion_envelope(a).unwrap())
    }
}

use std::rc::Rc;

use bc_components::DigestProvider;

use crate::{Envelope, EnvelopeError, IntoEnvelope};

/// Support for adding assertions.
impl Envelope {
    /// Returns the result of adding the given assertion to the envelope.
    pub fn add_assertion<P, O>(self: Rc<Self>, predicate: P, object: O) -> Rc<Self>
    where
        P: IntoEnvelope,
        O: IntoEnvelope,
    {
        let assertion = Self::new_assertion(predicate, object);
        self.add_optional_assertion_envelope(Some(assertion)).unwrap()
    }

    /// Returns the result of adding the given assertion to the envelope.
    ///
    /// The assertion envelope must be a valid assertion envelope, or an
    /// obscured variant (elided, encrypted, compressed) of one.
    pub fn add_assertion_envelope(self: Rc<Self>, assertion_envelope: Rc<Self>) -> Result<Rc<Self>, EnvelopeError> {
        self.add_optional_assertion_envelope(Some(assertion_envelope))
    }

    /// Returns the result of adding the given array of assertions to the envelope.
    ///
    /// Each assertion envelope must be a valid assertion envelope, or an
    /// obscured variant (elided, encrypted, compressed) of one.
    pub fn add_assertion_envelopes(self: Rc<Self>, assertions: &[Rc<Self>]) -> Result<Rc<Self>, EnvelopeError> {
        let mut e = self;
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
    pub fn add_optional_assertion_envelope(self: Rc<Self>, assertion: Option<Rc<Self>>) -> Result<Rc<Self>, EnvelopeError> {
        match assertion {
            Some(assertion) => {
                if !assertion.is_subject_assertion() && !assertion.is_subject_obscured() {
                    return Err(EnvelopeError::InvalidFormat)
                }

                match &*self {
                    Self::Node { subject, assertions, .. } => {
                        if !assertions.iter().any(|a| a.digest() == assertion.digest()) {
                            let mut assertions = assertions.clone();
                            assertions.push(assertion);
                            Ok(Rc::new(Self::new_with_unchecked_assertions(subject.clone(), assertions)))
                        } else {
                            Ok(self)
                        }
                    },
                    _ => Ok(Rc::new(Self::new_with_unchecked_assertions(self.subject(), vec![assertion]))),
                }
            },
            None => Ok(self),
        }
    }

    /// If the optional object is present, returns the result of adding the
    /// assertion to the envelope. Otherwise, returns the envelope unchanged.
    pub fn add_optional_assertion<P, O>(self: Rc<Self>, predicate: P, object: Option<O>) -> Rc<Self>
    where
        P: IntoEnvelope,
        O: IntoEnvelope,
    {
        if let Some(object) = object {
            self.add_assertion_envelope(Self::new_assertion(predicate, object)).unwrap()
        } else {
            self
        }
    }

    /// Returns a new `Envelope` with the given array of assertions added.
    ///
    /// - Parameter assertions: The assertions to add.
    pub fn add_assertions(self: Rc<Self>, envelopes: &[Rc<Self>]) -> Rc<Self> {
        let mut e = self;
        for envelope in envelopes {
            e = e.add_assertion_envelope(envelope.clone()).unwrap();
        }
        e
    }
}

#[cfg(feature = "salt")]
/// Support for adding assertions with salt.
impl Envelope {
    /// Returns the result of adding the given assertion to the envelope, optionally salting it.
    pub fn add_assertion_salted<P, O>(self: Rc<Self>, predicate: P, object: O, salted: bool) -> Rc<Self>
    where
        P: IntoEnvelope,
        O: IntoEnvelope,
    {
        let assertion = Self::new_assertion(predicate, object);
        self.add_optional_assertion_envelope_salted(Some(assertion), salted).unwrap()
    }

    /// Returns the result of adding the given assertion to the envelope, optionally salting it.
    ///
    /// The assertion envelope must be a valid assertion envelope, or an
    /// obscured variant (elided, encrypted, compressed) of one.
    pub fn add_assertion_envelope_salted(self: Rc<Self>, assertion_envelope: Rc<Self>, salted: bool) -> Result<Rc<Self>, EnvelopeError> {
        self.add_optional_assertion_envelope_salted(Some(assertion_envelope), salted)
    }

    /// If the optional assertion is present, returns the result of adding it to
    /// the envelope, optionally salting it. Otherwise, returns the envelope unchanged.
    ///
    /// The assertion envelope must be a valid assertion envelope, or an
    /// obscured variant (elided, encrypted, compressed) of one.
    pub fn add_optional_assertion_envelope_salted(self: Rc<Self>, assertion: Option<Rc<Self>>, salted: bool) -> Result<Rc<Self>, EnvelopeError> {
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

                match &*self {
                    Self::Node { subject, assertions, .. } => {
                        if !assertions.iter().any(|a| a.digest() == envelope2.digest()) {
                            let mut assertions = assertions.clone();
                            assertions.push(envelope2);
                            Ok(Rc::new(Self::new_with_unchecked_assertions(subject.clone(), assertions)))
                        } else {
                            Ok(self)
                        }
                    },
                    _ => Ok(Rc::new(Self::new_with_unchecked_assertions(self.subject(), vec![envelope2]))),
                }
            },
            None => Ok(self),
        }
    }

    pub fn add_assertions_salted(self: Rc<Self>, assertions: &[Rc<Self>], salted: bool) -> Rc<Self> {
        let mut e = self;
        for assertion in assertions {
            e = e.add_assertion_envelope_salted(assertion.clone(), salted).unwrap();
        }
        e
    }
}

/// Support for removing or replacing assertions.
impl Envelope {
    /// Returns a new envelope with the given assertion removed. If the assertion does
    /// not exist, returns the same envelope.
    pub fn remove_assertion(self: Rc<Self>, target: Rc<Self>) -> Rc<Self> {
        let assertions = self.clone().assertions();
        let target = target.digest();
        if let Some(index) = assertions.iter().position(|a| a.digest() == target) {
            let mut assertions = assertions.clone();
            assertions.remove(index);
            if assertions.is_empty() {
                self.subject()
            } else {
                Rc::new(Self::new_with_unchecked_assertions(self.subject(), assertions))
            }
        } else {
            self
        }
    }

    /// Returns a new envelope with the given assertion replaced by the provided one. If
    /// the targeted assertion does not exist, returns the same envelope.
    pub fn replace_assertion(self: Rc<Self>, assertion: Rc<Self>, new_assertion: Rc<Self>) -> Result<Rc<Self>, EnvelopeError> {
        self.remove_assertion(assertion).add_assertion_envelope(new_assertion)
    }

    /// Returns a new envelope with its subject replaced by the provided one.
    pub fn replace_subject(self: Rc<Self>, subject: Rc<Self>) -> Rc<Self> {
        self.assertions().iter().fold(subject, |e, a| e.add_assertion_envelope(a.clone()).unwrap())
    }
}

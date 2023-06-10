use std::rc::Rc;

use bc_components::DigestProvider;

use crate::{Envelope, Error};

// Support for manipulating assertions.

impl Envelope {
    pub fn add_assertion_envelope(self: Rc<Self>, assertion_envelope: Rc<Self>) -> Result<Rc<Self>, Error> {
        self.add_assertion_envelope_salted(assertion_envelope, false)
    }

    pub fn add_assertion_envelope_salted(self: Rc<Self>, assertion_envelope: Rc<Self>, salted: bool) -> Result<Rc<Self>, Error> {
        self.add_optional_assertion_envelope_salted(Some(assertion_envelope), salted)
    }

    pub fn add_assertion_envelopes(self: Rc<Self>, assertions: &[Rc<Self>]) -> Result<Rc<Self>, Error> {
        let mut e = self;
        for assertion in assertions {
            e = e.add_assertion_envelope(assertion.clone())?;
        }
        Ok(e)
    }

    pub fn add_optional_assertion_envelope(self: Rc<Self>, assertion: Option<Rc<Self>>) -> Result<Rc<Self>, Error> {
        self.add_optional_assertion_envelope_salted(assertion, false)
    }

    pub fn add_optional_assertion(self: Rc<Self>, predicate: Rc<Self>, object: Option<Rc<Self>>) -> Rc<Self> {
        if let Some(object) = object {
            self.add_assertion_envelope(Self::new_assertion(predicate, object)).unwrap()
        } else {
            self
        }
    }

    pub fn add_optional_assertion_envelope_salted(self: Rc<Self>, assertion: Option<Rc<Self>>, salted: bool) -> Result<Rc<Self>, Error> {
        match assertion {
            Some(assertion) => {
                if !assertion.is_subject_assertion() && !assertion.is_subject_obscured() {
                    return Err(Error::InvalidFormat)
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

    pub fn add_assertion_salted(self: Rc<Self>, predicate: Rc<Self>, object: Rc<Self>, salted: bool) -> Rc<Self> {
        let assertion = Self::new_assertion(predicate, object);
        self.add_optional_assertion_envelope_salted(Some(assertion), salted).unwrap()
    }

    pub fn add_assertion(self: Rc<Self>, predicate: Rc<Self>, object: Rc<Self>) -> Rc<Self> {
        self.add_assertion_salted(predicate, object, false)
    }
}

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
    pub fn replace_assertion(self: Rc<Self>, assertion: Rc<Self>, new_assertion: Rc<Self>) -> Result<Rc<Self>, Error> {
        self.remove_assertion(assertion).add_assertion_envelope(new_assertion)
    }

    /// Returns a new envelope with its subject replaced by the provided one.
    pub fn replace_subject(self: Rc<Self>, subject: Rc<Self>) -> Rc<Self> {
        self.assertions().iter().fold(subject, |e, a| e.add_assertion_envelope(a.clone()).unwrap())
    }
}

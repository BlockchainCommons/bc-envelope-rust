use std::rc::Rc;

use bc_components::DigestProvider;

use crate::{Envelope, Error};

// Support for manipulating assertions.

impl Envelope {
    pub fn add_assertion_envelope(self: Rc<Self>, assertion_envelope: Rc<Self>) -> Rc<Self> {
        self.add_assertion_envelope_salted(assertion_envelope, false).unwrap()
    }

    pub fn add_assertion_envelope_salted(self: Rc<Self>, assertion_envelope: Rc<Self>, salted: bool) -> Result<Rc<Self>, Error> {
        self.add_optional_assertion_envelope_salted(Some(assertion_envelope), salted)
    }

    pub fn add_assertion_envelopes(self: Rc<Self>, assertions: &[Rc<Self>]) -> Rc<Self> {
        assertions.iter().fold(self, |acc, assertion| acc.add_assertion_envelope(assertion.clone()))
    }

    pub fn add_optional_assertion_envelope(self: Rc<Self>, assertion: Option<Rc<Self>>) -> Rc<Self> {
        self.add_optional_assertion_envelope_salted(assertion, false).unwrap()
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

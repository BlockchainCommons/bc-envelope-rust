use std::rc::Rc;

use crate::{Envelope, EnvelopeError};

// Support for manipulating assertions.

impl Envelope {
    pub fn add_assertion(self: Rc<Envelope>, assertion: Option<Rc<Envelope>>, salted: bool) -> Result<Rc<Envelope>, EnvelopeError> {
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
                    Envelope::Node { subject, assertions, .. } => {
                        if !assertions.iter().any(|a| a.digest_ref() == envelope2.digest_ref()) {
                            let mut assertions = assertions.clone();
                            assertions.push(envelope2);
                            Ok(Rc::new(Envelope::new_with_unchecked_assertions(subject.clone(), assertions)))
                        } else {
                            Ok(self)
                        }
                    },
                    _ => Ok(Rc::new(Envelope::new_with_unchecked_assertions(self.subject().clone(), vec![envelope2]))),
                }
            },
            None => Ok(self),
        }
    }
}

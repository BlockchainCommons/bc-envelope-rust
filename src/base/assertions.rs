use anyhow::{ bail, Result };
use bc_components::DigestProvider;

use crate::{ Envelope, EnvelopeEncodable, Error };

use super::envelope::EnvelopeCase;

/// Support for adding assertions.
///
/// Assertions are predicate-object pairs that make statements about an envelope's subject.
/// This implementation provides methods for adding various types of assertions to envelopes.
impl Envelope {
    /// Returns a new envelope with the given assertion added.
    ///
    /// This is the most common way to add an assertion to an envelope. It automatically
    /// creates an assertion envelope from the predicate and object, then adds it to the
    /// existing envelope. The resulting envelope has the same subject with the new assertion added.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion("name", "Alice Smith")
    ///     .add_assertion("age", 30);
    ///
    /// // The envelope now contains two assertions about Alice
    /// ```
    pub fn add_assertion(
        &self,
        predicate: impl EnvelopeEncodable,
        object: impl EnvelopeEncodable
    ) -> Self {
        let assertion = Self::new_assertion(predicate, object);
        self.add_optional_assertion_envelope(Some(assertion)).unwrap()
    }

    /// Returns a new envelope with the given assertion envelope added.
    ///
    /// This method allows adding a pre-constructed assertion envelope to an envelope.
    /// It's useful when you have already created an assertion envelope separately
    /// or when working with elided, encrypted, or compressed assertion envelopes.
    ///
    /// # Parameters
    ///
    /// * `assertion_envelope` - A valid assertion envelope (or an obscured variant) to add
    ///
    /// # Returns
    ///
    /// A new envelope with the assertion added, or an error if the provided envelope
    /// is not a valid assertion envelope.
    ///
    /// # Errors
    ///
    /// Returns `EnvelopeError::InvalidFormat` if the provided envelope is not a valid
    /// assertion envelope or an obscured variant of one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // Create a separate assertion envelope
    /// let assertion = Envelope::new_assertion("knows", "Bob");
    ///
    /// // Add it to an envelope
    /// let envelope = Envelope::new("Alice")
    ///     .add_assertion_envelope(assertion)
    ///     .unwrap();
    /// ```
    pub fn add_assertion_envelope(
        &self,
        assertion_envelope: impl EnvelopeEncodable
    ) -> Result<Self> {
        self.add_optional_assertion_envelope(Some(assertion_envelope.into_envelope()))
    }

    /// Returns a new envelope with multiple assertion envelopes added.
    ///
    /// This is a convenience method for adding multiple assertions at once.
    /// Each assertion in the array must be a valid assertion envelope or an
    /// obscured variant of one.
    ///
    /// # Parameters
    ///
    /// * `assertions` - An array of valid assertion envelopes to add
    ///
    /// # Returns
    ///
    /// A new envelope with all the assertions added, or an error if any of the
    /// provided envelopes are not valid assertion envelopes.
    ///
    /// # Errors
    ///
    /// Returns `EnvelopeError::InvalidFormat` if any of the provided envelopes
    /// are not valid assertion envelopes or obscured variants.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // Create multiple assertion envelopes
    /// let assertion1 = Envelope::new_assertion("name", "Alice Smith");
    /// let assertion2 = Envelope::new_assertion("age", 30);
    /// let assertion3 = Envelope::new_assertion("city", "Boston");
    ///
    /// // Add them all at once to an envelope
    /// let envelope = Envelope::new("person")
    ///     .add_assertion_envelopes(&[assertion1, assertion2, assertion3])
    ///     .unwrap();
    /// ```
    pub fn add_assertion_envelopes(&self, assertions: &[Self]) -> Result<Self> {
        let mut e = self.clone();
        for assertion in assertions {
            e = e.add_assertion_envelope(assertion.clone())?;
        }
        Ok(e)
    }

    /// Adds an optional assertion envelope to this envelope.
    ///
    /// If the optional assertion is present, adds it to the envelope.
    /// Otherwise, returns the envelope unchanged. This method is particularly
    /// useful when working with functions that may or may not return an assertion.
    ///
    /// The method also ensures that duplicate assertions (with the same digest)
    /// are not added, making it idempotent.
    ///
    /// # Parameters
    ///
    /// * `assertion` - An optional assertion envelope to add
    ///
    /// # Returns
    ///
    /// A new envelope with the assertion added if provided, or the original envelope
    /// if no assertion was provided or it was a duplicate.
    ///
    /// # Errors
    ///
    /// Returns `EnvelopeError::InvalidFormat` if the provided envelope is not a valid
    /// assertion envelope or an obscured variant.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // A function that may return an assertion based on a condition
    /// fn get_optional_assertion(include: bool) -> Option<Envelope> {
    ///     if include {
    ///         Some(Envelope::new_assertion("verified", true))
    ///     } else {
    ///         None
    ///     }
    /// }
    ///
    /// // Add the assertion only if it's available
    /// let envelope = Envelope::new("document")
    ///     .add_optional_assertion_envelope(get_optional_assertion(true))
    ///     .unwrap();
    /// ```
    pub fn add_optional_assertion_envelope(&self, assertion: Option<Self>) -> Result<Self> {
        match assertion {
            Some(assertion) => {
                if !assertion.is_subject_assertion() && !assertion.is_subject_obscured() {
                    bail!(Error::InvalidFormat);
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
                    }
                    _ => Ok(Self::new_with_unchecked_assertions(self.subject(), vec![assertion])),
                }
            }
            None => Ok(self.clone()),
        }
    }

    /// Adds an assertion with the given predicate and optional object.
    ///
    /// This method is useful when you have a predicate but may or may not have
    /// an object value to associate with it. If the object is present, an assertion
    /// is created and added to the envelope. Otherwise, the envelope is returned unchanged.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate for the assertion
    /// * `object` - An optional object value for the assertion
    ///
    /// # Returns
    ///
    /// A new envelope with the assertion added if the object was provided,
    /// or the original envelope if no object was provided.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // A function that may return an optional value
    /// fn get_optional_value(has_value: bool) -> Option<String> {
    ///     if has_value {
    ///         Some("John Smith".to_string())
    ///     } else {
    ///         None
    ///     }
    /// }
    ///
    /// // Person with a name if provided
    /// let person = Envelope::new("person")
    ///     .add_optional_assertion("name", get_optional_value(true));
    ///
    /// // Person without a name if not provided
    /// let person_without_name = Envelope::new("person")
    ///     .add_optional_assertion("name", get_optional_value(false));
    /// ```
    pub fn add_optional_assertion(
        &self,
        predicate: impl EnvelopeEncodable,
        object: Option<impl EnvelopeEncodable>
    ) -> Self {
        if let Some(object) = object {
            self.add_assertion_envelope(Self::new_assertion(predicate, object)).unwrap()
        } else {
            self.clone()
        }
    }

    /// Adds an assertion with the given predicate and string value, but only if the string is non-empty.
    ///
    /// This is a convenience method that only adds an assertion if the string value
    /// is non-empty. It's particularly useful when working with user input or optional
    /// text fields that should only be included if they contain actual content.
    ///
    /// # Parameters
    ///
    /// * `predicate` - The predicate for the assertion
    /// * `str` - The string value for the assertion
    ///
    /// # Returns
    ///
    /// A new envelope with the assertion added if the string is non-empty,
    /// or the original envelope if the string is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // Create a person with non-empty fields
    /// let person = Envelope::new("person")
    ///     .add_nonempty_string_assertion("name", "Alice Smith")
    ///     .add_nonempty_string_assertion("notes", ""); // This won't be added
    ///
    /// // Only the name assertion is added
    /// assert_eq!(person.assertions().len(), 1);
    /// ```
    pub fn add_nonempty_string_assertion(
        &self,
        predicate: impl EnvelopeEncodable,
        str: impl AsRef<str>
    ) -> Self {
        let str = str.as_ref();
        if str.is_empty() {
            self.clone()
        } else {
            self.add_assertion(predicate, str)
        }
    }

    /// Returns a new envelope with the given array of assertions added.
    ///
    /// Similar to `add_assertion_envelopes` but ignores any errors that might occur.
    /// This is useful when you're certain all envelopes in the array are valid
    /// assertion envelopes and don't need to handle errors.
    ///
    /// # Parameters
    ///
    /// * `envelopes` - An array of assertion envelopes to add
    ///
    /// # Returns
    ///
    /// A new envelope with all the valid assertions added
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // Create multiple assertion envelopes
    /// let assertion1 = Envelope::new_assertion("name", "Alice Smith");
    /// let assertion2 = Envelope::new_assertion("age", 30);
    ///
    /// // Add them all at once to an envelope
    /// let envelope = Envelope::new("person")
    ///     .add_assertions(&[assertion1, assertion2]);
    /// ```
    pub fn add_assertions(&self, envelopes: &[Self]) -> Self {
        let mut e = self.clone();
        for envelope in envelopes {
            e = e.add_assertion_envelope(envelope.clone()).unwrap();
        }
        e
    }
}

/// Support for adding conditional assertions.
///
/// These methods add assertions only when certain conditions are met.
impl Envelope {
    /// Adds an assertion only if the provided condition is true.
    ///
    /// This method allows for conditional inclusion of assertions based on a boolean condition.
    /// It's a convenient way to add assertions only in certain circumstances without
    /// requiring separate conditional logic.
    ///
    /// # Parameters
    ///
    /// * `condition` - Boolean that determines whether to add the assertion
    /// * `predicate` - The predicate for the assertion
    /// * `object` - The object value for the assertion
    ///
    /// # Returns
    ///
    /// A new envelope with the assertion added if the condition is true,
    /// or the original envelope if the condition is false.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// let is_verified = true;
    /// let is_expired = false;
    ///
    /// let document = Envelope::new("document")
    ///     .add_assertion_if(is_verified, "verified", true)   // This will be added
    ///     .add_assertion_if(is_expired, "expired", true);    // This won't be added
    ///
    /// // Only the verification assertion is present
    /// assert_eq!(document.assertions().len(), 1);
    /// ```
    pub fn add_assertion_if(
        &self,
        condition: bool,
        predicate: impl EnvelopeEncodable,
        object: impl EnvelopeEncodable
    ) -> Self {
        if condition { self.add_assertion(predicate, object) } else { self.clone() }
    }

    /// Adds an assertion envelope only if the provided condition is true.
    ///
    /// Similar to `add_assertion_if` but works with pre-constructed assertion envelopes.
    /// This is useful when you have already created an assertion envelope separately and
    /// want to conditionally add it.
    ///
    /// # Parameters
    ///
    /// * `condition` - Boolean that determines whether to add the assertion envelope
    /// * `assertion_envelope` - The assertion envelope to add
    ///
    /// # Returns
    ///
    /// A new envelope with the assertion added if the condition is true,
    /// or the original envelope if the condition is false.
    ///
    /// # Errors
    ///
    /// Returns `EnvelopeError::InvalidFormat` if the provided envelope is not a valid
    /// assertion envelope or an obscured variant and the condition is true.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// let assertion = Envelope::new_assertion("verified", true);
    /// let should_include = true;
    ///
    /// let document = Envelope::new("document")
    ///     .add_assertion_envelope_if(should_include, assertion)
    ///     .unwrap();
    /// ```
    pub fn add_assertion_envelope_if(
        &self,
        condition: bool,
        assertion_envelope: Self
    ) -> Result<Self> {
        if condition { self.add_assertion_envelope(assertion_envelope) } else { Ok(self.clone()) }
    }
}

#[cfg(feature = "salt")]
/// Support for adding assertions with salt.
///
/// Salting adds random data to an assertion to change its digest
/// while preserving semantic meaning. This is useful for decorrelation -
/// making it impossible to determine if two elided envelopes contain the
/// same assertion by comparing their digests.
impl Envelope {
    /// Returns the result of adding the given assertion to the envelope, optionally salting it.
    pub fn add_assertion_salted<P, O>(&self, predicate: P, object: O, salted: bool) -> Self
        where P: EnvelopeEncodable, O: EnvelopeEncodable
    {
        let assertion = Self::new_assertion(predicate, object);
        self.add_optional_assertion_envelope_salted(Some(assertion), salted).unwrap()
    }

    /// Returns the result of adding the given assertion to the envelope, optionally salting it.
    ///
    /// The assertion envelope must be a valid assertion envelope, or an
    /// obscured variant (elided, encrypted, compressed) of one.
    pub fn add_assertion_envelope_salted(
        &self,
        assertion_envelope: Self,
        salted: bool
    ) -> Result<Self> {
        self.add_optional_assertion_envelope_salted(Some(assertion_envelope), salted)
    }

    /// If the optional assertion is present, returns the result of adding it to
    /// the envelope, optionally salting it. Otherwise, returns the envelope unchanged.
    ///
    /// The assertion envelope must be a valid assertion envelope, or an
    /// obscured variant (elided, encrypted, compressed) of one.
    pub fn add_optional_assertion_envelope_salted(
        &self,
        assertion: Option<Self>,
        salted: bool
    ) -> Result<Self> {
        match assertion {
            Some(assertion) => {
                if !assertion.is_subject_assertion() && !assertion.is_subject_obscured() {
                    bail!(Error::InvalidFormat);
                }
                let envelope2 = if salted { assertion.add_salt() } else { assertion };

                match self.case() {
                    EnvelopeCase::Node { subject, assertions, .. } => {
                        if !assertions.iter().any(|a| a.digest() == envelope2.digest()) {
                            let mut assertions = assertions.clone();
                            assertions.push(envelope2);
                            Ok(Self::new_with_unchecked_assertions(subject.clone(), assertions))
                        } else {
                            Ok(self.clone())
                        }
                    }
                    _ => Ok(Self::new_with_unchecked_assertions(self.subject(), vec![envelope2])),
                }
            }
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
///
/// These methods allow for modifying an envelope by removing or replacing
/// existing assertions, while maintaining the envelope's immutability model.
impl Envelope {
    /// Returns a new envelope with the given assertion removed.
    ///
    /// Finds and removes an assertion matching the target assertion's digest.
    /// If the assertion doesn't exist, returns the same envelope unchanged.
    /// If removing the assertion would leave the envelope with no assertions,
    /// returns just the subject as a new envelope.
    ///
    /// # Parameters
    ///
    /// * `target` - The assertion envelope to remove
    ///
    /// # Returns
    ///
    /// A new envelope with the specified assertion removed if found,
    /// or the original envelope if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // Create an envelope with assertions
    /// let person = Envelope::new("Alice")
    ///     .add_assertion("name", "Alice Smith")
    ///     .add_assertion("age", 30);
    ///
    /// // Create the assertion to remove
    /// let name_assertion = Envelope::new_assertion("name", "Alice Smith");
    ///
    /// // Remove the name assertion
    /// let modified = person.remove_assertion(name_assertion);
    ///
    /// // The envelope now only has the age assertion
    /// assert_eq!(modified.assertions().len(), 1);
    /// ```
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

    /// Returns a new envelope with the given assertion replaced by a new one.
    ///
    /// This method removes the specified assertion and adds a new one in its place.
    /// If the targeted assertion does not exist, returns the same envelope with the
    /// new assertion added.
    ///
    /// # Parameters
    ///
    /// * `assertion` - The assertion envelope to replace
    /// * `new_assertion` - The new assertion envelope to add
    ///
    /// # Returns
    ///
    /// A new envelope with the assertion replaced if found,
    /// or the original envelope with the new assertion added if not found.
    ///
    /// # Errors
    ///
    /// Returns `EnvelopeError::InvalidFormat` if the new assertion is not a valid
    /// assertion envelope or an obscured variant.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // Create an envelope with assertions
    /// let person = Envelope::new("Alice")
    ///     .add_assertion("name", "Alice Smith")
    ///     .add_assertion("age", 30);
    ///
    /// // Create the assertion to replace and the new assertion
    /// let old_name = Envelope::new_assertion("name", "Alice Smith");
    /// let new_name = Envelope::new_assertion("name", "Alice Johnson");
    ///
    /// // Replace the name assertion
    /// let modified = person.replace_assertion(old_name, new_name).unwrap();
    /// ```
    pub fn replace_assertion(&self, assertion: Self, new_assertion: Self) -> Result<Self> {
        self.remove_assertion(assertion).add_assertion_envelope(new_assertion)
    }

    /// Returns a new envelope with its subject replaced by the provided one.
    ///
    /// This method preserves all assertions from the original envelope but
    /// applies them to a new subject. It effectively creates a new envelope
    /// with the provided subject and copies over all assertions from the current envelope.
    ///
    /// # Parameters
    ///
    /// * `subject` - The new subject for the envelope
    ///
    /// # Returns
    ///
    /// A new envelope with the new subject and all assertions from the original envelope.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bc_envelope::prelude::*;
    /// // Create an envelope for Alice
    /// let alice = Envelope::new("Alice")
    ///     .add_assertion("age", 30)
    ///     .add_assertion("city", "Boston");
    ///
    /// // Replace the subject to create an envelope for Bob with the same assertions
    /// let bob = alice.replace_subject(Envelope::new("Bob"));
    ///
    /// // Bob now has the same assertions
    /// assert_eq!(bob.extract_subject::<String>().unwrap(), "Bob");
    /// assert_eq!(bob.assertions().len(), 2);
    /// ```
    pub fn replace_subject(&self, subject: Self) -> Self {
        self.assertions()
            .into_iter()
            .fold(subject, |e, a| e.add_assertion_envelope(a).unwrap())
    }
}

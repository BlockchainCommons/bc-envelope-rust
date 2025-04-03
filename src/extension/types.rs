use anyhow::{bail, Result};
use bc_components::DigestProvider;

use crate::{Envelope, EnvelopeEncodable, EnvelopeError};
use crate::extension::{known_values, KnownValue};

/// # Type System for Gordian Envelopes
///
/// This module provides functionality for adding, querying, and verifying types within envelopes.
/// In Gordian Envelope, types are implemented using the special `'isA'` Known Value predicate,
/// which is semantically equivalent to the RDF `rdf:type` concept.
///
/// Type information enables:
/// - Semantic classification of envelopes
/// - Type verification before processing content
/// - Conversion between domain objects and envelopes
/// - Schema validation
///
/// ## Type Representation
///
/// Types are represented as assertions with the `'isA'` predicate (known value 1) and an object
/// that specifies the type. The type object is typically either:
///
/// 1. A Known Value from the registry (e.g., `known_values::SEED_TYPE`)
/// 2. A custom type represented as an envelope
///
/// ## Usage Patterns
///
/// The type system is commonly used in two ways:
///
/// 1. **Type Tagging**: Adding type information to envelopes to indicate their semantic meaning
///    ```
///    use bc_envelope::prelude::*;
///    
///    // Create an envelope representing a person
///    let person = Envelope::new("Alice")
///        .add_type("Person")
///        .add_assertion("age", 30);
///    ```
///
/// 2. **Type Checking**: Verifying that an envelope has the expected type before processing
///    ```no_run
///    use bc_envelope::prelude::*;
///    use anyhow::Result;
///    
///    fn process_person(envelope: &Envelope) -> Result<()> {
///        // Verify this is a person before processing
///        envelope.check_type_envelope("Person")?;
///        
///        // Now we can safely extract person-specific information
///        let name: String = envelope.subject().try_into()?;
///        let age = envelope.extract_object_for_predicate::<u8>("age")?;
///        
///        println!("{} is {} years old", name, age);
///        Ok(())
///    }
///    ```
///
/// ## Domain Object Conversion
///
/// The type system also enables conversion between domain objects and envelopes.
/// The pattern typically involves:
///
/// 1. Implementing `From<DomainObject> for Envelope` to convert objects to envelopes
/// 2. Implementing `TryFrom<Envelope> for DomainObject` to convert envelopes back to objects
/// 3. Using `check_type()` in the TryFrom implementation to verify the envelope has the correct type
///
/// See the `test_seed.rs` file in the tests directory for an example of this pattern.
impl Envelope {
    /// Adds a type assertion to the envelope using the `'isA'` predicate.
    ///
    /// This method provides a convenient way to declare the type of an envelope
    /// using the standard `'isA'` predicate (known value 1). The type can be any
    /// value that can be converted to an envelope, typically a string or a 
    /// Known Value from the registry.
    ///
    /// # Parameters
    /// - `object`: The type to assign to this envelope
    ///
    /// # Returns
    /// A new envelope with the type assertion added
    ///
    /// # Examples
    ///
    /// Using a string type:
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a document and declare its type
    /// let document = Envelope::new("Important Content")
    ///     .add_type("Document");
    ///
    /// // Verify the type was added
    /// assert!(document.has_type_envelope("Document"));
    /// ```
    ///
    /// Using a predefined Known Value type:
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a seed envelope with the standard SEED_TYPE
    /// let seed_data = "seed data".to_string();
    /// let seed = Envelope::new(seed_data)
    ///     .add_type(known_values::SEED_TYPE);
    ///
    /// // Verify the type was added
    /// assert!(seed.has_type(&known_values::SEED_TYPE));
    /// ```
    pub fn add_type(&self, object: impl EnvelopeEncodable) -> Self {
        self.add_assertion(known_values::IS_A, object)
    }

    /// Returns all type objects from the envelope's `'isA'` assertions.
    ///
    /// This method retrieves all objects of assertions that use the `'isA'` predicate.
    /// Each returned envelope represents a type that has been assigned to this envelope.
    ///
    /// # Returns
    /// A vector of envelopes, each representing a type assigned to this envelope
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use std::convert::TryInto;
    ///
    /// // Create an envelope with multiple types
    /// let multi_typed = Envelope::new("Versatile Entity")
    ///     .add_type("Person")
    ///     .add_type("Employee")
    ///     .add_type("Manager");
    ///
    /// // Get all the type objects
    /// let types = multi_typed.types();
    ///
    /// // There should be 3 types
    /// assert_eq!(types.len(), 3);
    ///
    /// // For each type, we could verify its presence directly using digests
    /// let person_type = Envelope::new("Person");
    /// let employee_type = Envelope::new("Employee");
    /// let manager_type = Envelope::new("Manager");
    /// 
    /// let has_person = types.iter().any(|e| e.digest() == person_type.digest());
    /// let has_employee = types.iter().any(|e| e.digest() == employee_type.digest());
    /// let has_manager = types.iter().any(|e| e.digest() == manager_type.digest());
    ///
    /// assert!(has_person);
    /// assert!(has_employee);
    /// assert!(has_manager);
    /// ```
    pub fn types(&self) -> Vec<Self> {
        self.objects_for_predicate(known_values::IS_A)
    }

    /// Gets a single type object from the envelope's `'isA'` assertions.
    ///
    /// This method is useful when an envelope is expected to have exactly one type.
    /// It returns an error if the envelope has zero or multiple types.
    ///
    /// # Returns
    /// - `Ok(Envelope)`: The single type object if exactly one exists
    /// - `Err(EnvelopeError::AmbiguousType)`: If multiple types exist
    ///
    /// # Examples
    ///
    /// With a single type:
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an envelope with a single type
    /// let person = Envelope::new("Alice")
    ///     .add_type("Person");
    ///
    /// // Get the type
    /// let type_obj = person.get_type().unwrap();
    /// let type_string: String = type_obj.try_into().unwrap();
    /// assert_eq!(type_string, "Person");
    /// ```
    ///
    /// With multiple types (results in error):
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create an envelope with multiple types
    /// let multi_typed = Envelope::new("Alice")
    ///     .add_type("Person")
    ///     .add_type("Employee");
    ///
    /// // Trying to get a single type will fail
    /// let result = multi_typed.get_type();
    /// assert!(result.is_err());
    /// ```
    pub fn get_type(&self) -> Result<Self> {
        let t = self.types();
        if t.len() == 1 {
            Ok(t[0].clone())
        } else {
            bail!(EnvelopeError::AmbiguousType)
        }
    }

    /// Checks if the envelope has a specific type, using an envelope as the type identifier.
    ///
    /// This method compares the digest of each type object with the digest of the provided
    /// envelope to determine if the envelope has the specified type.
    ///
    /// # Parameters
    /// - `t`: The type to check for, which will be converted to an envelope
    ///
    /// # Returns
    /// `true` if the envelope has the specified type, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a typed envelope
    /// let document = Envelope::new("Contract")
    ///     .add_type("LegalDocument")
    ///     .add_assertion("status", "Draft");
    ///
    /// // Check for various types
    /// assert!(document.has_type_envelope("LegalDocument"));
    /// assert!(!document.has_type_envelope("Spreadsheet"));
    ///
    /// // Can also check with an envelope
    /// let legal_doc_type = Envelope::new("LegalDocument");
    /// assert!(document.has_type_envelope(legal_doc_type));
    /// ```
    pub fn has_type_envelope(&self, t: impl EnvelopeEncodable) -> bool {
        let e = t.into_envelope();
        self.types().iter().any(|x| x.digest() == e.digest())
    }

    /// Checks if the envelope has a specific type, using a Known Value as the type identifier.
    ///
    /// Similar to `has_type_envelope`, but specifically for checking against standard
    /// Known Value types from the registry.
    ///
    /// # Parameters
    /// - `t`: The Known Value type to check for
    ///
    /// # Returns
    /// `true` if the envelope has the specified Known Value type, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    ///
    /// // Create a seed envelope
    /// let seed_data = "seed data".to_string();
    /// let seed = Envelope::new(seed_data)
    ///     .add_type(known_values::SEED_TYPE);
    ///
    /// // Check the type using the Known Value
    /// assert!(seed.has_type(&known_values::SEED_TYPE));
    /// assert!(!seed.has_type(&known_values::PRIVATE_KEY_TYPE));
    /// ```
    pub fn has_type(&self, t: &KnownValue) -> bool {
        let type_envelope: Envelope = t.clone().to_envelope();
        self.types().iter().any(|x| x.digest() == type_envelope.digest())
    }

    /// Verifies that the envelope has a specific Known Value type.
    ///
    /// This method is similar to `has_type` but returns a Result, making it
    /// suitable for use in validation chains with the `?` operator.
    ///
    /// # Parameters
    /// - `t`: The Known Value type to check for
    ///
    /// # Returns
    /// - `Ok(())`: If the envelope has the specified type
    /// - `Err(EnvelopeError::InvalidType)`: If the envelope does not have the specified type
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use anyhow::Result;
    ///
    /// // Function that processes a seed envelope
    /// fn process_seed(envelope: &Envelope) -> Result<String> {
    ///     // Verify this is a seed
    ///     envelope.check_type(&known_values::SEED_TYPE)?;
    ///     
    ///     // Extract the seed data
    ///     let seed_data: String = envelope.subject().try_into()?;
    ///     Ok(seed_data)
    /// }
    ///
    /// // Create a seed envelope
    /// let seed_data = "seed data".to_string();
    /// let seed = Envelope::new(seed_data.clone())
    ///     .add_type(known_values::SEED_TYPE);
    ///
    /// // Process the seed
    /// let result = process_seed(&seed);
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap(), seed_data);
    ///
    /// // Create a non-seed envelope
    /// let not_a_seed = Envelope::new("Not a seed")
    ///     .add_type("SomethingElse");
    ///
    /// // Processing should fail
    /// let result = process_seed(&not_a_seed);
    /// assert!(result.is_err());
    /// ```
    pub fn check_type(&self, t: &KnownValue) -> Result<()> {
        if self.has_type(t) {
            Ok(())
        } else {
            bail!(EnvelopeError::InvalidType)
        }
    }

    /// Verifies that the envelope has a specific type, using an envelope as the type identifier.
    ///
    /// This method is similar to `has_type_envelope` but returns a Result, making it
    /// suitable for use in validation chains with the `?` operator.
    ///
    /// # Parameters
    /// - `t`: The type to check for, which will be converted to an envelope
    ///
    /// # Returns
    /// - `Ok(())`: If the envelope has the specified type
    /// - `Err(EnvelopeError::InvalidType)`: If the envelope does not have the specified type
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_envelope::prelude::*;
    /// use anyhow::Result;
    ///
    /// // Function that processes a person
    /// fn process_person(envelope: &Envelope) -> Result<String> {
    ///     // Verify this is a person
    ///     envelope.check_type_envelope("Person")?;
    ///     
    ///     // Extract the name
    ///     let name: String = envelope.subject().try_into()?;
    ///     Ok(name)
    /// }
    ///
    /// // Create a person envelope
    /// let person = Envelope::new("Alice")
    ///     .add_type("Person");
    ///
    /// // Process the person
    /// let result = process_person(&person);
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap(), "Alice");
    ///
    /// // Create a non-person envelope
    /// let document = Envelope::new("Contract")
    ///     .add_type("Document");
    ///
    /// // Processing should fail
    /// let result = process_person(&document);
    /// assert!(result.is_err());
    /// ```
    pub fn check_type_envelope(&self, t: impl EnvelopeEncodable) -> Result<()> {
        if self.has_type_envelope(t) {
            Ok(())
        } else {
            bail!(EnvelopeError::InvalidType)
        }
    }
}

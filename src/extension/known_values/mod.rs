//! Known Values extension for Gordian Envelope.
//!
//! This module implements the Known Values extension for Gordian Envelope as defined in
//! [BCR-2023-003](https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-003-envelope-known-value.md).
//! Known Values provide a compact, deterministic way to represent ontological concepts as unique
//! 64-bit unsigned integers, particularly useful as predicates in Envelope assertions.
//!
//! # Components
//!
//! The Known Values module consists of three main components:
//!
//! - [`KnownValue`] - The core type representing a known value with its numeric value and optional name
//! - [`KnownValuesStore`] - A store mapping between known values and their names
//! - Registry - A predefined set of standard known values exposed as constants (`IS_A`, `NOTE`, etc.)
//!
//! # Usage
//!
//! Known Values can be used in Envelopes in any position (subject, predicate, or object),
//! but are most commonly used as predicates in assertions:
//!
//! ```
//! use bc_envelope::prelude::*;
//!
//! // Create an envelope with a known value as predicate
//! let envelope = Envelope::new("Alice")
//!     .add_assertion(known_values::IS_A, "Person");
//!
//! // The known value will be displayed with its name in formatted output
//! assert_eq!(envelope.format_flat(), r#""Alice" [ 'isA': "Person" ]"#);
//!
//! // Extract the assertion with the known value predicate
//! let assertion = envelope.assertion_with_predicate(known_values::IS_A).unwrap();
//! assert_eq!(assertion.extract_object::<String>().unwrap(), "Person");
//! ```
//!
//! # Registry
//!
//! The module provides a registry of predefined known values as constants,
//! each with a unique numeric value and canonical name. These constants are
//! exposed directly in the `known_values` namespace for easy access.
//!
//! ```
//! use bc_envelope::prelude::*;
//!
//! // Standard known values from the registry
//! assert_eq!(known_values::IS_A.value(), 1);
//! assert_eq!(known_values::NOTE.value(), 4);
//! assert_eq!(known_values::SIGNED.value(), 3);
//! ```
//!
//! # Custom Known Values
//!
//! While the module provides a standard registry of known values, you can also
//! create custom known values for your application:
//!
//! ```
//! use bc_envelope::prelude::*;
//! use bc_envelope::known_value_constant;
//! use paste::paste;
//!
//! // Define a custom known value
//! known_value_constant!(MY_CUSTOM_VALUE, 1000, "myCustomValue");
//!
//! // Use it like any other known value
//! let envelope = Envelope::new("Document123")
//!     .add_assertion(MY_CUSTOM_VALUE, "Important");
//! ```

pub mod known_value;
pub use known_value::KnownValue;

pub mod known_values_registry;
pub use known_values_registry as registry;
pub use registry::*;

pub mod known_values_store;
pub use known_values_store::KnownValuesStore;

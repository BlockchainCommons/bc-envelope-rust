pub mod assertion;
pub use assertion::Assertion;

pub mod assertions;
pub mod cbor;
pub mod digest;

pub mod envelope;
pub use envelope::{Envelope, EnvelopeCase};

/// Types dealing with elision.
///
/// Actual functions for elision are on the [`Envelope`] type itself.
pub mod elide;

pub mod error;
pub use error::{Error, Result};

pub mod envelope_encodable;
pub use envelope_encodable::EnvelopeEncodable;

pub mod envelope_decodable;

pub mod queries;

/// Types dealing with recursive walking of envelopes.
///
/// The [`Envelope`] type itself has functions for walking envelopes.
pub mod walk;

pub mod wrap;

pub mod leaf;

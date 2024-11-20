pub mod assertion;
pub mod assertions;
pub mod cbor;
pub mod digest;
pub mod envelope;

/// Types dealing with elision.
///
/// Actual functions for elision are on the [`Envelope`] type itself.
pub mod elide;

pub mod error;

pub mod envelope_encodable;
pub use envelope_encodable::EnvelopeEncodable;

pub mod envelope_decodable;

pub mod queries;

/// Types dealing with formatting envelopes.
pub mod format;
pub mod format_context;
pub use format_context::*;
pub mod tree_format;

/// Types dealing with recursive walking of envelopes.
///
/// The [`Envelope`] type itself has functions for walking envelopes.
pub mod walk;

pub mod wrap;
pub mod envelope_summary;

pub use assertion::Assertion;
pub use envelope::Envelope;
pub use error::EnvelopeError;
pub use format_context::{FormatContext, GLOBAL_FORMAT_CONTEXT};
pub use envelope_summary::EnvelopeSummary;

pub mod signature;
pub mod compress;
pub mod salt;
pub mod encrypt;

/// Types dealing with elision.
///
/// Actual functions for elision are on the [`Envelope`] type itself.
pub mod elide;

pub mod recipient;

/// Types dealing with SSKR splitting.
pub mod sskr;

pub mod known_values;
pub use known_values::*;

/// Types dealing with envelope expressions (and distributed function evaluation)
pub mod expressions;

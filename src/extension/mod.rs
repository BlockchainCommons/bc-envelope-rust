pub mod signature;
pub mod compress;

#[cfg(feature = "salt")]
pub mod salt;

pub mod encrypt;

/// Types dealing with elision.
///
/// Actual functions for elision are on the [`Envelope`] type itself.
pub mod elide;

pub mod recipient;

#[cfg(feature = "sskr")]
/// Types dealing with SSKR splitting.
pub mod sskr;

pub mod known_values;
pub use known_values::*;

#[cfg(feature = "expressions")]
/// Types dealing with envelope expressions (and distributed function evaluation)
pub mod expressions;

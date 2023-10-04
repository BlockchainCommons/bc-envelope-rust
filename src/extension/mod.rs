/// Public key signing and verification.
#[cfg(feature = "signature")]
pub mod signature;

/// Compression.
#[cfg(feature = "compress")]
pub mod compress;

/// Decorrelation using salt.
#[cfg(feature = "salt")]
pub mod salt;

/// Symmetric key encryption.
#[cfg(feature = "encrypt")]
pub mod encrypt;

/// Public key encryption.
#[cfg(feature = "recipient")]
pub mod recipient;

/// Types dealing with SSKR splitting.
#[cfg(feature = "sskr")]
pub mod sskr;

#[cfg(feature = "known_value")]
pub mod known_values;
#[cfg(feature = "known_value")]
pub use known_values::*;

#[cfg(feature = "expression")]
/// Types dealing with envelope expressions (and distributed function evaluation)
pub mod expression;

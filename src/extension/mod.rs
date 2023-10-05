/// Attachments
#[cfg(feature = "attachment")]
pub mod attachment;

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

/// Types dealing with Known Values.
#[cfg(feature = "known_value")]
pub mod known_values;
#[cfg(feature = "known_value")]
pub use known_values::*;

/// Types dealing with envelope expressions (and distributed function evaluation)
#[cfg(feature = "expression")]
pub mod expression;

/// Working with type ('isA') assertions.
#[cfg(feature = "types")]
pub mod types;

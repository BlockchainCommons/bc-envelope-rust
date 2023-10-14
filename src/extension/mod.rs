///
/// Attachments Extension
///
#[cfg(feature = "attachment")]
pub mod attachment;

///
/// Compression Extension
///
#[cfg(feature = "compress")]
pub mod compress;

///
/// Symmetric Encryption Extension
///
#[cfg(feature = "encrypt")]
pub mod encrypt;

///
/// Expressions Extension
///
/// Includes distributed function evaluation.
///
#[cfg(feature = "expression")]
pub mod expression;

///
/// Known Values Extension
///
#[cfg(feature = "known_value")]
pub mod known_values;
#[cfg(feature = "known_value")]
pub use known_values::*;

///
/// Inclusion Proof Extension
/// 
#[cfg(feature = "proof")]
pub mod proof;

///
/// Public Key Encryption Extension
///
#[cfg(feature = "recipient")]
pub mod recipient;

///
/// Public Key Signing Extension
///
#[cfg(feature = "signature")]
pub mod signature;

///
/// Salt Extension
///
#[cfg(feature = "salt")]
pub mod salt;

///
/// SSKR Extension
///
#[cfg(feature = "sskr")]
pub mod sskr;

///
/// Types Extension
///
#[cfg(feature = "types")]
pub mod types;

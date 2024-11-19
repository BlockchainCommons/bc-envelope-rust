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
#[cfg(feature = "expression")]
pub mod expressions;
#[cfg(feature = "expression")]
pub use expressions::{
    Expression,
    ExpressionBehavior,
    IntoExpression,
    Request,
    RequestBehavior,
    Response,
    ResponseBehavior,
};

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
/// SSH Keys Extension
///
#[cfg(feature = "ssh")]
pub mod ssh;

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

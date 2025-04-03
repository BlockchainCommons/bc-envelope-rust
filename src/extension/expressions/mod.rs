//! # Envelope Expressions
//!
//! This module provides functionality for creating, manipulating, and evaluating
//! expressions encoded in Gordian Envelopes.
//!
//! Envelope Expressions are a method for encoding machine-evaluatable expressions
//! using Gordian Envelope. Using expressions, envelope provides a cryptographically 
//! strong foundation on which to build requests and responses for distributed function calls.
//!
//! ## Overview
//!
//! An expression in Gordian Envelope consists of:
//!
//! 1. A function identifier (the subject of the envelope)
//! 2. Zero or more parameters (assertions on the envelope)
//!
//! Functions and parameters are CBOR-tagged to distinguish them from regular envelope content:
//! - Functions are tagged with #6.40006
//! - Parameters are tagged with #6.40007
//!
//! ## Expression Structure
//!
//! In envelope notation, expressions look like this:
//!
//! ```text
//! «function» [
//!     ❰parameter❱: argument
//!     ❰parameter❱: argument
//!     ...
//! ]
//! ```
//!
//! When printed, function identifiers are enclosed in double angle brackets (`«` and `»`),
//! and parameter identifiers are enclosed in heavy single angle brackets (`❰` and `❱`).
//!
//! ## Examples
//!
//! A simple addition expression:
//!
//! ```text
//! «add» [
//!     ❰lhs❱: 2
//!     ❰rhs❱: 3
//! ]
//! ```
//!
//! A function call with named parameters:
//!
//! ```text
//! «"verifySignature"» [
//!     ❰"key"❱: SigningPublicKey
//!     ❰"sig"❱: Signature
//!     ❰"digest"❱: Digest
//! ]
//! ```
//!
//! ## Function and Parameter Identifiers
//!
//! Both function and parameter identifiers can be:
//! - Numeric: For well-known, standardized functions and parameters
//! - String: For application-specific or less common functions and parameters
//!
//! ## Composition
//!
//! Expressions can be composed, with one expression being a parameter to another:
//!
//! ```text
//! «"verifySignature"» [
//!     ❰"key"❱: SigningPublicKey
//!     ❰"sig"❱: Signature
//!     ❰"digest"❱: «"sha256"» [
//!         ❰_❱: messageData
//!     ]
//! ]
//! ```

mod function;
pub use function::Function;

mod functions_store;
pub use functions_store::FunctionsStore;

/// A collection of functions that can be used in envelope expressions.
pub mod functions;
pub use functions::*;

mod parameter;
pub use parameter::Parameter;

mod parameters_store;
pub use parameters_store::ParametersStore;

/// A collection of known values that can be used in envelope expressions.
pub mod parameters;
pub use parameters::*;

// mod expression_impl;

pub mod expression;
pub use expression::{
    Expression,
    ExpressionBehavior,
    IntoExpression,
};

pub mod request;
pub use request::{
    Request,
    RequestBehavior,
};

pub mod response;
pub use response::{
    Response,
    ResponseBehavior,
};

pub mod event;
pub use event::{
    Event,
    EventBehavior,
};

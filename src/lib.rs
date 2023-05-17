#![feature(closure_lifetime_binder)]

mod assertions;
mod queries;
mod digest;
mod salt;
mod format;
mod cbor;

pub mod parameter_registry;
pub mod function_registry;

pub mod known_value_registry;
pub use known_value_registry::KNOWN_VALUES;

mod envelope;
pub use crate::envelope::Envelope;

mod format_context;
pub use format_context::FormatContext;

mod envelope_error;
pub use envelope_error::EnvelopeError;

mod walk;
pub use walk::{EdgeType, Visitor};

mod assertion;
pub use assertion::Assertion;

mod known_value;
pub use known_value::KnownValue;

mod known_values;
pub use known_values::KnownValues;

mod function;
pub use function::Function;

mod known_functions;
pub use known_functions::KnownFunctions;

mod parameter;
pub use parameter::Parameter;

mod known_parameters;
pub use known_parameters::KnownParameters;

#[cfg(test)]
mod tests {
    use bc_crypto::sha256;

    #[test]
    fn it_works() {
        let input = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let expected = "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1";
        let result = hex::encode(sha256(input.as_bytes()));
        assert_eq!(result, expected);
    }
}

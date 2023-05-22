#![feature(iter_intersperse)]

mod assertions;
mod queries;
mod digest;
mod salt;
mod format;
mod cbor;
mod string_utils;
mod known_value_registry;
mod parameter_registry;
mod function_registry;

mod envelope;
pub use crate::envelope::Envelope;

mod format_context;
pub use format_context::{FormatContext, FORMAT_CONTEXT};

mod envelope_error;
pub use envelope_error::EnvelopeError;

mod walk;
pub use walk::{EdgeType, Visitor};

mod assertion;
pub use assertion::Assertion;

pub mod known_value;
pub use known_value::KnownValue;

pub mod function;
pub use function::Function;

pub mod parameter;
pub use parameter::Parameter;

mod known_values;
pub use known_values::KnownValues;

mod known_functions;
pub use known_functions::KnownFunctions;

mod known_parameters;
pub use known_parameters::KnownParameters;

#[cfg(test)]
mod tests {
    mod type_tests;
    mod core_tests;

    use bc_crypto::sha256;

    #[test]
    fn it_works() {
        let input = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let expected = "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1";
        let result = hex::encode(sha256(input.as_bytes()));
        assert_eq!(result, expected);
    }
}

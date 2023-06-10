#![feature(iter_intersperse)]

mod assertions;
mod cbor;
mod compress;
mod digest;
mod encrypt;
mod function_registry;
mod known_value_registry;
mod parameter_registry;
mod queries;
mod recipient;
mod salt;
mod signature;
mod string_utils;
mod tree_format;
mod wrap;

mod format;
pub use format::{EnvelopeFormat, EnvelopeFormatItem};

mod sskr;
pub use sskr::{SSKRShare, SSKRSpec, SSKRGroupSpec, SSKRSecret};

mod enclose;
pub use enclose::Enclosable;

mod elide;
pub use elide::ObscureAction;

mod envelope;
pub use crate::envelope::Envelope;

mod format_context;
pub use format_context::{FormatContext, FORMAT_CONTEXT};

mod error;
pub use error::Error;

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
    pub mod test_data;
    mod seed;
    pub use seed::Seed;

    mod check_encoding;
    mod compression_tests;
    mod core_encoding_tests;
    mod core_nesting_tests;
    mod core_tests;
    mod crypto_tests;
    mod elision_tests;
    mod encrypted_tests;
    mod format_tests;
    mod non_correlation_tests;
    mod obscuring_tests;
    mod type_tests;

    use bc_crypto::hash::sha256;

    #[test]
    fn it_works() {
        let input = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let expected = "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1";
        let result = hex::encode(sha256(input.as_bytes()));
        assert_eq!(result, expected);
    }
}

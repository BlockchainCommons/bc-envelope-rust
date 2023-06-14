#![doc(html_root_url = "https://docs.rs/bc-envelope/0.1.0")]
#![warn(rust_2018_idioms)]
#![feature(iter_intersperse)]

mod assertions;
mod cbor;
mod compress;
mod digest;
mod encrypt;
mod expression;
mod functions;
mod known_values;
mod parameters;
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

mod into_envelope;
pub use into_envelope::IntoEnvelope;

mod elide;
pub use elide::ObscureAction;

mod envelope;
pub use crate::envelope::Envelope;

mod format_context;
pub use format_context::{FormatContext, GLOBAL_FORMAT_CONTEXT};

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

mod known_values_store;
pub use known_values_store::KnownValuesStore;

mod functions_store;
pub use functions_store::FunctionsStore;

mod parameters_store;
pub use parameters_store::ParametersStore;

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
    mod function_tests;
    mod non_correlation_tests;
    mod obscuring_tests;
    mod type_tests;

    #[test]
    fn test_readme_deps() {
        version_sync::assert_markdown_deps_updated!("README.md");
    }

    #[test]
    fn test_html_root_url() {
        version_sync::assert_html_root_url_updated!("src/lib.rs");
    }
}

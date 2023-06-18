#![doc(html_root_url = "https://docs.rs/bc-envelope/0.1.0")]
#![warn(rust_2018_idioms)]
#![feature(iter_intersperse)]

mod assertions;
mod cbor;
mod compress;
mod digest;
mod encrypt;
mod queries;
mod recipient;
mod salt;
mod signature;
mod string_utils;
mod tree_format;
mod wrap;

pub mod expressions;
pub mod format;
pub mod known_values;
pub mod sskr;
pub mod walk;
pub mod elide;

mod into_envelope;
pub use into_envelope::IntoEnvelope;

mod envelope;
pub use crate::envelope::Envelope;

mod error;
pub use error::Error;

mod assertion;
pub use assertion::Assertion;

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
    mod expressions_tests;
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

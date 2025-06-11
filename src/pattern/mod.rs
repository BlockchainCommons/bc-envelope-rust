// Pattern module - provides pattern matching functionality for envelopes
mod matcher;
mod pattern;

mod and_pattern;
mod assertions_pattern;
mod bool_pattern;
mod leaf_pattern;
mod number_pattern;
mod or_pattern;
mod text_pattern;
mod wrapped_pattern;
mod predicate_pattern;
mod object_pattern;

// mod array_pattern;
// mod byte_string_pattern;
// mod digest_pattern;
// mod known_value_pattern;
// mod map_pattern;
// mod node_pattern;
// mod obscured_pattern;
// mod repeat_pattern;
// mod tag_pattern;

// Re-export all types
pub use and_pattern::AndPattern;
pub use assertions_pattern::AssertionsPattern;
pub use bool_pattern::BoolPattern;
pub use leaf_pattern::LeafPattern;
pub use matcher::{Matcher, Path};
pub use number_pattern::NumberPattern;
pub use or_pattern::OrPattern;
pub use pattern::Pattern;
pub use text_pattern::TextPattern;
pub use wrapped_pattern::WrappedPattern;
pub use predicate_pattern::PredicatePattern;
pub use object_pattern::ObjectPattern;

// pub use array_pattern::ArrayPattern;
// pub use byte_string_pattern::ByteStringPattern;
// pub use digest_pattern::DigestPattern;
// pub use known_value_pattern::KnownValuePattern;
// pub use map_pattern::MapPattern;
// pub use node_pattern::NodePattern;
// pub use obscured_pattern::ObscuredPattern;
// pub use repeat_pattern::RepeatPattern;
// pub use tag_pattern::TagPattern;

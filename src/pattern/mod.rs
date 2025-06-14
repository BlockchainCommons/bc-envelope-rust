// Pattern module - provides pattern matching functionality for envelopes
mod matcher;
mod pattern_impl;
mod greediness;
mod vm;
mod atomic;

// Subdirectory modules
mod leaf;
mod meta;
mod structure;

// Re-export greediness
pub use greediness::Greediness;

// Re-export all types
pub use matcher::{Matcher, Path};
pub use pattern_impl::Pattern;

// Individual pattern types are pub(crate) - not re-exported
// pub use leaf::{BoolPattern, LeafPattern, NumberPattern, TextPattern};
// pub use meta::{AndPattern, OrPattern, SearchPattern, SequencePattern};
// pub use structure::{AssertionsPattern, ObjectPattern, PredicatePattern,
// SubjectPattern, WrappedPattern};

// pub use structure::{DigestPattern, NodePattern, ObscuredPattern};
// pub use leaf::{ArrayPattern, ByteStringPattern, CborPattern,
// KnownValuePattern, MapPattern, NullPattern, TagPattern};
// pub use meta::{NotPattern, RepeatPattern};

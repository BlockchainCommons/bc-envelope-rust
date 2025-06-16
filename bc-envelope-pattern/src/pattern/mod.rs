// Pattern module - provides pattern matching functionality for envelopes
mod compile;
mod greediness;
mod matcher;
mod pattern_impl;
mod vm;

// Subdirectory modules
mod leaf;
mod meta;
mod structure;

// Re-export all types
pub use compile::{Compilable, compile_as_atomic};
pub use greediness::Greediness;
pub use matcher::{Matcher, Path};
pub use pattern_impl::Pattern;

// Meta patterns - patterns for combining and modifying other patterns

mod and_pattern;
mod or_pattern;
mod search_pattern;
mod sequence_pattern;

// Uncommented modules are not yet implemented
// mod not_pattern;
// mod repeat_pattern;

pub(crate) use and_pattern::AndPattern;
pub(crate) use or_pattern::OrPattern;
pub(crate) use search_pattern::SearchPattern;
pub(crate) use sequence_pattern::SequencePattern;

// pub(crate) use not_pattern::NotPattern;
// pub(crate) use repeat_pattern::RepeatPattern;

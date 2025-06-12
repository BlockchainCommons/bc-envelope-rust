// Meta patterns - patterns for combining and modifying other patterns

pub(crate) mod and_pattern;
pub(crate) mod or_pattern;
pub(crate) mod search_pattern;
pub(crate) mod sequence_pattern;

// Uncommented modules are not yet implemented
// pub(crate) mod not_pattern;
// pub(crate) mod repeat_pattern;

pub(crate) use and_pattern::AndPattern;
pub(crate) use or_pattern::OrPattern;
pub(crate) use search_pattern::SearchPattern;
pub(crate) use sequence_pattern::SequencePattern;

// pub(crate) use not_pattern::NotPattern;
// pub(crate) use repeat_pattern::RepeatPattern;

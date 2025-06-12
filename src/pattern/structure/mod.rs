// Structure patterns - patterns dealing with envelope structure

pub(crate) mod assertions_pattern;
pub(crate) mod object_pattern;
pub(crate) mod predicate_pattern;
pub(crate) mod subject_pattern;
pub(crate) mod wrapped_pattern;

// Uncommented modules are not yet implemented
// pub(crate) mod digest_pattern;
// pub(crate) mod node_pattern;
// pub(crate) mod obscured_pattern;

pub(crate) use assertions_pattern::AssertionsPattern;
pub(crate) use object_pattern::ObjectPattern;
pub(crate) use predicate_pattern::PredicatePattern;
pub(crate) use subject_pattern::SubjectPattern;
pub(crate) use wrapped_pattern::WrappedPattern;

// pub(crate) use digest_pattern::DigestPattern;
// pub(crate) use node_pattern::NodePattern;
// pub(crate) use obscured_pattern::ObscuredPattern;

use std::ops::RangeInclusive;

use crate::{
    Envelope, Pattern,
    pattern::{
        Compilable, Matcher, Path, compile_as_atomic, leaf::LeafPattern,
        vm::Instr,
    },
};

/// Pattern for matching maps.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum MapPattern {
    /// Matches any map.
    Any,
    /// Matches maps with a specific count of entries.
    Count(RangeInclusive<usize>),
}

impl MapPattern {
    /// Creates a new `MapPattern` that matches any map.
    pub fn any() -> Self { MapPattern::Any }

    /// Creates a new `MapPattern` that matches maps with a specific count of
    /// entries.
    pub fn range_count(range: RangeInclusive<usize>) -> Self {
        MapPattern::Count(range)
    }

    /// Creates a new `MapPattern` that matches maps with exactly the specified
    /// count of entries.
    pub fn count(count: usize) -> Self { MapPattern::Count(count..=count) }
}

impl Matcher for MapPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        if let Some(map) = envelope.subject().as_map() {
            match self {
                MapPattern::Any => vec![vec![envelope.clone()]],
                MapPattern::Count(range) => {
                    if range.contains(&map.len()) {
                        vec![vec![envelope.clone()]]
                    } else {
                        vec![]
                    }
                }
            }
        } else {
            vec![]
        }
    }
}

impl Compilable for MapPattern {
    fn compile(&self, code: &mut Vec<Instr>, literals: &mut Vec<Pattern>) {
        compile_as_atomic(
            &Pattern::Leaf(LeafPattern::Map(self.clone())),
            code,
            literals,
        );
    }
}

#[cfg(test)]
mod tests {
    use dcbor::prelude::*;

    use super::*;
    use bc_envelope::Envelope;

    #[test]
    fn test_map_pattern_any() {
        // Create a CBOR map directly
        let mut cbor_map = Map::new();
        cbor_map.insert("key1", "value1");
        cbor_map.insert("key2", "value2");
        let envelope = Envelope::new(cbor_map);

        let pattern = MapPattern::any();
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![envelope.clone()]);

        // Test with non-map envelope
        let text_envelope = Envelope::new("test");
        let paths = pattern.paths(&text_envelope);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_map_pattern_count() {
        // Create a CBOR map directly
        let mut cbor_map = Map::new();
        cbor_map.insert("key1", "value1");
        cbor_map.insert("key2", "value2");
        let envelope = Envelope::new(cbor_map);

        // Test exact count
        let pattern = MapPattern::count(2);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test count range
        let pattern = MapPattern::range_count(1..=3);
        let paths = pattern.paths(&envelope);
        assert_eq!(paths.len(), 1);

        // Test count mismatch
        let pattern = MapPattern::count(5);
        let paths = pattern.paths(&envelope);
        assert!(paths.is_empty());
    }
}

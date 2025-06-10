use std::ops::RangeInclusive;

use bc_components::{Digest, DigestProvider};
use dcbor::prelude::*;
use known_values::KnownValue;

use crate::Envelope;

pub trait MatchPattern {
    /// Checks if the envelope matches the pattern.
    fn matches(&self, envelope: &Envelope) -> bool;
}

#[derive(Debug, Clone)]
pub enum DigestPattern {
    /// Matches the exact digest.
    Digest(Digest),
    /// Matches the hexadecimal prefix of a digest.
    HexPrefix(String),
    /// Matches the binary regular expression for a digest.
    BinaryRegex(regex::bytes::Regex),
}

impl MatchPattern for DigestPattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        let digest = envelope.digest();
        match self {
            DigestPattern::Digest(digest) => *digest == *digest,
            DigestPattern::HexPrefix(prefix) => {
                hex::encode(digest.as_ref()).starts_with(prefix)
            }
            DigestPattern::BinaryRegex(regex) => regex.is_match(digest.as_bytes()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodePattern {
    /// Matches any node.
    Any,
    /// Matches a node with the specified count of assertions.
    AssertionsCount(RangeInclusive<usize>),
}

impl MatchPattern for NodePattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        if !envelope.is_node() {
            return false;
        }
        match self {
            NodePattern::Any => true,
            NodePattern::AssertionsCount(range) => {
                range.contains(&envelope.assertions().len())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum NumberPattern {
    /// Matches any number.
    Any,
    /// Matches the specific number.
    Exact(f64),
    /// Matches numbers within a range.
    Range(RangeInclusive<f64>),
    /// Matches numbers that are NaN (Not a Number).
    NaN,
}

impl MatchPattern for NumberPattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        let envelope = envelope.subject();
        match self {
            NumberPattern::Any => envelope.is_number(),
            NumberPattern::Exact(value) => {
                envelope.extract_subject().ok() == Some(*value)
            }
            NumberPattern::Range(range) => envelope
                .extract_subject()
                .map_or(false, |n| range.contains(&n)),
            NumberPattern::NaN => {
                envelope.extract_subject().map_or(false, |n: f64| n.is_nan())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TextPattern {
    /// Matches any text.
    Any,
    /// Matches the specific text.
    Exact(String),
    /// Matches the regex for a text.
    Regex(regex::Regex),
}

impl MatchPattern for TextPattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        if let Ok(s) = envelope.extract_subject::<String>() {
            match self {
                TextPattern::Any => true,
                TextPattern::Exact(value) => s == *value,
                TextPattern::Regex(regex) => regex.is_match(&s),
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub enum ByteStringPattern {
    /// Matches any byte string.
    Any,
    /// Matches the specific byte string.
    Exact(Vec<u8>),
    /// Matches the regex for a byte string.
    Regex(regex::bytes::Regex),
}

impl MatchPattern for ByteStringPattern {
    fn matches(&self, envelope: &Envelope) -> bool {
        if let Ok(bytes) = envelope.extract_subject::<ByteString>() {
            match self {
                ByteStringPattern::Any => true,
                ByteStringPattern::Exact(value) => value == bytes.as_ref(),
                ByteStringPattern::Regex(regex) => regex.is_match(&bytes),
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub enum TagPattern {
    /// Matches any tag.
    Any,
    /// Matches the specific tag.
    Tag(Tag),
    /// Matches the specific tag name.
    Name(String),
    /// Matches the regex for a tag.
    NameRegex(regex::Regex),
}

#[derive(Debug, Clone)]
pub enum ArrayPattern {
    /// Matches any array.
    Any,
    /// Matches arrays with a specific count of elements.
    Count(RangeInclusive<usize>),
}

#[derive(Debug, Clone)]
pub enum MapPattern {
    /// Matches any map.
    Any,
    /// Matches maps with a specific count of entries.
    Count(RangeInclusive<usize>),
}

#[derive(Debug, Clone)]
pub enum BoolPattern {
    /// Matches any boolean value.
    Any,
    /// Matches the specific boolean value.
    Exact(bool),
}

#[derive(Debug, Clone)]
pub enum LeafPattern {
    /// Matches any leaf.
    Any,
    /// Matches the specific CBOR.
    CBOR(CBOR),
    /// Matches a numeric value.
    Number(NumberPattern),
    /// Matches a text value.
    Text(TextPattern),
    /// Matches a byte string value.
    ByteString(ByteStringPattern),
    /// Matches a tag value.
    Tag(TagPattern),
    /// Matches an array.
    Array(ArrayPattern),
    /// Matches a map.
    Map(MapPattern),
    /// Matches a boolean value.
    Boolean(BoolPattern),
    /// Matches the null value.
    Null,
}

#[derive(Debug, Clone)]
pub enum KnownValuePattern {
    /// Matches any known value.
    Any,
    /// Matches the specific known value.
    KnownValue(KnownValue),
    /// Matches the specific known value name.
    Name(String),
    /// Matches the regex for a known value name.
    NameRegex(regex::Regex),
    /// Matches the Unit known value.
    Unit,
}

#[derive(Debug, Clone)]
pub enum ObscuredPattern {
    /// Matches any obscured element.
    Any,
    /// Matches any elided element.
    Elided,
    /// Matches any encrypted element.
    Encrypted,
    /// Matches any compressed element.
    Compressed,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    /// Matches any element.
    Any,
    /// Matches elements with a specific digest.
    Digest(DigestPattern),
    /// Matches a node with specific properties.
    Node(NodePattern),
    /// Matches a leaf with specific properties.
    Leaf(LeafPattern),
    /// Matches a known value.
    KnownValue(KnownValuePattern),
    /// Matches a wrapped element.
    Wrapped,
    /// Matches an assertion element.
    Assertion,
    /// Matches an obscured element.
    Obscured(ObscuredPattern),
    /// Matches an element that matches all conditions.
    And(AllPattern),
    /// Matches an element that matches any condition.
    Or(AnyPattern),
    /// Matches a sequence of elements.
    Sequence(SequencePattern),
    /// Matches an element with a specific cardinality.
    Repeat(RepeatPattern),
}

#[derive(Debug, Clone)]
pub struct AllPattern {
    patterns: Vec<Pattern>,
}

#[derive(Debug, Clone)]
pub struct AnyPattern {
    patterns: Vec<Pattern>,
}

#[derive(Debug, Clone)]
pub struct SequencePattern {
    patterns: Vec<Pattern>,
}

#[derive(Debug, Clone)]
pub struct RepeatPattern {
    element: Box<Pattern>,
    range: RangeInclusive<usize>,
}

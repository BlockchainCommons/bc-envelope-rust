use std::ops::RangeInclusive;

use crate::{
    Envelope,
    pattern::{Matcher, Path},
};

/// Pattern for matching number values.
#[derive(Debug, Clone)]
pub enum NumberPattern {
    /// Matches any number.
    Any,
    /// Matches the specific number.
    Exact(f64),
    /// Matches numbers within a range, inclusive (..=).
    Range(RangeInclusive<f64>),
    /// Matches numbers that are greater than the specified value.
    GreaterThan(f64),
    /// Matches numbers that are greater than or equal to the specified value.
    GreaterThanOrEqual(f64),
    /// Matches numbers that are less than the specified value.
    LessThan(f64),
    /// Matches numbers that are less than or equal to the specified value.
    LessThanOrEqual(f64),
    /// Matches numbers that are NaN (Not a Number).
    NaN,
}

impl NumberPattern {
    /// Creates a new `NumberPattern` that matches any number.
    pub fn any() -> Self { NumberPattern::Any }

    /// Creates a new `NumberPattern` that matches the exact number.
    pub fn exact<T>(value: T) -> Self
    where
        T: Into<f64>,
    {
        NumberPattern::Exact(value.into())
    }

    /// Creates a new `NumberPattern` that matches numbers within the specified
    /// range.
    pub fn range<A>(range: RangeInclusive<A>) -> Self
    where
        A: Into<f64> + Copy,
    {
        let start = (*range.start()).into();
        let end = (*range.end()).into();
        NumberPattern::Range(RangeInclusive::new(start, end))
    }

    /// Creates a new `NumberPattern` that matches numbers greater than the
    /// specified value.
    pub fn greater_than<T>(value: T) -> Self
    where
        T: Into<f64>,
    {
        NumberPattern::GreaterThan(value.into())
    }

    /// Creates a new `NumberPattern` that matches numbers greater than or
    /// equal to the specified value.
    pub fn greater_than_or_equal<T>(value: T) -> Self
    where
        T: Into<f64>,
    {
        NumberPattern::GreaterThanOrEqual(value.into())
    }

    /// Creates a new `NumberPattern` that matches numbers less than the
    /// specified value.
    pub fn less_than<T>(value: T) -> Self
    where
        T: Into<f64>,
    {
        NumberPattern::LessThan(value.into())
    }

    /// Creates a new `NumberPattern` that matches numbers less than or equal
    /// to the specified value.
    pub fn less_than_or_equal<T>(value: T) -> Self
    where
        T: Into<f64>,
    {
        NumberPattern::LessThanOrEqual(value.into())
    }

    /// Creates a new `NumberPattern` that matches NaN values.
    pub fn nan() -> Self { NumberPattern::NaN }
}

impl Matcher for NumberPattern {
    fn paths(&self, envelope: &Envelope) -> Vec<Path> {
        let subject = envelope.subject();
        let is_hit = match self {
            NumberPattern::Any => subject.is_number(),
            NumberPattern::Exact(want) => {
                subject.extract_subject().ok() == Some(*want)
            }
            NumberPattern::Range(want) => {
                subject.extract_subject().is_ok_and(|n| want.contains(&n))
            }
            NumberPattern::GreaterThan(want) => {
                subject.extract_subject().is_ok_and(|n: f64| n > *want)
            }
            NumberPattern::GreaterThanOrEqual(want) => {
                subject.extract_subject().is_ok_and(|n: f64| n >= *want)
            }
            NumberPattern::LessThan(want) => {
                subject.extract_subject().is_ok_and(|n: f64| n < *want)
            }
            NumberPattern::LessThanOrEqual(want) => {
                subject.extract_subject().is_ok_and(|n: f64| n <= *want)
            }
            NumberPattern::NaN => subject.is_nan(),
        };

        if is_hit {
            vec![vec![envelope.clone()]]
        } else {
            vec![]
        }
    }
}

/// Greediness (a.k.a. laziness / possessiveness) for quantifiers.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Greediness {
    /// Try maximum count first, back-track downwards.
    Greedy,
    /// Try minimum count first, grow as necessary.
    Lazy,
    /// Take maximum count and **never** back-track.
    Possessive,
}

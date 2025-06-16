/// Greediness (a.k.a. laziness / possessiveness) for quantifiers.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Greediness {
    /// Grabs as many repetitions as possible, then backtracks if the rest of the pattern cannot match.
    Greedy,
    /// Starts with as few repetitions as possible, adding more only if the rest of the pattern cannot match.
    Lazy,
    /// Grabs as many repetitions as possible and never backtracks; if the rest of the pattern cannot match, the whole match fails.
    Possessive,
}

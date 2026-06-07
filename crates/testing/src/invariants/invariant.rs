/// Whether an invariant check passed or failed with one or more violations.
pub enum InvariantOutcome {
    Passed,
    Failed(Vec<String>),
}

/// A post-run property that can be checked against collected stream data.
///
/// Each implementing struct holds the prepared input it needs; callers construct
/// it after reading the stream, then hand it to [`InvariantRunner`](super::InvariantRunner).
pub trait Invariant {
    fn name(&self) -> &str;

    fn check(&self) -> InvariantOutcome;
}

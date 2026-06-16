//! Vote value object: signals for answer quality and perishability.
//!
//! Votes on answers include helpfulness signals (Helpful, Unhelpful) and
//! perishability signals (StillValid: "this answer is still accurate").
//! The StillValid signal is particularly important for domain knowledge
//! that may become outdated with clinical/software advances.

/// A vote on an answer.
///
/// Votes record community signals about answer quality and continued relevance.
/// - `Helpful`: user found the answer useful
/// - `Unhelpful`: user found the answer not useful or incorrect
/// - `StillValid`: explicit signal that the answer is still accurate (perishability signal)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Vote {
    /// User found this answer helpful.
    Helpful,
    /// User found this answer unhelpful or incorrect.
    Unhelpful,
    /// User confirms this answer is still accurate (perishability check).
    StillValid,
}

impl std::fmt::Display for Vote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Helpful => write!(f, "Helpful"),
            Self::Unhelpful => write!(f, "Unhelpful"),
            Self::StillValid => write!(f, "StillValid"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vote_display() {
        assert_eq!(format!("{}", Vote::Helpful), "Helpful");
        assert_eq!(format!("{}", Vote::Unhelpful), "Unhelpful");
        assert_eq!(format!("{}", Vote::StillValid), "StillValid");
    }

    #[test]
    fn votes_are_cloneable() {
        let vote = Vote::Helpful;
        let cloned = vote.clone();
        assert_eq!(vote, cloned);
    }

    #[test]
    fn votes_are_hashable() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Vote::Helpful);
        set.insert(Vote::Unhelpful);
        set.insert(Vote::StillValid);

        assert_eq!(set.len(), 3);
        assert!(set.contains(&Vote::Helpful));
    }
}

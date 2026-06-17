//! Vote value object: signals for answer quality and perishability.
//!
//! Votes on answers include helpfulness signals (Helpful, Unhelpful) and
//! perishability signals (StillValid: "this answer is still accurate").
//! The StillValid signal is particularly important for domain knowledge
//! that may become outdated with clinical/software advances.
//!
//! Every vote is attributed to a voter ([`CastVote`]). Votes are deduplicated
//! per voter **per axis** (see [`VoteAxis`]): one voter contributes at most one
//! quality vote and at most one perishability vote. This makes the counts
//! tamper-resistant — a single actor cannot inflate a tally by voting in a loop,
//! which matters because `StillValid` is the perishability signal used to decide
//! whether a possibly-outdated clinical-software answer is still trustworthy.

use crate::domain::id::UserId;

/// A vote on an answer.
///
/// Votes record community signals about answer quality and continued relevance.
/// - `Helpful`: user found the answer useful
/// - `Unhelpful`: user found the answer not useful or incorrect
/// - `StillValid`: explicit signal that the answer is still accurate (perishability signal)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Vote {
    /// User found this answer helpful.
    Helpful,
    /// User found this answer unhelpful or incorrect.
    Unhelpful,
    /// User confirms this answer is still accurate (perishability check).
    StillValid,
}

/// The independent axis a vote belongs to.
///
/// `Helpful` and `Unhelpful` are mutually exclusive opinions on the same
/// *quality* axis, so a new one replaces a voter's prior quality vote.
/// `StillValid` lives on its own *perishability* axis, so a voter may hold both
/// a quality vote and a perishability vote at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VoteAxis {
    /// Helpful / Unhelpful.
    Quality,
    /// StillValid.
    Perishability,
}

impl Vote {
    /// The axis this vote belongs to.
    pub fn axis(self) -> VoteAxis {
        match self {
            Vote::Helpful | Vote::Unhelpful => VoteAxis::Quality,
            Vote::StillValid => VoteAxis::Perishability,
        }
    }
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

/// A vote attributed to the voter who cast it.
///
/// Attribution is what makes tallies tamper-resistant: deduplication per
/// `(voter, axis)` is only possible because each vote carries its `voter`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CastVote {
    voter: UserId,
    vote: Vote,
}

impl CastVote {
    /// Create an attributed vote.
    pub fn new(voter: UserId, vote: Vote) -> Self {
        CastVote { voter, vote }
    }

    /// The user who cast this vote.
    pub fn voter(&self) -> UserId {
        self.voter
    }

    /// The vote that was cast.
    pub fn vote(&self) -> Vote {
        self.vote
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
    fn vote_axes() {
        assert_eq!(Vote::Helpful.axis(), VoteAxis::Quality);
        assert_eq!(Vote::Unhelpful.axis(), VoteAxis::Quality);
        assert_eq!(Vote::StillValid.axis(), VoteAxis::Perishability);
    }

    #[test]
    fn votes_are_cloneable() {
        let vote = Vote::Helpful;
        let cloned = vote;
        assert_eq!(vote, cloned);
    }

    #[test]
    fn cast_vote_carries_voter_and_vote() {
        let cv = CastVote::new(UserId::new(7), Vote::StillValid);
        assert_eq!(cv.voter(), UserId::new(7));
        assert_eq!(cv.vote(), Vote::StillValid);
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

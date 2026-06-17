//! Answer aggregate: replies to questions with optional credential authority.
//!
//! Answers are created with a body, author, license, and an optional authority
//! snapshot captured at authoring time. Like questions, answers have revision
//! history with monotonic timestamps. Additionally, answers track community
//! votes (attributed to voters) for helpfulness and perishability signals.

use crate::domain::body::Body;
use crate::domain::credential::AuthoritySnapshot;
use crate::domain::id::{AnswerId, UserId};
use crate::domain::license::License;
use crate::domain::question::{Revision, RevisionError};
use crate::domain::vote::{CastVote, Vote};
use std::time::SystemTime;

/// An answer aggregate: a reply to a question with votes and revision history.
///
/// Answers carry a license (data on the record) and, optionally, the authority
/// they were authored under (`AuthoritySnapshot` — scope + weight frozen at
/// authoring time, never a bare presence bit). Like questions, answers can be
/// edited, creating a revision history.
#[derive(Clone, Debug, PartialEq)]
pub struct Answer {
    /// Unique identifier for this answer.
    id: AnswerId,
    /// The current body of the answer.
    current_body: Body,
    /// The user who created this answer.
    author_id: UserId,
    /// When the answer was originally created.
    created_at: SystemTime,
    /// The license under which this answer's content is held.
    license: License,
    /// The authority the answer was authored under, captured at authoring time.
    /// `None` for anonymous/unverified answers.
    credential: Option<AuthoritySnapshot>,
    /// All edits to the answer body, in chronological order.
    revisions: Vec<Revision>,
    /// Community votes on this answer, attributed to voters and deduplicated
    /// per `(voter, axis)`.
    votes: Vec<CastVote>,
}

impl Answer {
    /// Create a new answer with an initial body, license, and optional authority.
    ///
    /// The answer starts with no revision history and no votes. The credential
    /// authority is optional: answers can be posted anonymously or unverified.
    pub fn new(
        id: AnswerId,
        initial_body: Body,
        author_id: UserId,
        created_at: SystemTime,
        license: License,
        credential: Option<AuthoritySnapshot>,
    ) -> Self {
        Answer {
            id,
            current_body: initial_body,
            author_id,
            created_at,
            license,
            credential,
            revisions: Vec::new(),
            votes: Vec::new(),
        }
    }

    /// Access the answer's unique identifier.
    pub fn id(&self) -> AnswerId {
        self.id
    }

    /// Access the current body of the answer.
    pub fn current_body(&self) -> &Body {
        &self.current_body
    }

    /// Access the user who created this answer.
    pub fn author_id(&self) -> UserId {
        self.author_id
    }

    /// Access the original creation timestamp.
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }

    /// Access the license this answer's content is held under.
    pub fn license(&self) -> License {
        self.license
    }

    /// Access the authority snapshot the answer was authored under, if any.
    pub fn credential(&self) -> Option<&AuthoritySnapshot> {
        self.credential.as_ref()
    }

    /// Check if this answer was posted with a verified credential.
    pub fn has_credential(&self) -> bool {
        self.credential.is_some()
    }

    /// Access the revision history (all previous bodies).
    pub fn revisions(&self) -> &[Revision] {
        &self.revisions
    }

    /// Access all votes on this answer.
    pub fn votes(&self) -> &[CastVote] {
        &self.votes
    }

    /// Count votes of a specific type (distinct voters, by construction).
    pub fn vote_count(&self, vote_type: &Vote) -> usize {
        self.votes.iter().filter(|v| v.vote() == *vote_type).count()
    }

    /// Get the total number of revisions (edits) to this answer.
    pub fn revision_count(&self) -> usize {
        self.revisions.len()
    }

    /// Get the total number of votes (one per voter per axis).
    pub fn total_votes(&self) -> usize {
        self.votes.len()
    }

    /// The most recent timestamp known to this aggregate.
    fn latest_timestamp(&self) -> SystemTime {
        self.revisions
            .last()
            .map(Revision::created_at)
            .unwrap_or(self.created_at)
    }

    /// Edit the answer body, recording the old body as a revision.
    ///
    /// # Errors
    ///
    /// Returns `RevisionError::NonMonotonicTimestamp` if `edited_at` is earlier
    /// than the aggregate's latest known timestamp; the body is left unchanged.
    pub fn edit_body(
        &mut self,
        new_body: Body,
        edited_at: SystemTime,
    ) -> Result<(), RevisionError> {
        if edited_at < self.latest_timestamp() {
            return Err(RevisionError::NonMonotonicTimestamp);
        }
        let old_body = std::mem::replace(&mut self.current_body, new_body);
        self.revisions.push(Revision::new(old_body, edited_at));
        Ok(())
    }

    /// Record a vote from `voter` on this answer.
    ///
    /// Votes are deduplicated per `(voter, axis)`: a voter has at most one
    /// quality vote (Helpful/Unhelpful, mutually exclusive — a new one replaces
    /// the old) and at most one perishability vote (StillValid). This prevents a
    /// single actor from inflating any tally; in particular it protects the
    /// StillValid perishability signal from ballot-stuffing.
    pub fn add_vote(&mut self, voter: UserId, vote: Vote) {
        let axis = vote.axis();
        self.votes
            .retain(|cv| !(cv.voter() == voter && cv.vote().axis() == axis));
        self.votes.push(CastVote::new(voter, vote));
    }

    /// Get counts of helpful vs unhelpful votes in a single pass.
    ///
    /// Returns (helpful_count, unhelpful_count).
    pub fn helpfulness_counts(&self) -> (usize, usize) {
        let mut helpful = 0;
        let mut unhelpful = 0;
        for cv in &self.votes {
            match cv.vote() {
                Vote::Helpful => helpful += 1,
                Vote::Unhelpful => unhelpful += 1,
                Vote::StillValid => {}
            }
        }
        (helpful, unhelpful)
    }

    /// Get the count of StillValid perishability votes (distinct voters).
    pub fn still_valid_count(&self) -> usize {
        self.vote_count(&Vote::StillValid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::credential::{AuthorityWeight, CredentialScope};

    fn make_test_body(s: &str) -> Body {
        Body::new(s).expect("valid test body")
    }

    fn sample_authority() -> AuthoritySnapshot {
        AuthoritySnapshot::new(
            CredentialScope::Engineering,
            AuthorityWeight::new(0.9).unwrap(),
        )
    }

    #[test]
    fn create_answer_without_credential() {
        let id = AnswerId::new(1);
        let body = make_test_body("Use #[tokio::main] attribute");
        let author = UserId::new(42);
        let created_at = SystemTime::now();

        let answer = Answer::new(id, body.clone(), author, created_at, License::Native, None);

        assert_eq!(answer.id(), id);
        assert_eq!(answer.current_body(), &body);
        assert_eq!(answer.author_id(), author);
        assert_eq!(answer.created_at(), created_at);
        assert_eq!(answer.license(), License::Native);
        assert!(!answer.has_credential());
        assert_eq!(answer.credential(), None);
        assert_eq!(answer.revisions(), &[]);
        assert_eq!(answer.votes(), &[]);
    }

    #[test]
    fn create_answer_captures_authority_snapshot() {
        let answer = Answer::new(
            AnswerId::new(2),
            make_test_body("Verified answer"),
            UserId::new(100),
            SystemTime::now(),
            License::Native,
            Some(sample_authority()),
        );

        assert!(answer.has_credential());
        let snap = answer.credential().expect("authority present");
        assert_eq!(snap.scope(), CredentialScope::Engineering);
        assert_eq!(snap.weight().value(), 0.9);
    }

    #[test]
    fn edit_answer_body() {
        let id = AnswerId::new(3);
        let initial_body = make_test_body("Initial answer");
        let author = UserId::new(200);
        let created_at = SystemTime::now();

        let mut answer = Answer::new(
            id,
            initial_body.clone(),
            author,
            created_at,
            License::Native,
            None,
        );
        assert_eq!(answer.revision_count(), 0);

        let edit_time = created_at + std::time::Duration::from_secs(10);
        let new_body = make_test_body("Updated with more clarity");
        answer.edit_body(new_body.clone(), edit_time).unwrap();

        assert_eq!(answer.current_body(), &new_body);
        assert_eq!(answer.revision_count(), 1);
        assert_eq!(answer.revisions()[0].body(), &initial_body);
    }

    #[test]
    fn edit_rejects_non_monotonic_timestamp() {
        let created_at = SystemTime::now();
        let mut answer = Answer::new(
            AnswerId::new(3),
            make_test_body("Original"),
            UserId::new(1),
            created_at,
            License::Native,
            None,
        );
        let before = created_at - std::time::Duration::from_secs(1);
        assert_eq!(
            answer.edit_body(make_test_body("Backdated"), before),
            Err(RevisionError::NonMonotonicTimestamp)
        );
        assert_eq!(answer.revision_count(), 0);
    }

    #[test]
    fn votes_are_deduplicated_per_voter_per_axis() {
        let mut answer = Answer::new(
            AnswerId::new(4),
            make_test_body("Test answer"),
            UserId::new(300),
            SystemTime::now(),
            License::Native,
            None,
        );

        // One actor voting Helpful in a loop cannot inflate the tally.
        answer.add_vote(UserId::new(1), Vote::Helpful);
        answer.add_vote(UserId::new(1), Vote::Helpful);
        answer.add_vote(UserId::new(1), Vote::Helpful);
        assert_eq!(answer.vote_count(&Vote::Helpful), 1);

        // A new quality vote replaces the prior one on the same axis.
        answer.add_vote(UserId::new(1), Vote::Unhelpful);
        assert_eq!(answer.vote_count(&Vote::Helpful), 0);
        assert_eq!(answer.vote_count(&Vote::Unhelpful), 1);

        // StillValid is a separate axis, so the same voter may hold both.
        answer.add_vote(UserId::new(1), Vote::StillValid);
        assert_eq!(answer.vote_count(&Vote::Unhelpful), 1);
        assert_eq!(answer.still_valid_count(), 1);

        // Distinct voters each count once.
        answer.add_vote(UserId::new(2), Vote::Helpful);
        answer.add_vote(UserId::new(3), Vote::Helpful);
        assert_eq!(answer.vote_count(&Vote::Helpful), 2);
    }

    #[test]
    fn still_valid_cannot_be_stuffed_by_one_actor() {
        let mut answer = Answer::new(
            AnswerId::new(6),
            make_test_body("Answer"),
            UserId::new(500),
            SystemTime::now(),
            License::Native,
            None,
        );
        for _ in 0..100 {
            answer.add_vote(UserId::new(7), Vote::StillValid);
        }
        assert_eq!(answer.still_valid_count(), 1);

        answer.add_vote(UserId::new(8), Vote::StillValid);
        answer.add_vote(UserId::new(9), Vote::StillValid);
        assert_eq!(answer.still_valid_count(), 3);
    }

    #[test]
    fn helpfulness_counts() {
        let mut answer = Answer::new(
            AnswerId::new(5),
            make_test_body("Answer"),
            UserId::new(400),
            SystemTime::now(),
            License::Native,
            None,
        );

        answer.add_vote(UserId::new(1), Vote::Helpful);
        answer.add_vote(UserId::new(2), Vote::Helpful);
        answer.add_vote(UserId::new(3), Vote::Helpful);
        answer.add_vote(UserId::new(4), Vote::Unhelpful);

        let (helpful, unhelpful) = answer.helpfulness_counts();
        assert_eq!(helpful, 3);
        assert_eq!(unhelpful, 1);
    }

    #[test]
    fn author_and_created_at_immutable() {
        let id = AnswerId::new(7);
        let body = make_test_body("Original");
        let author = UserId::new(600);
        let original_created = SystemTime::now();

        let mut answer = Answer::new(id, body, author, original_created, License::Native, None);

        let edit_time = original_created + std::time::Duration::from_secs(3600);
        let new_body = make_test_body("Edited");
        answer.edit_body(new_body, edit_time).unwrap();

        assert_eq!(answer.author_id(), author);
        assert_eq!(answer.created_at(), original_created);
    }

    #[test]
    fn answer_is_cloneable() {
        let id = AnswerId::new(8);
        let body = make_test_body("Answer text");
        let author = UserId::new(700);
        let created_at = SystemTime::now();

        let mut answer = Answer::new(
            id,
            body,
            author,
            created_at,
            License::CcBySa4,
            Some(sample_authority()),
        );
        answer.add_vote(UserId::new(1), Vote::Helpful);

        let cloned = answer.clone();

        assert_eq!(answer, cloned);
        assert_eq!(answer.id(), cloned.id());
        assert_eq!(answer.votes(), cloned.votes());
    }

    #[test]
    fn multiple_edits_accumulate_revisions() {
        let id = AnswerId::new(9);
        let body1 = make_test_body("First version");
        let author = UserId::new(800);
        let created_at = SystemTime::now();

        let mut answer = Answer::new(id, body1.clone(), author, created_at, License::Native, None);

        let time1 = created_at + std::time::Duration::from_secs(1);
        let body2 = make_test_body("Second version");
        answer.edit_body(body2.clone(), time1).unwrap();

        let time2 = created_at + std::time::Duration::from_secs(2);
        let body3 = make_test_body("Third version");
        answer.edit_body(body3.clone(), time2).unwrap();

        assert_eq!(answer.revision_count(), 2);
        assert_eq!(answer.current_body(), &body3);
        assert_eq!(answer.revisions()[0].body(), &body1);
        assert_eq!(answer.revisions()[1].body(), &body2);
    }
}

//! Answer aggregate: replies to questions with optional credential weight.
//!
//! Answers are created with a body, author, and optional verified credential.
//! Like questions, answers have revision history. Additionally, answers track
//! community votes for helpfulness and perishability signals.

use crate::domain::body::Body;
use crate::domain::id::{AnswerId, UserId};
use crate::domain::question::Revision;
use crate::domain::vote::Vote;
use std::time::SystemTime;

/// An answer aggregate: a reply to a question with votes and revision history.
///
/// Answers are authored with optional verified credentials. The credential,
/// if present, provides authority weight to the answer. Like questions,
/// answers can be edited, creating a revision history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    /// Unique identifier for this answer.
    id: AnswerId,
    /// The current body of the answer.
    current_body: Body,
    /// The user who created this answer.
    author_id: UserId,
    /// When the answer was originally created.
    created_at: SystemTime,
    /// Optional verified credential of the author (for authority weight).
    /// Presence indicates the author was verified in some professional domain.
    credential: Option<()>, // Placeholder: will hold VerifiedCredential<Active> in M2
    /// All edits to the answer body, in chronological order.
    revisions: Vec<Revision>,
    /// Community votes on this answer (helpfulness, perishability signals).
    votes: Vec<Vote>,
}

impl Answer {
    /// Create a new answer with an initial body.
    ///
    /// The answer starts with no revision history and no votes.
    /// The credential is optional: answers can be posted anonymously or unverified.
    pub fn new(
        id: AnswerId,
        initial_body: Body,
        author_id: UserId,
        created_at: SystemTime,
        credential: Option<()>,
    ) -> Self {
        Answer {
            id,
            current_body: initial_body,
            author_id,
            created_at,
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

    /// Check if this answer was posted with a verified credential.
    pub fn has_credential(&self) -> bool {
        self.credential.is_some()
    }

    /// Access the revision history (all previous bodies).
    pub fn revisions(&self) -> &[Revision] {
        &self.revisions
    }

    /// Access all votes on this answer.
    pub fn votes(&self) -> &[Vote] {
        &self.votes
    }

    /// Count votes of a specific type.
    pub fn vote_count(&self, vote_type: &Vote) -> usize {
        self.votes.iter().filter(|v| v == &vote_type).count()
    }

    /// Get the total number of revisions (edits) to this answer.
    pub fn revision_count(&self) -> usize {
        self.revisions.len()
    }

    /// Get the total number of votes.
    pub fn total_votes(&self) -> usize {
        self.votes.len()
    }

    /// Edit the answer body, recording the old body as a revision.
    ///
    /// The current body becomes a revision, and the new body becomes current.
    pub fn edit_body(&mut self, new_body: Body, edited_at: SystemTime) {
        let old_body = std::mem::replace(&mut self.current_body, new_body);
        self.revisions.push(Revision::new(old_body, edited_at));
    }

    /// Add a vote to this answer.
    ///
    /// Votes accumulate; duplicates are allowed (user can vote multiple times).
    pub fn add_vote(&mut self, vote: Vote) {
        self.votes.push(vote);
    }

    /// Get counts of helpful vs unhelpful votes.
    ///
    /// Returns (helpful_count, unhelpful_count).
    pub fn helpfulness_counts(&self) -> (usize, usize) {
        let helpful = self
            .votes
            .iter()
            .filter(|v| matches!(v, Vote::Helpful))
            .count();
        let unhelpful = self
            .votes
            .iter()
            .filter(|v| matches!(v, Vote::Unhelpful))
            .count();
        (helpful, unhelpful)
    }

    /// Get the count of StillValid perishability votes.
    pub fn still_valid_count(&self) -> usize {
        self.votes
            .iter()
            .filter(|v| matches!(v, Vote::StillValid))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_body(s: &str) -> Body {
        Body::new(s).expect("valid test body")
    }

    #[test]
    fn create_answer_without_credential() {
        let id = AnswerId::new(1);
        let body = make_test_body("Use #[tokio::main] attribute");
        let author = UserId::new(42);
        let created_at = SystemTime::now();

        let answer = Answer::new(id, body.clone(), author, created_at, None);

        assert_eq!(answer.id(), id);
        assert_eq!(answer.current_body(), &body);
        assert_eq!(answer.author_id(), author);
        assert_eq!(answer.created_at(), created_at);
        assert!(!answer.has_credential());
        assert_eq!(answer.revisions(), &[]);
        assert_eq!(answer.votes(), &[]);
    }

    #[test]
    fn create_answer_with_credential_placeholder() {
        let id = AnswerId::new(2);
        let body = make_test_body("Verified answer");
        let author = UserId::new(100);
        let created_at = SystemTime::now();

        let answer = Answer::new(id, body.clone(), author, created_at, Some(()));

        assert!(answer.has_credential());
    }

    #[test]
    fn edit_answer_body() {
        let id = AnswerId::new(3);
        let initial_body = make_test_body("Initial answer");
        let author = UserId::new(200);
        let created_at = SystemTime::now();

        let mut answer = Answer::new(id, initial_body.clone(), author, created_at, None);
        assert_eq!(answer.revision_count(), 0);

        let edit_time = SystemTime::now();
        let new_body = make_test_body("Updated with more clarity");
        answer.edit_body(new_body.clone(), edit_time);

        assert_eq!(answer.current_body(), &new_body);
        assert_eq!(answer.revision_count(), 1);
        assert_eq!(answer.revisions()[0].body(), &initial_body);
    }

    #[test]
    fn add_votes_and_count() {
        let id = AnswerId::new(4);
        let body = make_test_body("Test answer");
        let author = UserId::new(300);
        let created_at = SystemTime::now();

        let mut answer = Answer::new(id, body, author, created_at, None);

        answer.add_vote(Vote::Helpful);
        answer.add_vote(Vote::Helpful);
        answer.add_vote(Vote::Unhelpful);
        answer.add_vote(Vote::StillValid);

        assert_eq!(answer.total_votes(), 4);
        assert_eq!(answer.vote_count(&Vote::Helpful), 2);
        assert_eq!(answer.vote_count(&Vote::Unhelpful), 1);
        assert_eq!(answer.vote_count(&Vote::StillValid), 1);
    }

    #[test]
    fn helpfulness_counts() {
        let id = AnswerId::new(5);
        let body = make_test_body("Answer");
        let author = UserId::new(400);
        let created_at = SystemTime::now();

        let mut answer = Answer::new(id, body, author, created_at, None);

        answer.add_vote(Vote::Helpful);
        answer.add_vote(Vote::Helpful);
        answer.add_vote(Vote::Helpful);
        answer.add_vote(Vote::Unhelpful);

        let (helpful, unhelpful) = answer.helpfulness_counts();
        assert_eq!(helpful, 3);
        assert_eq!(unhelpful, 1);
    }

    #[test]
    fn still_valid_count() {
        let id = AnswerId::new(6);
        let body = make_test_body("Answer");
        let author = UserId::new(500);
        let created_at = SystemTime::now();

        let mut answer = Answer::new(id, body, author, created_at, None);

        answer.add_vote(Vote::StillValid);
        answer.add_vote(Vote::StillValid);
        answer.add_vote(Vote::Helpful);
        answer.add_vote(Vote::StillValid);

        assert_eq!(answer.still_valid_count(), 3);
    }

    #[test]
    fn author_and_created_at_immutable() {
        let id = AnswerId::new(7);
        let body = make_test_body("Original");
        let author = UserId::new(600);
        let original_created = SystemTime::now();

        let mut answer = Answer::new(id, body, author, original_created, None);

        let edit_time = original_created + std::time::Duration::from_secs(3600);
        let new_body = make_test_body("Edited");
        answer.edit_body(new_body, edit_time);

        assert_eq!(answer.author_id(), author);
        assert_eq!(answer.created_at(), original_created);
    }

    #[test]
    fn answer_is_cloneable() {
        let id = AnswerId::new(8);
        let body = make_test_body("Answer text");
        let author = UserId::new(700);
        let created_at = SystemTime::now();

        let mut answer = Answer::new(id, body, author, created_at, Some(()));
        answer.add_vote(Vote::Helpful);

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

        let mut answer = Answer::new(id, body1.clone(), author, created_at, None);

        let time1 = created_at + std::time::Duration::from_secs(1);
        let body2 = make_test_body("Second version");
        answer.edit_body(body2.clone(), time1);

        let time2 = created_at + std::time::Duration::from_secs(2);
        let body3 = make_test_body("Third version");
        answer.edit_body(body3.clone(), time2);

        assert_eq!(answer.revision_count(), 2);
        assert_eq!(answer.current_body(), &body3);
        assert_eq!(answer.revisions()[0].body(), &body1);
        assert_eq!(answer.revisions()[1].body(), &body2);
    }
}

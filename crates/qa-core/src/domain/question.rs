//! Question aggregate: the root entity for all questions in qa-core.
//!
//! Questions are created with a body, author, and timestamp. The revision history
//! tracks all edits to the question body over time.

use crate::domain::body::Body;
use crate::domain::id::{QuestionId, UserId};
use std::time::SystemTime;

/// A revision of a question body, recording when it was changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revision {
    /// The body text at this revision.
    body: Body,
    /// When this revision was created.
    created_at: SystemTime,
}

impl Revision {
    /// Create a new revision with a body and timestamp.
    pub fn new(body: Body, created_at: SystemTime) -> Self {
        Revision { body, created_at }
    }

    /// Access the body of this revision.
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// Access the creation timestamp of this revision.
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }
}

/// A question aggregate: an immutable collection of metadata and revision history.
///
/// Once created, a question's identity (id), author, and creation timestamp are fixed.
/// The question body may be edited, which appends to the revision history.
/// The current body is always accessible via `current_body()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    /// Unique identifier for this question.
    id: QuestionId,
    /// The current body of the question.
    current_body: Body,
    /// The user who created this question.
    author_id: UserId,
    /// When the question was originally created.
    created_at: SystemTime,
    /// All edits to the question body, in chronological order.
    /// Empty if the question has never been edited (only the current body exists).
    revisions: Vec<Revision>,
}

impl Question {
    /// Create a new question with an initial body.
    ///
    /// The question starts with no revision history (revisions is empty).
    /// The current body is the initial body passed here.
    pub fn new(
        id: QuestionId,
        initial_body: Body,
        author_id: UserId,
        created_at: SystemTime,
    ) -> Self {
        Question {
            id,
            current_body: initial_body,
            author_id,
            created_at,
            revisions: Vec::new(),
        }
    }

    /// Access the question's unique identifier.
    pub fn id(&self) -> QuestionId {
        self.id
    }

    /// Access the current body of the question.
    pub fn current_body(&self) -> &Body {
        &self.current_body
    }

    /// Access the user who created this question.
    pub fn author_id(&self) -> UserId {
        self.author_id
    }

    /// Access the original creation timestamp.
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }

    /// Access the revision history (all previous bodies).
    pub fn revisions(&self) -> &[Revision] {
        &self.revisions
    }

    /// Edit the question body, recording the old body as a revision.
    ///
    /// The current body becomes a revision, and the new body becomes current.
    pub fn edit_body(&mut self, new_body: Body, edited_at: SystemTime) {
        let old_body = std::mem::replace(&mut self.current_body, new_body);
        self.revisions.push(Revision::new(old_body, edited_at));
    }

    /// Get the total number of revisions (edits) to this question.
    pub fn revision_count(&self) -> usize {
        self.revisions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_body(s: &str) -> Body {
        Body::new(s).expect("valid test body")
    }

    #[test]
    fn create_question_with_initial_body() {
        let id = QuestionId::new(1);
        let body = make_test_body("How do I use async/await in Rust?");
        let author = UserId::new(42);
        let created_at = SystemTime::now();

        let question = Question::new(id, body.clone(), author, created_at);

        assert_eq!(question.id(), id);
        assert_eq!(question.current_body(), &body);
        assert_eq!(question.author_id(), author);
        assert_eq!(question.created_at(), created_at);
        assert_eq!(question.revisions(), &[]);
    }

    #[test]
    fn edit_question_body() {
        let id = QuestionId::new(2);
        let initial_body = make_test_body("Initial question");
        let author = UserId::new(100);
        let created_at = SystemTime::now();

        let mut question = Question::new(id, initial_body.clone(), author, created_at);
        assert_eq!(question.revision_count(), 0);

        let edit_time = SystemTime::now();
        let new_body = make_test_body("Updated question with more detail");
        question.edit_body(new_body.clone(), edit_time);

        assert_eq!(question.current_body(), &new_body);
        assert_eq!(question.revision_count(), 1);
        assert_eq!(question.revisions()[0].body(), &initial_body);
        assert_eq!(question.revisions()[0].created_at(), edit_time);
    }

    #[test]
    fn multiple_edits_accumulate_revisions() {
        let id = QuestionId::new(3);
        let body1 = make_test_body("First version");
        let author = UserId::new(200);
        let created_at = SystemTime::now();

        let mut question = Question::new(id, body1.clone(), author, created_at);

        let time1 = created_at + std::time::Duration::from_secs(1);
        let body2 = make_test_body("Second version");
        question.edit_body(body2.clone(), time1);

        let time2 = created_at + std::time::Duration::from_secs(2);
        let body3 = make_test_body("Third version");
        question.edit_body(body3.clone(), time2);

        assert_eq!(question.revision_count(), 2);
        assert_eq!(question.current_body(), &body3);
        assert_eq!(question.revisions()[0].body(), &body1);
        assert_eq!(question.revisions()[1].body(), &body2);
    }

    #[test]
    fn question_id_immutable() {
        let id = QuestionId::new(999);
        let body = make_test_body("Test");
        let author = UserId::new(1);
        let created_at = SystemTime::now();

        let question = Question::new(id, body, author, created_at);
        assert_eq!(question.id(), id);
        // Verify immutability through identity comparison
        assert_eq!(question.id(), QuestionId::new(999));
    }

    #[test]
    fn author_and_created_at_immutable() {
        let id = QuestionId::new(42);
        let body = make_test_body("Original");
        let author = UserId::new(7);
        let original_created = SystemTime::now();

        let mut question = Question::new(id, body, author, original_created);

        let edit_time = original_created + std::time::Duration::from_secs(3600);
        let new_body = make_test_body("Edited");
        question.edit_body(new_body, edit_time);

        assert_eq!(question.author_id(), author);
        assert_eq!(question.created_at(), original_created);
    }

    #[test]
    fn revision_preserves_timestamp() {
        let id = QuestionId::new(5);
        let body = make_test_body("Initial");
        let author = UserId::new(10);
        let created_at = SystemTime::now();

        let mut question = Question::new(id, body, author, created_at);

        let specific_time = created_at + std::time::Duration::from_secs(7200);
        let new_body = make_test_body("Modified");
        question.edit_body(new_body, specific_time);

        assert_eq!(question.revisions()[0].created_at(), specific_time);
    }

    #[test]
    fn question_is_cloneable() {
        let id = QuestionId::new(11);
        let body = make_test_body("Original");
        let author = UserId::new(22);
        let created_at = SystemTime::now();

        let question = Question::new(id, body, author, created_at);
        let cloned = question.clone();

        assert_eq!(question, cloned);
        assert_eq!(question.id(), cloned.id());
        assert_eq!(question.current_body(), cloned.current_body());
    }
}

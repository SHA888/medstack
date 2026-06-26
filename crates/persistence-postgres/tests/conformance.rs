//! Conformance test suite for Postgres persistence adapter.
//!
//! Tests that all M1 aggregates (Question and Answer) can be persisted to Postgres
//! and retrieved with full fidelity, including revisions, votes, and credentials.
//!
//! To run these tests, set the `MEDOVERFLOW_POSTGRES_TEST_URL` environment variable
//! to a Postgres connection string for a test database. If not set, tests will be skipped.

use persistence_postgres::PostgresPersistence;
use qa_core::domain::answer::Answer;
use qa_core::domain::body::Body;
use qa_core::domain::credential::{AuthoritySnapshot, AuthorityWeight, CredentialScope};
use qa_core::domain::id::{AnswerId, QuestionId, UserId};
use qa_core::domain::license::License;
use qa_core::domain::ports::{AggregateId, PersistableAggregate, PersistencePort};
use qa_core::domain::question::Question;
use qa_core::domain::tag::{Jurisdiction, Tag};
use qa_core::domain::vote::Vote;
use std::time::SystemTime;

/// Get test database URL from environment, or None if not available.
fn test_db_url() -> Option<String> {
    std::env::var("MEDOVERFLOW_POSTGRES_TEST_URL").ok()
}

/// Macro to skip test if Postgres is not available.
macro_rules! skip_if_no_postgres {
    () => {
        if test_db_url().is_none() {
            eprintln!("Skipping Postgres test: MEDOVERFLOW_POSTGRES_TEST_URL not set");
            return;
        }
    };
}

#[test]
fn test_question_round_trip_simple() {
    skip_if_no_postgres!();
    let url = test_db_url().unwrap();
    let adapter = PostgresPersistence::new(&url).expect("Failed to connect to Postgres");

    let q = Question::new(
        QuestionId::new(1),
        Body::new("What is Rust?").unwrap(),
        UserId::new(100),
        SystemTime::now(),
        License::Native,
        vec![],
    );

    let aggregate = PersistableAggregate::Question(q.clone());
    adapter.persist(aggregate).unwrap();

    let retrieved = adapter
        .retrieve(AggregateId::Question(QuestionId::new(1)))
        .unwrap();
    match retrieved {
        PersistableAggregate::Question(retrieved_q) => assert_eq!(retrieved_q, q),
        _ => panic!("expected question"),
    }
}

#[test]
fn test_question_with_tags() {
    skip_if_no_postgres!();
    let url = test_db_url().unwrap();
    let adapter = PostgresPersistence::new(&url).expect("Failed to connect to Postgres");

    let tags = vec![
        Tag::new(
            "rust",
            "2025-06-14",
            Jurisdiction::new("US").unwrap(),
        )
        .unwrap(),
        Tag::new(
            "async-await",
            "2025-06-15",
            Jurisdiction::new("ID").unwrap(),
        )
        .unwrap(),
    ];

    let q = Question::new(
        QuestionId::new(2),
        Body::new("How to use async/await in Rust?").unwrap(),
        UserId::new(42),
        SystemTime::now(),
        License::CcBySa4,
        tags,
    );

    adapter
        .persist(PersistableAggregate::Question(q.clone()))
        .unwrap();

    let retrieved = adapter
        .retrieve(AggregateId::Question(QuestionId::new(2)))
        .unwrap();
    match retrieved {
        PersistableAggregate::Question(retrieved_q) => {
            assert_eq!(retrieved_q.id(), q.id());
            assert_eq!(retrieved_q.current_body(), q.current_body());
            assert_eq!(retrieved_q.author_id(), q.author_id());
            assert_eq!(retrieved_q.license(), q.license());
            assert_eq!(retrieved_q.tags(), q.tags());
        }
        _ => panic!("expected question"),
    }
}

#[test]
fn test_question_with_revisions() {
    skip_if_no_postgres!();
    let url = test_db_url().unwrap();
    let adapter = PostgresPersistence::new(&url).expect("Failed to connect to Postgres");

    let now = SystemTime::now();
    let later = now + std::time::Duration::from_secs(10);

    let mut q = Question::new(
        QuestionId::new(3),
        Body::new("Original question").unwrap(),
        UserId::new(1),
        now,
        License::Native,
        vec![],
    );

    q.edit_body(Body::new("Updated question").unwrap(), later)
        .unwrap();

    adapter
        .persist(PersistableAggregate::Question(q.clone()))
        .unwrap();

    let retrieved = adapter
        .retrieve(AggregateId::Question(QuestionId::new(3)))
        .unwrap();
    match retrieved {
        PersistableAggregate::Question(retrieved_q) => {
            assert_eq!(retrieved_q.current_body(), q.current_body());
            assert_eq!(retrieved_q.revision_count(), q.revision_count());
            assert_eq!(retrieved_q.revisions()[0].body(), q.revisions()[0].body());
        }
        _ => panic!("expected question"),
    }
}

#[test]
fn test_answer_round_trip_simple() {
    skip_if_no_postgres!();
    let url = test_db_url().unwrap();
    let adapter = PostgresPersistence::new(&url).expect("Failed to connect to Postgres");

    let a = Answer::new(
        AnswerId::new(1),
        Body::new("Rust is a systems language").unwrap(),
        UserId::new(200),
        SystemTime::now(),
        License::Native,
        None,
    );

    adapter
        .persist(PersistableAggregate::Answer(a.clone()))
        .unwrap();

    let retrieved = adapter
        .retrieve(AggregateId::Answer(AnswerId::new(1)))
        .unwrap();
    match retrieved {
        PersistableAggregate::Answer(retrieved_a) => assert_eq!(retrieved_a, a),
        _ => panic!("expected answer"),
    }
}

#[test]
fn test_answer_with_credential() {
    skip_if_no_postgres!();
    let url = test_db_url().unwrap();
    let adapter = PostgresPersistence::new(&url).expect("Failed to connect to Postgres");

    let cred = AuthoritySnapshot::new(
        CredentialScope::Engineering,
        AuthorityWeight::new(0.95).unwrap(),
    );

    let a = Answer::new(
        AnswerId::new(2),
        Body::new("Use the tokio runtime for async").unwrap(),
        UserId::new(300),
        SystemTime::now(),
        License::CcBy4,
        Some(cred),
    );

    adapter
        .persist(PersistableAggregate::Answer(a.clone()))
        .unwrap();

    let retrieved = adapter
        .retrieve(AggregateId::Answer(AnswerId::new(2)))
        .unwrap();
    match retrieved {
        PersistableAggregate::Answer(retrieved_a) => {
            assert_eq!(retrieved_a, a);
            assert!(retrieved_a.has_credential());
            assert_eq!(
                retrieved_a.credential().unwrap().scope(),
                CredentialScope::Engineering
            );
            assert_eq!(retrieved_a.credential().unwrap().weight().value(), 0.95);
        }
        _ => panic!("expected answer"),
    }
}

#[test]
fn test_answer_with_revisions() {
    skip_if_no_postgres!();
    let url = test_db_url().unwrap();
    let adapter = PostgresPersistence::new(&url).expect("Failed to connect to Postgres");

    let now = SystemTime::now();
    let later = now + std::time::Duration::from_secs(20);

    let mut a = Answer::new(
        AnswerId::new(3),
        Body::new("First version of answer").unwrap(),
        UserId::new(1),
        now,
        License::Native,
        None,
    );

    a.edit_body(Body::new("Revised answer with more detail").unwrap(), later)
        .unwrap();

    adapter
        .persist(PersistableAggregate::Answer(a.clone()))
        .unwrap();

    let retrieved = adapter
        .retrieve(AggregateId::Answer(AnswerId::new(3)))
        .unwrap();
    match retrieved {
        PersistableAggregate::Answer(retrieved_a) => {
            assert_eq!(retrieved_a.current_body(), a.current_body());
            assert_eq!(retrieved_a.revision_count(), a.revision_count());
            assert_eq!(retrieved_a.revisions()[0].body(), a.revisions()[0].body());
        }
        _ => panic!("expected answer"),
    }
}

#[test]
fn test_answer_with_votes() {
    skip_if_no_postgres!();
    let url = test_db_url().unwrap();
    let adapter = PostgresPersistence::new(&url).expect("Failed to connect to Postgres");

    let now = SystemTime::now();

    let mut a = Answer::new(
        AnswerId::new(4),
        Body::new("Helpful answer").unwrap(),
        UserId::new(1),
        now,
        License::Native,
        None,
    );

    a.add_vote(UserId::new(100), Vote::Helpful);
    a.add_vote(UserId::new(101), Vote::Helpful);
    a.add_vote(UserId::new(102), Vote::StillValid);

    adapter
        .persist(PersistableAggregate::Answer(a.clone()))
        .unwrap();

    let retrieved = adapter
        .retrieve(AggregateId::Answer(AnswerId::new(4)))
        .unwrap();
    match retrieved {
        PersistableAggregate::Answer(retrieved_a) => {
            assert_eq!(retrieved_a.total_votes(), a.total_votes());
            assert_eq!(
                retrieved_a.vote_count(&Vote::Helpful),
                a.vote_count(&Vote::Helpful)
            );
            assert_eq!(
                retrieved_a.vote_count(&Vote::StillValid),
                a.vote_count(&Vote::StillValid)
            );
        }
        _ => panic!("expected answer"),
    }
}

#[test]
fn test_multiple_questions_independent() {
    skip_if_no_postgres!();
    let url = test_db_url().unwrap();
    let adapter = PostgresPersistence::new(&url).expect("Failed to connect to Postgres");

    let q1 = Question::new(
        QuestionId::new(10),
        Body::new("Question 1").unwrap(),
        UserId::new(1),
        SystemTime::now(),
        License::Native,
        vec![],
    );
    let q2 = Question::new(
        QuestionId::new(11),
        Body::new("Question 2").unwrap(),
        UserId::new(2),
        SystemTime::now(),
        License::CcBySa4,
        vec![],
    );

    adapter.persist(PersistableAggregate::Question(q1.clone())).ok();
    adapter.persist(PersistableAggregate::Question(q2.clone())).ok();

    let retrieved_q1 = adapter
        .retrieve(AggregateId::Question(QuestionId::new(10)))
        .unwrap();
    let retrieved_q2 = adapter
        .retrieve(AggregateId::Question(QuestionId::new(11)))
        .unwrap();

    assert_eq!(retrieved_q1, PersistableAggregate::Question(q1));
    assert_eq!(retrieved_q2, PersistableAggregate::Question(q2));
}

#[test]
fn test_question_update_preserves_identity() {
    skip_if_no_postgres!();
    let url = test_db_url().unwrap();
    let adapter = PostgresPersistence::new(&url).expect("Failed to connect to Postgres");

    let mut q = Question::new(
        QuestionId::new(20),
        Body::new("Original").unwrap(),
        UserId::new(1),
        SystemTime::now(),
        License::Native,
        vec![],
    );

    adapter
        .persist(PersistableAggregate::Question(q.clone()))
        .unwrap();

    q.edit_body(
        Body::new("Updated").unwrap(),
        SystemTime::now() + std::time::Duration::from_secs(1),
    )
    .unwrap();
    adapter
        .persist(PersistableAggregate::Question(q.clone()))
        .unwrap();

    let retrieved = adapter
        .retrieve(AggregateId::Question(QuestionId::new(20)))
        .unwrap();
    match retrieved {
        PersistableAggregate::Question(retrieved_q) => {
            assert_eq!(retrieved_q.id(), QuestionId::new(20));
            assert_eq!(retrieved_q.current_body(), q.current_body());
            assert_eq!(retrieved_q.revision_count(), 1);
        }
        _ => panic!("expected question"),
    }
}

#[test]
fn test_answer_update_preserves_votes() {
    skip_if_no_postgres!();
    let url = test_db_url().unwrap();
    let adapter = PostgresPersistence::new(&url).expect("Failed to connect to Postgres");

    let now = SystemTime::now();
    let mut a = Answer::new(
        AnswerId::new(10),
        Body::new("Initial answer").unwrap(),
        UserId::new(1),
        now,
        License::Native,
        None,
    );

    a.add_vote(UserId::new(100), Vote::Helpful);
    adapter
        .persist(PersistableAggregate::Answer(a.clone()))
        .unwrap();

    a.edit_body(
        Body::new("Updated answer").unwrap(),
        now + std::time::Duration::from_secs(5),
    )
    .unwrap();
    adapter
        .persist(PersistableAggregate::Answer(a.clone()))
        .unwrap();

    let retrieved = adapter
        .retrieve(AggregateId::Answer(AnswerId::new(10)))
        .unwrap();
    match retrieved {
        PersistableAggregate::Answer(retrieved_a) => {
            assert_eq!(retrieved_a.total_votes(), 1);
            assert_eq!(retrieved_a.vote_count(&Vote::Helpful), 1);
            assert_eq!(retrieved_a.revision_count(), 1);
        }
        _ => panic!("expected answer"),
    }
}

#[test]
fn test_not_found_for_missing_question() {
    skip_if_no_postgres!();
    let url = test_db_url().unwrap();
    let adapter = PostgresPersistence::new(&url).expect("Failed to connect to Postgres");

    let result = adapter.retrieve(AggregateId::Question(QuestionId::new(999)));
    assert!(result.is_err());
}

#[test]
fn test_not_found_for_missing_answer() {
    skip_if_no_postgres!();
    let url = test_db_url().unwrap();
    let adapter = PostgresPersistence::new(&url).expect("Failed to connect to Postgres");

    let result = adapter.retrieve(AggregateId::Answer(AnswerId::new(999)));
    assert!(result.is_err());
}

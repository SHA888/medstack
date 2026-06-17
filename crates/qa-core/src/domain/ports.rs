//! Port definitions: interfaces to external systems.
//!
//! Ports are the boundaries of the qa-core hexagon. They define what services
//! qa-core needs from external systems (identity-verification, search, persistence)
//! without depending directly on those systems. Adapters implement these ports.
//!
//! This enforces the architectural rule: only ports point outward; the domain logic
//! itself has zero outward dependencies.

use crate::domain::answer::Answer;
use crate::domain::credential::AuthoritySnapshot;
use crate::domain::id::{AnswerId, ContentId, QuestionId, UserId};
use crate::domain::license::License;
use crate::domain::question::Question;
use std::fmt;

/// Representation of a content change for indexing.
///
/// This enum allows the search system to distinguish between questions and answers
/// when receiving change notifications. The search index tracks both types separately
/// to support type-specific queries and ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexableContent {
    /// A question has been created, modified, or deleted.
    Question(QuestionId),
    /// An answer has been created, modified, or deleted.
    Answer(AnswerId),
}

/// Representation of an aggregate for persistence.
///
/// This enum allows the persistence layer to store and retrieve both questions and answers
/// using a single port interface. The persistence layer treats these as opaque aggregates
/// and does not interpret their internal structure.
///
/// Note: only `PartialEq` (not `Eq`) — an `Answer` may carry an
/// `AuthoritySnapshot` whose weight is an `f64`, which is `PartialEq` but not
/// `Eq`. Round-trip persistence tests assert equality via `PartialEq`, which is
/// sufficient.
#[derive(Debug, Clone, PartialEq)]
pub enum PersistableAggregate {
    /// A question aggregate to be persisted or retrieved.
    Question(Question),
    /// An answer aggregate to be persisted or retrieved.
    Answer(Answer),
}

/// Unique identifier for a persisted aggregate.
///
/// This enum allows the persistence layer to address both questions and answers
/// using a discriminated union of their IDs, enabling a single retrieve method
/// to work with both types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateId {
    /// A question identified by its unique ID.
    Question(QuestionId),
    /// An answer identified by its unique ID.
    Answer(AnswerId),
}

/// Error type for persistence operations.
///
/// Covers all error cases that adapters (SQLite, Postgres) might encounter
/// when persisting or retrieving aggregates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceError {
    /// The requested aggregate was not found in the database.
    NotFound,
    /// Serialization or deserialization of aggregate data failed.
    SerializationError,
    /// A database-level error occurred (connection, constraint violation, etc).
    DatabaseError(String),
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersistenceError::NotFound => write!(f, "aggregate not found"),
            PersistenceError::SerializationError => {
                write!(f, "failed to serialize or deserialize aggregate")
            }
            PersistenceError::DatabaseError(msg) => write!(f, "database error: {}", msg),
        }
    }
}

/// Port for credential verification.
///
/// This port defines how qa-core requests verification of user credentials.
/// The identity-verification crate implements this port. Crucially, qa-core does
/// NOT receive the verification crate's `VerifiedCredential` (it cannot — the
/// architecture test forbids the dependency). Instead the adapter maps a
/// verified, active credential into qa-core's own [`AuthoritySnapshot`]: the
/// scope and authority weight captured **as of verification time**. This keeps
/// the seam stable (the return type lives in qa-core, so it does not change when
/// the real credential type lands) and records authority-at-authoring rather
/// than a bare "verified" bit.
///
/// Implementations must:
/// - Return `Some(snapshot)` if the user has a verified, active credential
/// - Return `None` if the user is not verified, does not exist, or is expired
/// - Never fabricate a snapshot for an unverified user
pub trait CredentialPort {
    /// Verify a user by ID, returning their authority snapshot if verified.
    ///
    /// # Arguments
    /// - `user_id`: The user to verify
    ///
    /// # Returns
    /// - `Some(snapshot)`: user is verified and active; carries scope + weight
    /// - `None`: user is not verified, does not exist, or the credential expired
    ///
    /// # Implementation notes
    /// Implementers (identity-verification crate) are responsible for:
    /// - Checking user identity against external authority systems
    /// - Managing credential expiry and lifecycle
    /// - Mapping their internal `VerifiedCredential<Active>` (scope + computed
    ///   authority weight) into qa-core's `AuthoritySnapshot`
    fn verify_credential(&self, user_id: UserId) -> Option<AuthoritySnapshot>;
}

/// A uniform reference to any licensable piece of content.
///
/// Questions and answers are addressed by their own ID newtypes, while imported
/// material that is not a qa-core aggregate uses `ContentId`. `ContentRef` unifies
/// the three so a caller holding *any* of them can query the licensing port —
/// without conflating the ID namespaces (a `QuestionId(5)` and a `ContentId(5)`
/// remain distinct references). This closes the gap where a caller holding an
/// aggregate ID had no way to reach `source_license`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentRef {
    /// A question aggregate.
    Question(QuestionId),
    /// An answer aggregate.
    Answer(AnswerId),
    /// Imported content that is not a qa-core aggregate (e.g. a mirrored row).
    Imported(ContentId),
}

impl From<QuestionId> for ContentRef {
    fn from(id: QuestionId) -> Self {
        ContentRef::Question(id)
    }
}

impl From<AnswerId> for ContentRef {
    fn from(id: AnswerId) -> Self {
        ContentRef::Answer(id)
    }
}

impl From<ContentId> for ContentRef {
    fn from(id: ContentId) -> Self {
        ContentRef::Imported(id)
    }
}

/// Port for content source licensing.
///
/// This port defines how qa-core queries the license of content from external sources.
/// The ingestion crate implements this port, tracking which license applies to each
/// piece of content based on its origin (Stack Exchange, Biostars, native, etc).
///
/// This is essential for:
/// - Ensuring all content respects its original license
/// - Attribution rendering (must know the license to render correct attribution)
/// - Legal compliance (must verify content can be used before including it)
///
/// Implementations must:
/// - Return the correct License for each content reference
/// - Never return an unknown or invalid license (exhaustive enum enforces this)
/// - Be consistent: same reference always returns same license
pub trait ContentSourcePort {
    /// Query the license of a piece of content by reference.
    ///
    /// # Arguments
    /// - `content`: The content to query — a question, answer, or imported item.
    ///   Use `ContentRef::from(question_id)` etc. when you hold an aggregate ID.
    ///
    /// # Returns
    /// - The License that applies to this content (exhaustive enum, no unknowns)
    ///
    /// # Implementation notes
    /// Implementers (ingestion crate) are responsible for:
    /// - Tracking per-source license rules (SE=CC BY-SA, Biostars=CC BY, etc)
    /// - Assigning licenses to mirrored content based on source
    /// - Assigning licenses to native content based on platform policy
    /// - Never returning an unknown license (compile-fail test enforces this)
    fn source_license(&self, content: ContentRef) -> License;
}

/// Port for search index updates.
///
/// This port defines how qa-core notifies the search system when questions or answers
/// are created, modified, or deleted. The search crate implements this port as a
/// read-side projection that stays synchronized with the qa-core write side.
///
/// This separation enables:
/// - Independent scaling of search indexing from the core domain logic
/// - Different storage backends for the search index (Elasticsearch, SQLite FTS, etc)
/// - Eventual consistency: the search index can lag behind the core domain
/// - Decoupled failure domains: if search indexing fails, it doesn't crash the domain
///
/// Implementations must:
/// - Accept notifications for questions and answers (exhaustive enum enforces this)
/// - Update the search index to reflect the new state
/// - Be idempotent: receiving the same notification twice must be safe
/// - Handle both creation and modification (the port does not distinguish; indexers can decide)
pub trait SearchIndexPort {
    /// Notify the search index that content has changed.
    ///
    /// # Arguments
    /// - `content`: The question or answer that changed (created, modified, or deleted)
    ///
    /// # Implementation notes
    /// Implementers (search crate) are responsible for:
    /// - Updating full-text indexes
    /// - Updating tag facets
    /// - Updating jurisdiction and date facets
    /// - Handling deletions (may store deletion markers instead of removing entirely)
    /// - Being idempotent: if the same content notification arrives twice, the result must be the same
    fn notify_content_changed(&self, content: IndexableContent);
}

/// Port for persistence of question and answer aggregates.
///
/// This port defines how qa-core persists and retrieves questions and answers
/// to/from durable storage. The persistence crate implements this port, providing
/// adapters for different backends (SQLite, Postgres, etc).
///
/// This separation enables:
/// - Multiple storage backends without qa-core knowing about any of them
/// - Independent scaling and optimization of persistence layer
/// - Testing with in-memory or mock adapters
/// - Decoupled failure domains: persistence failures don't crash domain logic
///
/// Implementations must:
/// - Persist aggregates atomically (all-or-nothing)
/// - Preserve aggregate identity (IDs must not be mutated during persistence)
/// - Support round-trip consistency: persist then retrieve returns an equal aggregate
/// - Be consistent: same aggregate ID always retrieves the same state within a transaction
pub trait PersistencePort {
    /// Persist an aggregate (question or answer) to durable storage.
    ///
    /// # Arguments
    /// - `aggregate`: The question or answer to persist (both new and updated aggregates)
    ///
    /// # Returns
    /// - `Ok(())`: Aggregate was persisted successfully
    /// - `Err(PersistenceError)`: Persistence failed (serialization, database error)
    ///
    /// # Implementation notes
    /// Implementers are responsible for:
    /// - Storing the aggregate data in a way that preserves all fields (id, body, author, timestamp, revisions, etc)
    /// - Handling both new (first-time) persistence and updates (aggregate already exists)
    /// - Atomicity: either the entire aggregate is persisted or nothing is persisted
    /// - Round-trip fidelity: an aggregate that is persisted then immediately retrieved must be equal
    /// - For both Question and Answer: the enum variant determines the schema/table to use
    fn persist(&self, aggregate: PersistableAggregate) -> Result<(), PersistenceError>;

    /// Retrieve an aggregate (question or answer) from durable storage by ID.
    ///
    /// # Arguments
    /// - `id`: The ID of the question or answer to retrieve
    ///
    /// # Returns
    /// - `Ok(aggregate)`: The aggregate was found and deserialized successfully
    /// - `Err(NotFound)`: No aggregate with this ID exists in storage
    /// - `Err(SerializationError)`: The aggregate exists but deserialization failed
    /// - `Err(DatabaseError)`: A database-level error occurred
    ///
    /// # Implementation notes
    /// Implementers are responsible for:
    /// - Using the ID variant (Question/Answer) to route to the correct table/storage
    /// - Deserializing the stored data into a complete aggregate (including revisions, votes)
    /// - Returning NotFound if the ID is not in storage (not an error, just an absence)
    /// - Returning consistent state: if called multiple times without intervening persists, always returns the same aggregate
    fn retrieve(&self, id: AggregateId) -> Result<PersistableAggregate, PersistenceError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::credential::{AuthorityWeight, CredentialScope};
    use std::collections::HashMap;

    fn sample_authority() -> AuthoritySnapshot {
        AuthoritySnapshot::new(
            CredentialScope::Engineering,
            AuthorityWeight::new(0.9).unwrap(),
        )
    }

    /// Mock credential port for testing qa-core logic.
    struct MockCredentialPort {
        verified_users: std::collections::HashSet<u64>,
    }

    impl MockCredentialPort {
        fn new(verified_users: Vec<u64>) -> Self {
            MockCredentialPort {
                verified_users: verified_users.into_iter().collect(),
            }
        }
    }

    impl CredentialPort for MockCredentialPort {
        fn verify_credential(&self, user_id: UserId) -> Option<AuthoritySnapshot> {
            if self.verified_users.contains(&user_id.inner()) {
                Some(sample_authority())
            } else {
                None
            }
        }
    }

    #[test]
    fn mock_port_returns_none_for_unverified() {
        let port = MockCredentialPort::new(vec![1, 2, 3]);
        assert_eq!(
            port.verify_credential(UserId::new(1)),
            Some(sample_authority())
        );
        assert_eq!(port.verify_credential(UserId::new(999)), None);
    }

    #[test]
    fn mock_port_verifies_users() {
        let port = MockCredentialPort::new(vec![42, 100]);
        assert!(port.verify_credential(UserId::new(42)).is_some());
        assert!(port.verify_credential(UserId::new(100)).is_some());
        assert_eq!(port.verify_credential(UserId::new(50)), None);
    }

    /// Mock content source port for testing qa-core logic.
    struct MockContentSourcePort {
        licenses: HashMap<ContentRef, License>,
    }

    impl MockContentSourcePort {
        fn new(licenses: HashMap<ContentRef, License>) -> Self {
            MockContentSourcePort { licenses }
        }
    }

    impl ContentSourcePort for MockContentSourcePort {
        fn source_license(&self, content: ContentRef) -> License {
            self.licenses
                .get(&content)
                .copied()
                .expect("content must have a license")
        }
    }

    #[test]
    fn mock_content_port_returns_correct_licenses() {
        let mut licenses = HashMap::new();
        // A question, an answer, and an imported row — three distinct namespaces,
        // all reachable through the one port via ContentRef.
        licenses.insert(ContentRef::from(QuestionId::new(1)), License::CcBySa4);
        licenses.insert(ContentRef::from(AnswerId::new(2)), License::CcBy4);
        licenses.insert(ContentRef::from(ContentId::new(3)), License::LinkOnly);

        let port = MockContentSourcePort::new(licenses);

        assert_eq!(
            port.source_license(QuestionId::new(1).into()),
            License::CcBySa4
        );
        assert_eq!(port.source_license(AnswerId::new(2).into()), License::CcBy4);
        assert_eq!(
            port.source_license(ContentId::new(3).into()),
            License::LinkOnly
        );
    }

    #[test]
    fn content_ref_keeps_namespaces_distinct() {
        // QuestionId(5) and ContentId(5) are different references, not the same key.
        let mut licenses = HashMap::new();
        licenses.insert(ContentRef::from(QuestionId::new(5)), License::CcBySa4);
        licenses.insert(ContentRef::from(ContentId::new(5)), License::LinkOnly);
        let port = MockContentSourcePort::new(licenses);
        assert_eq!(
            port.source_license(QuestionId::new(5).into()),
            License::CcBySa4
        );
        assert_eq!(
            port.source_license(ContentId::new(5).into()),
            License::LinkOnly
        );
    }

    #[test]
    #[should_panic]
    fn mock_content_port_panics_for_unlicensed_content() {
        let port = MockContentSourcePort::new(HashMap::new());
        let _ = port.source_license(ContentId::new(999).into());
    }

    /// Mock search index port for testing qa-core logic.
    #[derive(Clone)]
    struct MockSearchIndexPort {
        indexed_content: std::sync::Arc<std::sync::Mutex<Vec<IndexableContent>>>,
    }

    impl MockSearchIndexPort {
        fn new() -> Self {
            MockSearchIndexPort {
                indexed_content: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn get_indexed_content(&self) -> Vec<IndexableContent> {
            self.indexed_content.lock().unwrap().clone()
        }
    }

    impl SearchIndexPort for MockSearchIndexPort {
        fn notify_content_changed(&self, content: IndexableContent) {
            self.indexed_content.lock().unwrap().push(content);
        }
    }

    #[test]
    fn mock_search_port_records_question_changes() {
        let port = MockSearchIndexPort::new();
        let q1 = IndexableContent::Question(QuestionId::new(1));
        let q2 = IndexableContent::Question(QuestionId::new(2));

        port.notify_content_changed(q1);
        port.notify_content_changed(q2);

        let indexed = port.get_indexed_content();
        assert_eq!(indexed.len(), 2);
        assert_eq!(indexed[0], q1);
        assert_eq!(indexed[1], q2);
    }

    #[test]
    fn mock_search_port_records_answer_changes() {
        let port = MockSearchIndexPort::new();
        let a1 = IndexableContent::Answer(AnswerId::new(1));
        let a2 = IndexableContent::Answer(AnswerId::new(3));

        port.notify_content_changed(a1);
        port.notify_content_changed(a2);

        let indexed = port.get_indexed_content();
        assert_eq!(indexed.len(), 2);
        assert_eq!(indexed[0], a1);
        assert_eq!(indexed[1], a2);
    }

    #[test]
    fn mock_search_port_distinguishes_questions_and_answers() {
        let port = MockSearchIndexPort::new();
        let q = IndexableContent::Question(QuestionId::new(1));
        let a = IndexableContent::Answer(AnswerId::new(1));

        port.notify_content_changed(q);
        port.notify_content_changed(a);

        let indexed = port.get_indexed_content();
        assert_eq!(indexed.len(), 2);
        assert_eq!(indexed[0], q);
        assert_eq!(indexed[1], a);
        assert_ne!(indexed[0], indexed[1]);
    }

    /// Mock persistence port for testing qa-core logic.
    struct MockPersistencePort {
        storage: std::sync::Arc<std::sync::Mutex<HashMap<String, PersistableAggregate>>>,
    }

    impl MockPersistencePort {
        fn new() -> Self {
            MockPersistencePort {
                storage: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            }
        }

        fn id_key(id: AggregateId) -> String {
            match id {
                AggregateId::Question(qid) => format!("Q{}", qid.inner()),
                AggregateId::Answer(aid) => format!("A{}", aid.inner()),
            }
        }
    }

    impl PersistencePort for MockPersistencePort {
        fn persist(&self, aggregate: PersistableAggregate) -> Result<(), PersistenceError> {
            let mut storage = self.storage.lock().unwrap();
            let key = match &aggregate {
                PersistableAggregate::Question(q) => {
                    format!("Q{}", q.id().inner())
                }
                PersistableAggregate::Answer(a) => {
                    format!("A{}", a.id().inner())
                }
            };
            storage.insert(key, aggregate);
            Ok(())
        }

        fn retrieve(&self, id: AggregateId) -> Result<PersistableAggregate, PersistenceError> {
            let storage = self.storage.lock().unwrap();
            let key = Self::id_key(id);
            storage.get(&key).cloned().ok_or(PersistenceError::NotFound)
        }
    }

    #[test]
    fn mock_persistence_port_persists_and_retrieves_questions() {
        use crate::domain::body::Body;
        use std::time::SystemTime;

        let port = MockPersistencePort::new();
        let q = Question::new(
            QuestionId::new(1),
            Body::new("What is Rust?").unwrap(),
            UserId::new(100),
            SystemTime::now(),
            License::Native,
            Vec::new(),
        );

        let aggregate = PersistableAggregate::Question(q.clone());
        port.persist(aggregate).unwrap();

        let retrieved = port
            .retrieve(AggregateId::Question(QuestionId::new(1)))
            .unwrap();
        match retrieved {
            PersistableAggregate::Question(retrieved_q) => assert_eq!(retrieved_q, q),
            _ => panic!("expected question"),
        }
    }

    #[test]
    fn mock_persistence_port_persists_and_retrieves_answers() {
        use crate::domain::body::Body;
        use std::time::SystemTime;

        let port = MockPersistencePort::new();
        let a = Answer::new(
            AnswerId::new(1),
            Body::new("Rust is a systems programming language").unwrap(),
            UserId::new(200),
            SystemTime::now(),
            License::Native,
            None,
        );

        let aggregate = PersistableAggregate::Answer(a.clone());
        port.persist(aggregate).unwrap();

        let retrieved = port
            .retrieve(AggregateId::Answer(AnswerId::new(1)))
            .unwrap();
        match retrieved {
            PersistableAggregate::Answer(retrieved_a) => assert_eq!(retrieved_a, a),
            _ => panic!("expected answer"),
        }
    }

    #[test]
    fn mock_persistence_port_returns_not_found_for_missing_aggregate() {
        let port = MockPersistencePort::new();
        let result = port.retrieve(AggregateId::Question(QuestionId::new(999)));
        assert_eq!(result, Err(PersistenceError::NotFound));
    }

    #[test]
    fn mock_persistence_port_distinguishes_questions_and_answers_by_id() {
        use crate::domain::body::Body;
        use std::time::SystemTime;

        let port = MockPersistencePort::new();
        let q = Question::new(
            QuestionId::new(1),
            Body::new("Question?").unwrap(),
            UserId::new(1),
            SystemTime::now(),
            License::Native,
            Vec::new(),
        );
        let a = Answer::new(
            AnswerId::new(1),
            Body::new("Answer.").unwrap(),
            UserId::new(2),
            SystemTime::now(),
            License::Native,
            None,
        );

        port.persist(PersistableAggregate::Question(q.clone()))
            .unwrap();
        port.persist(PersistableAggregate::Answer(a.clone()))
            .unwrap();

        let q_retrieved = port
            .retrieve(AggregateId::Question(QuestionId::new(1)))
            .unwrap();
        let a_retrieved = port
            .retrieve(AggregateId::Answer(AnswerId::new(1)))
            .unwrap();

        match q_retrieved {
            PersistableAggregate::Question(q_ret) => assert_eq!(q_ret, q),
            _ => panic!("expected question"),
        }
        match a_retrieved {
            PersistableAggregate::Answer(a_ret) => assert_eq!(a_ret, a),
            _ => panic!("expected answer"),
        }
    }
}

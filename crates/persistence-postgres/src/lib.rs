//! Postgres adapter for the PersistencePort.
//!
//! This crate implements the PersistencePort interface with a Postgres backend,
//! providing durable storage for questions and answers with full round-trip fidelity.
//! The schema mirrors the SQLite adapter (persistence-sqlite) so both adapters can
//! be validated by the same conformance test suite (task 2.1.3).

use postgres::{Client, NoTls};
use qa_core::domain::answer::Answer;
use qa_core::domain::body::Body;
use qa_core::domain::credential::AuthoritySnapshot;
use qa_core::domain::id::{AnswerId, QuestionId, UserId};
use qa_core::domain::license::License;
use qa_core::domain::ports::{
    AggregateId, PersistableAggregate, PersistenceError, PersistencePort,
};
use qa_core::domain::question::{Question, Revision};
use qa_core::domain::tag::Tag;
use qa_core::domain::vote::{CastVote, Vote};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::time::SystemTime;

/// Postgres-backed persistence adapter.
///
/// The `client` field is wrapped in `RefCell` to allow the `PersistencePort` trait
/// (which takes `&self`) to drive the underlying mutable `postgres::Client`.
/// This adapter is intended for single-threaded use; it does not implement `Sync`.
pub struct PostgresPersistence {
    client: RefCell<Client>,
}

impl PostgresPersistence {
    /// Create a new Postgres persistence adapter with the given connection string.
    ///
    /// The schema is initialized on connection. The connection string follows the
    /// standard Postgres format: `postgres://user:password@host/dbname`.
    pub fn new(connection_string: &str) -> Result<Self, PersistenceError> {
        let client = Client::connect(connection_string, NoTls)
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        let adapter = PostgresPersistence {
            client: RefCell::new(client),
        };
        adapter.init_schema()?;
        Ok(adapter)
    }

    fn init_schema(&self) -> Result<(), PersistenceError> {
        self.client
            .borrow_mut()
            .batch_execute(
                r#"
            CREATE TABLE IF NOT EXISTS questions (
                id BIGINT PRIMARY KEY,
                current_body TEXT NOT NULL,
                author_id BIGINT NOT NULL,
                created_at_secs BIGINT NOT NULL,
                created_at_nanos INTEGER NOT NULL,
                license TEXT NOT NULL,
                tags_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS question_revisions (
                id BIGSERIAL PRIMARY KEY,
                question_id BIGINT NOT NULL,
                body TEXT NOT NULL,
                created_at_secs BIGINT NOT NULL,
                created_at_nanos INTEGER NOT NULL,
                FOREIGN KEY(question_id) REFERENCES questions(id)
            );

            CREATE TABLE IF NOT EXISTS answers (
                id BIGINT PRIMARY KEY,
                current_body TEXT NOT NULL,
                author_id BIGINT NOT NULL,
                created_at_secs BIGINT NOT NULL,
                created_at_nanos INTEGER NOT NULL,
                license TEXT NOT NULL,
                credential_json TEXT
            );

            CREATE TABLE IF NOT EXISTS answer_revisions (
                id BIGSERIAL PRIMARY KEY,
                answer_id BIGINT NOT NULL,
                body TEXT NOT NULL,
                created_at_secs BIGINT NOT NULL,
                created_at_nanos INTEGER NOT NULL,
                FOREIGN KEY(answer_id) REFERENCES answers(id)
            );

            CREATE TABLE IF NOT EXISTS answer_votes (
                id BIGSERIAL PRIMARY KEY,
                answer_id BIGINT NOT NULL,
                voter_id BIGINT NOT NULL,
                vote_type TEXT NOT NULL,
                UNIQUE(answer_id, voter_id, vote_type),
                FOREIGN KEY(answer_id) REFERENCES answers(id)
            );
        "#,
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))
    }
}

/// Helper to serialize/deserialize SystemTime to/from seconds and nanoseconds.
fn system_time_to_parts(time: SystemTime) -> Result<(i64, i32), PersistenceError> {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| PersistenceError::SerializationError)?;
    Ok((duration.as_secs() as i64, duration.subsec_nanos() as i32))
}

fn parts_to_system_time(secs: i64, nanos: i32) -> Result<SystemTime, PersistenceError> {
    if !(0..1_000_000_000).contains(&nanos) {
        return Err(PersistenceError::SerializationError);
    }
    SystemTime::UNIX_EPOCH
        .checked_add(std::time::Duration::new(secs as u64, nanos as u32))
        .ok_or(PersistenceError::SerializationError)
}

/// Serialize a License to a string.
fn license_to_string(license: &License) -> String {
    match license {
        License::CcBySa4 => "CcBySa4".to_string(),
        License::CcBy4 => "CcBy4".to_string(),
        License::Native => "Native".to_string(),
        License::LinkOnly => "LinkOnly".to_string(),
    }
}

/// Deserialize a License from a string.
/// Validates that the string matches one of the known License enum variants.
fn string_to_license(s: &str) -> Result<License, PersistenceError> {
    match s {
        "CcBySa4" => Ok(License::CcBySa4),
        "CcBy4" => Ok(License::CcBy4),
        "Native" => Ok(License::Native),
        "LinkOnly" => Ok(License::LinkOnly),
        _ => Err(PersistenceError::SerializationError),
    }
}

/// Serialize a Vec<Tag> to JSON.
fn tags_to_json(tags: &[Tag]) -> Result<String, PersistenceError> {
    let serialized: Vec<TagSerialized> = tags.iter().map(TagSerialized::from_tag).collect();
    serde_json::to_string(&serialized).map_err(|_| PersistenceError::SerializationError)
}

/// Deserialize a Vec<Tag> from JSON.
fn json_to_tags(json: &str) -> Result<Vec<Tag>, PersistenceError> {
    let serialized: Vec<TagSerialized> =
        serde_json::from_str(json).map_err(|_| PersistenceError::SerializationError)?;
    serialized
        .iter()
        .map(TagSerialized::to_tag)
        .collect::<Result<Vec<_>, _>>()
}

#[derive(Serialize, Deserialize)]
struct TagSerialized {
    label: String,
    date: String,
    jurisdiction: String,
}

impl TagSerialized {
    fn from_tag(tag: &Tag) -> Self {
        TagSerialized {
            label: tag.label().to_string(),
            date: format!("{}", tag.date()),
            jurisdiction: tag.jurisdiction().as_str().to_string(),
        }
    }

    fn to_tag(&self) -> Result<Tag, PersistenceError> {
        let jurisdiction = qa_core::domain::tag::Jurisdiction::new(&self.jurisdiction)
            .map_err(|_| PersistenceError::SerializationError)?;
        Tag::new(&self.label, &self.date, jurisdiction)
            .map_err(|_| PersistenceError::SerializationError)
    }
}

/// Serialize AuthoritySnapshot to JSON.
fn credential_to_json(cred: &AuthoritySnapshot) -> Result<String, PersistenceError> {
    let serialized = CredentialSerialized {
        scope: format!("{}", cred.scope()),
        weight: cred.weight().value(),
    };
    serde_json::to_string(&serialized).map_err(|_| PersistenceError::SerializationError)
}

/// Deserialize AuthoritySnapshot from JSON.
fn json_to_credential(json: &str) -> Result<AuthoritySnapshot, PersistenceError> {
    let serialized: CredentialSerialized =
        serde_json::from_str(json).map_err(|_| PersistenceError::SerializationError)?;
    let scope = match serialized.scope.as_str() {
        "Clinical" => qa_core::domain::credential::CredentialScope::Clinical,
        "Engineering" => qa_core::domain::credential::CredentialScope::Engineering,
        "Research" => qa_core::domain::credential::CredentialScope::Research,
        _ => return Err(PersistenceError::SerializationError),
    };
    let weight = qa_core::domain::credential::AuthorityWeight::new(serialized.weight)
        .map_err(|_| PersistenceError::SerializationError)?;
    Ok(AuthoritySnapshot::new(scope, weight))
}

#[derive(Serialize, Deserialize)]
struct CredentialSerialized {
    scope: String,
    weight: f64,
}

/// Serialize a Vote to a string.
fn vote_to_string(vote: &Vote) -> String {
    match vote {
        Vote::Helpful => "Helpful".to_string(),
        Vote::Unhelpful => "Unhelpful".to_string(),
        Vote::StillValid => "StillValid".to_string(),
    }
}

/// Deserialize a Vote from a string.
/// Validates that the string matches one of the known Vote enum variants.
fn string_to_vote(s: &str) -> Result<Vote, PersistenceError> {
    match s {
        "Helpful" => Ok(Vote::Helpful),
        "Unhelpful" => Ok(Vote::Unhelpful),
        "StillValid" => Ok(Vote::StillValid),
        _ => Err(PersistenceError::SerializationError),
    }
}

impl PersistencePort for PostgresPersistence {
    fn persist(&self, aggregate: PersistableAggregate) -> Result<(), PersistenceError> {
        match aggregate {
            PersistableAggregate::Question(q) => self.persist_question(&q),
            PersistableAggregate::Answer(a) => self.persist_answer(&a),
        }
    }

    fn retrieve(&self, id: AggregateId) -> Result<PersistableAggregate, PersistenceError> {
        match id {
            AggregateId::Question(qid) => self
                .retrieve_question(qid)
                .map(PersistableAggregate::Question),
            AggregateId::Answer(aid) => self.retrieve_answer(aid).map(PersistableAggregate::Answer),
        }
    }
}

impl PostgresPersistence {
    fn persist_question(&self, question: &Question) -> Result<(), PersistenceError> {
        let id = question.id().inner() as i64;
        let current_body = question.current_body().as_str().to_string();
        let author_id = question.author_id().inner() as i64;
        let (created_at_secs, created_at_nanos) = system_time_to_parts(question.created_at())?;
        let license = license_to_string(&question.license());
        let tags_json = tags_to_json(question.tags())?;

        let mut client = self.client.borrow_mut();
        client
            .execute(
                r#"
            INSERT INTO questions
            (id, current_body, author_id, created_at_secs, created_at_nanos, license, tags_json)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                current_body = EXCLUDED.current_body,
                author_id = EXCLUDED.author_id,
                created_at_secs = EXCLUDED.created_at_secs,
                created_at_nanos = EXCLUDED.created_at_nanos,
                license = EXCLUDED.license,
                tags_json = EXCLUDED.tags_json
        "#,
                &[
                    &id,
                    &current_body,
                    &author_id,
                    &created_at_secs,
                    &created_at_nanos,
                    &license,
                    &tags_json,
                ],
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Clear existing revisions for this question
        client
            .execute(
                "DELETE FROM question_revisions WHERE question_id = $1",
                &[&id],
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Insert revisions
        for revision in question.revisions() {
            let body = revision.body().as_str().to_string();
            let (rev_secs, rev_nanos) = system_time_to_parts(revision.created_at())?;
            client
                .execute(
                    r#"
                INSERT INTO question_revisions
                (question_id, body, created_at_secs, created_at_nanos)
                VALUES ($1, $2, $3, $4)
            "#,
                    &[&id, &body, &rev_secs, &rev_nanos],
                )
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    fn retrieve_question(&self, id: QuestionId) -> Result<Question, PersistenceError> {
        let id_i64 = id.inner() as i64;
        let mut client = self.client.borrow_mut();

        let row = client
            .query_opt(
                r#"
            SELECT current_body, author_id, created_at_secs, created_at_nanos, license, tags_json
            FROM questions
            WHERE id = $1
        "#,
                &[&id_i64],
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?
            .ok_or(PersistenceError::NotFound)?;

        let current_body_str: String = row.get(0);
        let author_id_i64: i64 = row.get(1);
        let created_at_secs: i64 = row.get(2);
        let created_at_nanos: i32 = row.get(3);
        let license_str: String = row.get(4);
        let tags_json: String = row.get(5);

        let current_body =
            Body::new(&current_body_str).map_err(|_| PersistenceError::SerializationError)?;
        let author_id = UserId::new(author_id_i64 as u64);
        let created_at = parts_to_system_time(created_at_secs, created_at_nanos)?;
        let license = string_to_license(&license_str)?;
        let tags = json_to_tags(&tags_json)?;

        // Retrieve revisions
        let rev_rows = client
            .query(
                "SELECT body, created_at_secs, created_at_nanos FROM question_revisions WHERE question_id = $1 ORDER BY id",
                &[&id_i64],
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let mut reconstructed_revisions = Vec::new();
        for rev_row in &rev_rows {
            let body_str: String = rev_row.get(0);
            let rev_secs: i64 = rev_row.get(1);
            let rev_nanos: i32 = rev_row.get(2);
            let body = Body::new(&body_str).map_err(|_| PersistenceError::SerializationError)?;
            let rev_time = parts_to_system_time(rev_secs, rev_nanos)?;
            reconstructed_revisions.push(Revision::new(body, rev_time));
        }

        Question::from_stored(
            id,
            current_body,
            author_id,
            created_at,
            license,
            tags,
            reconstructed_revisions,
        )
        .map_err(|_| PersistenceError::SerializationError)
    }

    fn persist_answer(&self, answer: &Answer) -> Result<(), PersistenceError> {
        let id = answer.id().inner() as i64;
        let current_body = answer.current_body().as_str().to_string();
        let author_id = answer.author_id().inner() as i64;
        let (created_at_secs, created_at_nanos) = system_time_to_parts(answer.created_at())?;
        let license = license_to_string(&answer.license());
        let credential_json = answer.credential().map(credential_to_json).transpose()?;

        let mut client = self.client.borrow_mut();
        client
            .execute(
                r#"
            INSERT INTO answers
            (id, current_body, author_id, created_at_secs, created_at_nanos, license, credential_json)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                current_body = EXCLUDED.current_body,
                author_id = EXCLUDED.author_id,
                created_at_secs = EXCLUDED.created_at_secs,
                created_at_nanos = EXCLUDED.created_at_nanos,
                license = EXCLUDED.license,
                credential_json = EXCLUDED.credential_json
        "#,
                &[
                    &id,
                    &current_body,
                    &author_id,
                    &created_at_secs,
                    &created_at_nanos,
                    &license,
                    &credential_json,
                ],
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Clear existing revisions for this answer
        client
            .execute("DELETE FROM answer_revisions WHERE answer_id = $1", &[&id])
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Insert revisions
        for revision in answer.revisions() {
            let body = revision.body().as_str().to_string();
            let (rev_secs, rev_nanos) = system_time_to_parts(revision.created_at())?;
            client
                .execute(
                    r#"
                INSERT INTO answer_revisions
                (answer_id, body, created_at_secs, created_at_nanos)
                VALUES ($1, $2, $3, $4)
            "#,
                    &[&id, &body, &rev_secs, &rev_nanos],
                )
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        }

        // Clear existing votes for this answer
        client
            .execute("DELETE FROM answer_votes WHERE answer_id = $1", &[&id])
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Insert votes
        for vote in answer.votes() {
            let voter_id = vote.voter().inner() as i64;
            let vote_type = vote_to_string(&vote.vote());
            client
                .execute(
                    r#"
                INSERT INTO answer_votes
                (answer_id, voter_id, vote_type)
                VALUES ($1, $2, $3)
            "#,
                    &[&id, &voter_id, &vote_type],
                )
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    fn retrieve_answer(&self, id: AnswerId) -> Result<Answer, PersistenceError> {
        let id_i64 = id.inner() as i64;
        let mut client = self.client.borrow_mut();

        let row = client
            .query_opt(
                r#"
            SELECT current_body, author_id, created_at_secs, created_at_nanos, license, credential_json
            FROM answers
            WHERE id = $1
        "#,
                &[&id_i64],
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?
            .ok_or(PersistenceError::NotFound)?;

        let current_body_str: String = row.get(0);
        let author_id_i64: i64 = row.get(1);
        let created_at_secs: i64 = row.get(2);
        let created_at_nanos: i32 = row.get(3);
        let license_str: String = row.get(4);
        let credential_json: Option<String> = row.get(5);

        let current_body =
            Body::new(&current_body_str).map_err(|_| PersistenceError::SerializationError)?;
        let author_id = UserId::new(author_id_i64 as u64);
        let created_at = parts_to_system_time(created_at_secs, created_at_nanos)?;
        let license = string_to_license(&license_str)?;
        let credential = credential_json
            .as_ref()
            .map(|j| json_to_credential(j))
            .transpose()?;

        // Retrieve revisions
        let rev_rows = client
            .query(
                "SELECT body, created_at_secs, created_at_nanos FROM answer_revisions WHERE answer_id = $1 ORDER BY id",
                &[&id_i64],
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let mut reconstructed_revisions = Vec::new();
        for rev_row in &rev_rows {
            let body_str: String = rev_row.get(0);
            let rev_secs: i64 = rev_row.get(1);
            let rev_nanos: i32 = rev_row.get(2);
            let body = Body::new(&body_str).map_err(|_| PersistenceError::SerializationError)?;
            let rev_time = parts_to_system_time(rev_secs, rev_nanos)?;
            reconstructed_revisions.push(Revision::new(body, rev_time));
        }

        // Retrieve votes
        let vote_rows = client
            .query(
                "SELECT voter_id, vote_type FROM answer_votes WHERE answer_id = $1 ORDER BY id",
                &[&id_i64],
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let mut reconstructed_votes = Vec::new();
        for vote_row in &vote_rows {
            let voter_id_i64: i64 = vote_row.get(0);
            let vote_type_str: String = vote_row.get(1);
            let voter = UserId::new(voter_id_i64 as u64);
            let vote = string_to_vote(&vote_type_str)?;
            reconstructed_votes.push(CastVote::new(voter, vote));
        }

        Answer::from_stored(
            id,
            current_body,
            author_id,
            created_at,
            license,
            credential,
            reconstructed_revisions,
            reconstructed_votes,
        )
        .map_err(|_| PersistenceError::SerializationError)
    }
}

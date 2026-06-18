//! SQLite adapter for the PersistencePort.
//!
//! This crate implements the PersistencePort interface with a SQLite backend,
//! providing durable storage for questions and answers with full round-trip fidelity.

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
use rusqlite::params;
use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// SQLite-backed persistence adapter.
pub struct SqlitePersistence {
    conn: Connection,
}

impl SqlitePersistence {
    /// Create a new SQLite persistence adapter with the given database path.
    ///
    /// The database file is created if it does not exist. The schema is initialized
    /// on first access.
    pub fn new(db_path: &str) -> Result<Self, PersistenceError> {
        let conn = Connection::open(db_path)
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        let adapter = SqlitePersistence { conn };
        adapter.init_schema()?;
        Ok(adapter)
    }

    /// Create a new in-memory SQLite persistence adapter (for testing).
    pub fn new_memory() -> Result<Self, PersistenceError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        let adapter = SqlitePersistence { conn };
        adapter.init_schema()?;
        Ok(adapter)
    }

    fn init_schema(&self) -> Result<(), PersistenceError> {
        self.conn
            .execute_batch(
                r#"
            CREATE TABLE IF NOT EXISTS questions (
                id INTEGER PRIMARY KEY,
                current_body TEXT NOT NULL,
                author_id INTEGER NOT NULL,
                created_at_secs INTEGER NOT NULL,
                created_at_nanos INTEGER NOT NULL,
                license TEXT NOT NULL,
                tags_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS question_revisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                question_id INTEGER NOT NULL,
                body TEXT NOT NULL,
                created_at_secs INTEGER NOT NULL,
                created_at_nanos INTEGER NOT NULL,
                FOREIGN KEY(question_id) REFERENCES questions(id)
            );

            CREATE TABLE IF NOT EXISTS answers (
                id INTEGER PRIMARY KEY,
                current_body TEXT NOT NULL,
                author_id INTEGER NOT NULL,
                created_at_secs INTEGER NOT NULL,
                created_at_nanos INTEGER NOT NULL,
                license TEXT NOT NULL,
                credential_json TEXT
            );

            CREATE TABLE IF NOT EXISTS answer_revisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                answer_id INTEGER NOT NULL,
                body TEXT NOT NULL,
                created_at_secs INTEGER NOT NULL,
                created_at_nanos INTEGER NOT NULL,
                FOREIGN KEY(answer_id) REFERENCES answers(id)
            );

            CREATE TABLE IF NOT EXISTS answer_votes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                answer_id INTEGER NOT NULL,
                voter_id INTEGER NOT NULL,
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
    if nanos < 0 || nanos >= 1_000_000_000 {
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

impl PersistencePort for SqlitePersistence {
    fn persist(&self, aggregate: PersistableAggregate) -> Result<(), PersistenceError> {
        match aggregate {
            PersistableAggregate::Question(q) => self.persist_question(&q),
            PersistableAggregate::Answer(a) => self.persist_answer(&a),
        }
    }

    fn retrieve(&self, id: AggregateId) -> Result<PersistableAggregate, PersistenceError> {
        match id {
            AggregateId::Question(qid) => self.retrieve_question(qid).map(PersistableAggregate::Question),
            AggregateId::Answer(aid) => self.retrieve_answer(aid).map(PersistableAggregate::Answer),
        }
    }
}

impl SqlitePersistence {
    fn persist_question(&self, question: &Question) -> Result<(), PersistenceError> {
        let id = question.id().inner() as i64;
        let current_body = question.current_body().as_str();
        let author_id = question.author_id().inner() as i64;
        let (created_at_secs, created_at_nanos) = system_time_to_parts(question.created_at())?;
        let license = license_to_string(&question.license());
        let tags_json = tags_to_json(question.tags())?;

        self.conn
            .execute(
                r#"
            INSERT OR REPLACE INTO questions
            (id, current_body, author_id, created_at_secs, created_at_nanos, license, tags_json)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
                params![id, current_body, author_id, created_at_secs, created_at_nanos, license, tags_json],
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Clear existing revisions for this question
        self.conn
            .execute("DELETE FROM question_revisions WHERE question_id = ?", params![id])
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Insert revisions
        for revision in question.revisions() {
            let body = revision.body().as_str();
            let (rev_secs, rev_nanos) = system_time_to_parts(revision.created_at())?;
            self.conn
                .execute(
                    r#"
                INSERT INTO question_revisions
                (question_id, body, created_at_secs, created_at_nanos)
                VALUES (?, ?, ?, ?)
            "#,
                    params![id, body, rev_secs, rev_nanos],
                )
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    fn retrieve_question(&self, id: QuestionId) -> Result<Question, PersistenceError> {
        let id_i64 = id.inner() as i64;
        let mut stmt = self
            .conn
            .prepare(
                r#"
            SELECT current_body, author_id, created_at_secs, created_at_nanos, license, tags_json
            FROM questions
            WHERE id = ?
        "#,
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let question = stmt
            .query_row(params![id_i64], |row| {
                let current_body: String = row.get(0)?;
                let author_id: i64 = row.get(1)?;
                let created_at_secs: i64 = row.get(2)?;
                let created_at_nanos: i32 = row.get(3)?;
                let license: String = row.get(4)?;
                let tags_json: String = row.get(5)?;

                Ok((
                    current_body,
                    author_id,
                    created_at_secs,
                    created_at_nanos,
                    license,
                    tags_json,
                ))
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => PersistenceError::NotFound,
                _ => PersistenceError::DatabaseError(e.to_string()),
            })?;

        let current_body = Body::new(&question.0)
            .map_err(|_| PersistenceError::SerializationError)?;
        let author_id = UserId::new(question.1 as u64);
        let created_at = parts_to_system_time(question.2, question.3)?;
        let license = string_to_license(&question.4)?;
        let tags = json_to_tags(&question.5)?;

        // Retrieve revisions
        let mut rev_stmt = self
            .conn
            .prepare("SELECT body, created_at_secs, created_at_nanos FROM question_revisions WHERE question_id = ? ORDER BY id")
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let revisions = rev_stmt
            .query_map(params![id_i64], |row| {
                let body: String = row.get(0)?;
                let created_at_secs: i64 = row.get(1)?;
                let created_at_nanos: i32 = row.get(2)?;
                Ok((body, created_at_secs, created_at_nanos))
            })
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?
            .collect::<SqliteResult<Vec<_>>>()
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let mut reconstructed_revisions = Vec::new();
        for (body_str, rev_secs, rev_nanos) in revisions {
            let body =
                Body::new(&body_str).map_err(|_| PersistenceError::SerializationError)?;
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
        let current_body = answer.current_body().as_str();
        let author_id = answer.author_id().inner() as i64;
        let (created_at_secs, created_at_nanos) = system_time_to_parts(answer.created_at())?;
        let license = license_to_string(&answer.license());
        let credential_json = answer
            .credential()
            .map(|c| credential_to_json(c))
            .transpose()?;

        self.conn
            .execute(
                r#"
            INSERT OR REPLACE INTO answers
            (id, current_body, author_id, created_at_secs, created_at_nanos, license, credential_json)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
                params![
                    id,
                    current_body,
                    author_id,
                    created_at_secs,
                    created_at_nanos,
                    license,
                    credential_json
                ],
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Clear existing revisions for this answer
        self.conn
            .execute("DELETE FROM answer_revisions WHERE answer_id = ?", params![id])
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Insert revisions
        for revision in answer.revisions() {
            let body = revision.body().as_str();
            let (rev_secs, rev_nanos) = system_time_to_parts(revision.created_at())?;
            self.conn
                .execute(
                    r#"
                INSERT INTO answer_revisions
                (answer_id, body, created_at_secs, created_at_nanos)
                VALUES (?, ?, ?, ?)
            "#,
                    params![id, body, rev_secs, rev_nanos],
                )
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        }

        // Clear existing votes for this answer
        self.conn
            .execute("DELETE FROM answer_votes WHERE answer_id = ?", params![id])
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Insert votes
        for vote in answer.votes() {
            let voter_id = vote.voter().inner() as i64;
            let vote_type = vote_to_string(&vote.vote());
            self.conn
                .execute(
                    r#"
                INSERT INTO answer_votes
                (answer_id, voter_id, vote_type)
                VALUES (?, ?, ?)
            "#,
                    params![id, voter_id, vote_type],
                )
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    fn retrieve_answer(&self, id: AnswerId) -> Result<Answer, PersistenceError> {
        let id_i64 = id.inner() as i64;
        let mut stmt = self
            .conn
            .prepare(
                r#"
            SELECT current_body, author_id, created_at_secs, created_at_nanos, license, credential_json
            FROM answers
            WHERE id = ?
        "#,
            )
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let answer_row = stmt
            .query_row(params![id_i64], |row| {
                let current_body: String = row.get(0)?;
                let author_id: i64 = row.get(1)?;
                let created_at_secs: i64 = row.get(2)?;
                let created_at_nanos: i32 = row.get(3)?;
                let license: String = row.get(4)?;
                let credential_json: Option<String> = row.get(5)?;

                Ok((
                    current_body,
                    author_id,
                    created_at_secs,
                    created_at_nanos,
                    license,
                    credential_json,
                ))
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => PersistenceError::NotFound,
                _ => PersistenceError::DatabaseError(e.to_string()),
            })?;

        let current_body = Body::new(&answer_row.0)
            .map_err(|_| PersistenceError::SerializationError)?;
        let author_id = UserId::new(answer_row.1 as u64);
        let created_at = parts_to_system_time(answer_row.2, answer_row.3)?;
        let license = string_to_license(&answer_row.4)?;
        let credential = answer_row
            .5
            .as_ref()
            .map(|j| json_to_credential(j))
            .transpose()?;

        // Retrieve revisions
        let mut rev_stmt = self
            .conn
            .prepare("SELECT body, created_at_secs, created_at_nanos FROM answer_revisions WHERE answer_id = ? ORDER BY id")
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let revisions = rev_stmt
            .query_map(params![id_i64], |row| {
                let body: String = row.get(0)?;
                let created_at_secs: i64 = row.get(1)?;
                let created_at_nanos: i32 = row.get(2)?;
                Ok((body, created_at_secs, created_at_nanos))
            })
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?
            .collect::<SqliteResult<Vec<_>>>()
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let mut reconstructed_revisions = Vec::new();
        for (body_str, rev_secs, rev_nanos) in revisions {
            let body =
                Body::new(&body_str).map_err(|_| PersistenceError::SerializationError)?;
            let rev_time = parts_to_system_time(rev_secs, rev_nanos)?;
            reconstructed_revisions.push(Revision::new(body, rev_time));
        }

        // Retrieve votes
        let mut vote_stmt = self
            .conn
            .prepare("SELECT voter_id, vote_type FROM answer_votes WHERE answer_id = ? ORDER BY id")
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let votes = vote_stmt
            .query_map(params![id_i64], |row| {
                let voter_id: i64 = row.get(0)?;
                let vote_type: String = row.get(1)?;
                Ok((voter_id, vote_type))
            })
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?
            .collect::<SqliteResult<Vec<_>>>()
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let mut reconstructed_votes = Vec::new();
        for (voter_id, vote_type_str) in votes {
            let voter = UserId::new(voter_id as u64);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_persist_and_retrieve_question() {
        let adapter = SqlitePersistence::new_memory().unwrap();
        let q = Question::new(
            QuestionId::new(1),
            Body::new("What is Rust?").unwrap(),
            UserId::new(100),
            SystemTime::now(),
            License::Native,
            Vec::new(),
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
    fn test_sqlite_persist_and_retrieve_answer() {
        let adapter = SqlitePersistence::new_memory().unwrap();
        let a = Answer::new(
            AnswerId::new(1),
            Body::new("Rust is a systems language").unwrap(),
            UserId::new(200),
            SystemTime::now(),
            License::Native,
            None,
        );

        let aggregate = PersistableAggregate::Answer(a.clone());
        adapter.persist(aggregate).unwrap();

        let retrieved = adapter
            .retrieve(AggregateId::Answer(AnswerId::new(1)))
            .unwrap();
        match retrieved {
            PersistableAggregate::Answer(retrieved_a) => assert_eq!(retrieved_a, a),
            _ => panic!("expected answer"),
        }
    }

    #[test]
    fn test_sqlite_returns_not_found_for_missing() {
        let adapter = SqlitePersistence::new_memory().unwrap();
        let result = adapter.retrieve(AggregateId::Question(QuestionId::new(999)));
        assert_eq!(result, Err(PersistenceError::NotFound));
    }
}

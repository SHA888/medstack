//! Common serialization utilities for persistence adapters.
//!
//! Provides canonical serialization for License, Tag, Vote, and VerifiedCredential
//! types so that multiple adapters (SQLite, Postgres) maintain consistent round-trip
//! fidelity without duplication.

use qa_core::domain::credential::AuthoritySnapshot;
use qa_core::domain::license::License;
use qa_core::domain::tag::Tag;
use qa_core::domain::vote::Vote;
use qa_core::domain::ports::PersistenceError;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Helper to serialize/deserialize SystemTime to/from seconds and nanoseconds.
pub fn system_time_to_parts(time: SystemTime) -> Result<(i64, i32), PersistenceError> {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| PersistenceError::SerializationError)?;
    Ok((duration.as_secs() as i64, duration.subsec_nanos() as i32))
}

pub fn parts_to_system_time(secs: i64, nanos: i32) -> Result<SystemTime, PersistenceError> {
    if secs < 0 {
        return Err(PersistenceError::SerializationError);
    }
    if !(0..1_000_000_000).contains(&nanos) {
        return Err(PersistenceError::SerializationError);
    }
    SystemTime::UNIX_EPOCH
        .checked_add(std::time::Duration::new(secs as u64, nanos as u32))
        .ok_or(PersistenceError::SerializationError)
}

/// Serialize a License to a string.
pub fn license_to_string(license: &License) -> String {
    match license {
        License::CcBySa4 => "CcBySa4".to_string(),
        License::CcBy4 => "CcBy4".to_string(),
        License::Native => "Native".to_string(),
        License::LinkOnly => "LinkOnly".to_string(),
    }
}

/// Deserialize a License from a string.
pub fn string_to_license(s: &str) -> Result<License, PersistenceError> {
    match s {
        "CcBySa4" => Ok(License::CcBySa4),
        "CcBy4" => Ok(License::CcBy4),
        "Native" => Ok(License::Native),
        "LinkOnly" => Ok(License::LinkOnly),
        _ => Err(PersistenceError::SerializationError),
    }
}

/// Serialize a Vec<Tag> to JSON.
pub fn tags_to_json(tags: &[Tag]) -> Result<String, PersistenceError> {
    let serialized: Vec<TagSerialized> = tags.iter().map(TagSerialized::from_tag).collect();
    serde_json::to_string(&serialized).map_err(|_| PersistenceError::SerializationError)
}

/// Deserialize a Vec<Tag> from JSON.
pub fn json_to_tags(json: &str) -> Result<Vec<Tag>, PersistenceError> {
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
pub fn credential_to_json(cred: &AuthoritySnapshot) -> Result<String, PersistenceError> {
    let serialized = CredentialSerialized {
        scope: format!("{}", cred.scope()),
        weight: cred.weight().value(),
    };
    serde_json::to_string(&serialized).map_err(|_| PersistenceError::SerializationError)
}

/// Deserialize AuthoritySnapshot from JSON.
pub fn json_to_credential(json: &str) -> Result<AuthoritySnapshot, PersistenceError> {
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
pub fn vote_to_string(vote: &Vote) -> String {
    match vote {
        Vote::Helpful => "Helpful".to_string(),
        Vote::Unhelpful => "Unhelpful".to_string(),
        Vote::StillValid => "StillValid".to_string(),
    }
}

/// Get the axis for a Vote (Quality for Helpful/Unhelpful, Perishability for StillValid).
pub fn vote_to_axis(vote: &Vote) -> String {
    match vote {
        Vote::Helpful | Vote::Unhelpful => "Quality".to_string(),
        Vote::StillValid => "Perishability".to_string(),
    }
}

/// Deserialize a Vote from a string.
pub fn string_to_vote(s: &str) -> Result<Vote, PersistenceError> {
    match s {
        "Helpful" => Ok(Vote::Helpful),
        "Unhelpful" => Ok(Vote::Unhelpful),
        "StillValid" => Ok(Vote::StillValid),
        _ => Err(PersistenceError::SerializationError),
    }
}

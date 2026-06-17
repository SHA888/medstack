//! Domain identity types: QuestionId, AnswerId, UserId, ContentId.
//!
//! These are distinct newtypes that make identity *confusion* impossible at
//! compile time: a `QuestionId` cannot be passed where an `AnswerId` is expected,
//! and the wrapped `u64` cannot be read or compared except through the typed API.
//!
//! The constructors are intentionally public: adapters (persistence, ingestion,
//! search) must mint these from stored rows when rehydrating aggregates. The
//! guarantee is type-distinctness, NOT constructor privacy — an ID carries no
//! authority on its own, so being constructible is harmless. (Contrast
//! `VerifiedCredential`, whose constructor IS private because it carries trust.)

/// Unique identifier for a question.
///
/// A distinct newtype: cannot be confused with any other ID type at compile time.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QuestionId(u64);

impl QuestionId {
    /// Construct a QuestionId (public: adapters mint these when loading rows).
    pub fn new(id: u64) -> Self {
        QuestionId(id)
    }

    /// Access the inner ID value.
    pub fn inner(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for QuestionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Q{}", self.0)
    }
}

/// Unique identifier for an answer.
///
/// A distinct newtype: cannot be confused with any other ID type at compile time.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnswerId(u64);

impl AnswerId {
    /// Construct an AnswerId (public: adapters mint these when loading rows).
    pub fn new(id: u64) -> Self {
        AnswerId(id)
    }

    /// Access the inner ID value.
    pub fn inner(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for AnswerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A{}", self.0)
    }
}

/// Unique identifier for a user.
///
/// A distinct newtype: cannot be confused with any other ID type at compile time.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(u64);

impl UserId {
    /// Construct a UserId (public: adapters mint these when loading rows).
    pub fn new(id: u64) -> Self {
        UserId(id)
    }

    /// Access the inner ID value.
    pub fn inner(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "U{}", self.0)
    }
}

/// Unique identifier for an imported piece of content that is not itself a
/// qa-core aggregate (e.g. a mirrored Stack Exchange / Biostars row).
///
/// A distinct newtype: cannot be confused with any other ID type at compile time.
/// Questions and answers are addressed by `QuestionId` / `AnswerId`; to query a
/// license across all content kinds uniformly, use `ports::ContentRef`, which
/// maps each kind into the licensing port without conflating the ID namespaces.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentId(u64);

impl ContentId {
    /// Construct a ContentId (public: adapters mint these when loading rows).
    pub fn new(id: u64) -> Self {
        ContentId(id)
    }

    /// Access the inner ID value.
    pub fn inner(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ContentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "C{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newtype_construction() {
        let q = QuestionId::new(1);
        let a = AnswerId::new(1);
        let u = UserId::new(1);

        assert_eq!(q.inner(), 1);
        assert_eq!(a.inner(), 1);
        assert_eq!(u.inner(), 1);
    }

    #[test]
    fn display_formats_with_prefix() {
        assert_eq!(format!("{}", QuestionId::new(42)), "Q42");
        assert_eq!(format!("{}", AnswerId::new(99)), "A99");
        assert_eq!(format!("{}", UserId::new(7)), "U7");
    }

    #[test]
    fn ids_are_copy() {
        let q1 = QuestionId::new(1);
        let q2 = q1; // implicit copy
        assert_eq!(q1, q2);
    }

    #[test]
    fn ids_are_comparable() {
        let q1 = QuestionId::new(1);
        let q2 = QuestionId::new(1);
        let q3 = QuestionId::new(2);

        assert_eq!(q1, q2);
        assert_ne!(q1, q3);
        assert!(q1 < q3);
        assert!(q3 > q1);
    }

    #[test]
    fn ids_are_hashable() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let q1 = QuestionId::new(1);
        let q2 = QuestionId::new(1);
        let q3 = QuestionId::new(2);

        set.insert(q1);
        assert!(set.contains(&q2)); // q1 and q2 are equal
        assert!(!set.contains(&q3));
    }

    #[test]
    fn content_id_construction() {
        let c = ContentId::new(42);
        assert_eq!(c.inner(), 42);
    }

    #[test]
    fn content_id_display() {
        assert_eq!(format!("{}", ContentId::new(42)), "C42");
    }

    #[test]
    fn content_ids_are_comparable() {
        let c1 = ContentId::new(1);
        let c2 = ContentId::new(1);
        let c3 = ContentId::new(2);

        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
        assert!(c1 < c3);
    }

    #[test]
    fn content_ids_are_hashable() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let c1 = ContentId::new(1);
        let c2 = ContentId::new(1);
        let c3 = ContentId::new(2);

        set.insert(c1);
        assert!(set.contains(&c2));
        assert!(!set.contains(&c3));
    }

    #[test]
    fn content_id_is_copy() {
        let c1 = ContentId::new(1);
        let c2 = c1; // implicit copy
        assert_eq!(c1, c2);
    }
}

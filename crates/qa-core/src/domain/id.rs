//! Domain identity types: QuestionId, AnswerId, UserId.
//!
//! These are opaque newtypes that make identity confusion impossible at compile time.
//! Only the qa-core domain module can construct them. Outside code must use the
//! provided constructors.

/// Unique identifier for a question.
///
/// Private constructor enforces that only qa-core can create question IDs.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QuestionId(u64);

impl QuestionId {
    /// Construct a new QuestionId. Only qa-core domain module can call this.
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
/// Private constructor enforces that only qa-core can create answer IDs.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnswerId(u64);

impl AnswerId {
    /// Construct a new AnswerId. Only qa-core domain module can call this.
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
/// Private constructor enforces that only qa-core can create user IDs.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(u64);

impl UserId {
    /// Construct a new UserId. Only qa-core domain module can call this.
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
}

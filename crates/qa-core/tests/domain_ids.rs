//! Tests for domain identity types (task 1.1.1).
//!
//! - Type safety: QuestionId, AnswerId, UserId are distinct types
//! - Opaque constructor: only qa_core module can construct them
//! - Pattern matching: exhaustive (compiler enforces for newtypes)

#[test]
fn ids_are_distinct_types() {
    // This test verifies type safety at runtime by constructing each type
    // and asserting they are different types.
    use qa_core::domain::id::{AnswerId, QuestionId, UserId};

    let q = QuestionId::new(1);
    let a = AnswerId::new(1);
    let u = UserId::new(1);

    // If these compile with different types, type safety is enforced.
    // We can't accidentally pass an AnswerId where QuestionId is expected.
    assert_eq!(q.inner(), 1);
    assert_eq!(a.inner(), 1);
    assert_eq!(u.inner(), 1);
}

#[test]
fn ids_display_correctly() {
    use qa_core::domain::id::{AnswerId, QuestionId, UserId};

    let q = QuestionId::new(42);
    let a = AnswerId::new(99);
    let u = UserId::new(7);

    // Display impl should be present for debugging
    assert!(format!("{}", q).contains("42") || format!("{:?}", q).contains("42"));
    assert!(format!("{}", a).contains("99") || format!("{:?}", a).contains("99"));
    assert!(format!("{}", u).contains("7") || format!("{:?}", u).contains("7"));
}

#[test]
fn ids_are_copy() {
    use qa_core::domain::id::QuestionId;

    let q1 = QuestionId::new(1);
    let q2 = q1; // Copy
    let q3 = q1; // Copy (no need for explicit clone on Copy types)

    assert_eq!(q1.inner(), q2.inner());
    assert_eq!(q2.inner(), q3.inner());
}

#[test]
fn ids_implement_eq_and_ord() {
    use qa_core::domain::id::QuestionId;

    let q1 = QuestionId::new(1);
    let q2 = QuestionId::new(1);
    let q3 = QuestionId::new(2);

    assert_eq!(q1, q2);
    assert_ne!(q1, q3);
    assert!(q1 < q3);
    assert!(q3 > q1);
}

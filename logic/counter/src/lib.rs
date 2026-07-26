//! Counter business logic — the Rust side of the nui counter example.
//!
//! nui owns the state; this crate owns the logic: pure, typed functions the
//! UI calls through the generated bridge. The expected signatures — and a
//! compile-time check that they exist — live in `generated.rs`, produced
//! from the same `.nui` file as the UI, so the two sides cannot drift.
//!
//! Everything in this file is the actual business logic; there is no other
//! handwritten code anywhere in the pipeline.

uniffi::setup_scaffolding!();

mod generated;

#[uniffi::export]
pub fn counter_increment(count: i64) -> i64 {
    count.saturating_add(1)
}

#[uniffi::export]
pub fn counter_decrement(count: i64) -> i64 {
    count.saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increments() {
        assert_eq!(counter_increment(0), 1);
    }

    #[test]
    fn decrements() {
        assert_eq!(counter_decrement(5), 4);
    }

    #[test]
    fn saturates_instead_of_overflowing() {
        assert_eq!(counter_increment(i64::MAX), i64::MAX);
    }
}

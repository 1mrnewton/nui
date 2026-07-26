//! Example business logic — the Rust side of the nui demo apps.
//!
//! nui owns the state; this crate owns the logic: pure, typed functions the
//! UI calls through the generated bridge. The expected signatures — and a
//! compile-time check that they exist — live in `generated.rs` (Counter) and
//! `generated_toggle.rs` (Toggle), produced from the same `.nui` files as
//! the UI, so the two sides cannot drift.
//!
//! Everything in this file is the actual business logic; there is no other
//! handwritten code anywhere in the pipeline.

uniffi::setup_scaffolding!();

mod generated;
mod generated_profile;
mod generated_toggle;

// Record types come from the generated interface; only the fn bodies here
// are handwritten.
pub use generated_profile::ProfilePerson;

#[uniffi::export]
pub fn counter_increment(count: i64) -> i64 {
    count.saturating_add(1)
}

#[uniffi::export]
pub fn counter_decrement(count: i64) -> i64 {
    count.saturating_sub(1)
}

#[uniffi::export]
pub fn toggle_toggle(value: bool) -> bool {
    !value
}

/// Pure: the next person is a function of the current one (a fixed cycle),
/// so the same input always gives the same output.
#[uniffi::export]
pub fn profile_next(current: ProfilePerson) -> ProfilePerson {
    const PEOPLE: [(&str, &str); 3] = [
        (
            "Ada Lovelace",
            "Wrote the first program, a century before the hardware.",
        ),
        ("Grace Hopper", "Built the first compiler and coined the bug."),
        ("Alan Turing", "Defined computation itself."),
    ];
    let index = PEOPLE
        .iter()
        .position(|(name, _)| *name == current.name)
        .unwrap_or(PEOPLE.len() - 1);
    let (name, bio) = PEOPLE[(index + 1) % PEOPLE.len()];
    ProfilePerson {
        name: name.into(),
        bio: bio.into(),
    }
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

    #[test]
    fn toggles() {
        assert!(toggle_toggle(false));
        assert!(!toggle_toggle(true));
    }

    #[test]
    fn cycles_through_people_and_back() {
        let ada = ProfilePerson {
            name: "Ada Lovelace".into(),
            bio: String::new(),
        };
        let grace = profile_next(ada.clone());
        assert_eq!(grace.name, "Grace Hopper");
        let alan = profile_next(grace);
        assert_eq!(alan.name, "Alan Turing");
        assert_eq!(profile_next(alan).name, "Ada Lovelace");
    }
}

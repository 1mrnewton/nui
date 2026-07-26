//! Tests for the logic-side backends: Rust signature checks and the Swift
//! bridge.

use nui::{compile, rust_logic, swift_bridge};

const COUNTER: &str = include_str!("../examples/counter.nui");

#[test]
fn generates_rust_signature_checks_for_counter() {
    let doc = compile(COUNTER).unwrap();
    let source = rust_logic::generate(&doc);
    for expected in [
        "//     #[uniffi::export]",
        "//     pub fn counter_increment(count: i64) -> i64",
        "//     pub fn counter_decrement(count: i64) -> i64",
        "fn _nui_check_logic_signatures() {",
        "let _: fn(i64) -> i64 = crate::counter_increment;",
        "let _: fn(i64) -> i64 = crate::counter_decrement;",
    ] {
        assert!(
            source.contains(expected),
            "missing {expected:?} in generated Rust:\n{source}"
        );
    }
}

#[test]
fn generates_swift_bridge_for_counter() {
    let doc = compile(COUNTER).unwrap();
    let source = swift_bridge::generate(&doc);
    for expected in [
        "struct RustCounterLogic: CounterLogic {",
        "func increment(count: Int) async -> Int {",
        "Int(counterIncrement(count: Int64(count)))",
        "func decrement(count: Int) async -> Int {",
        "Int(counterDecrement(count: Int64(count)))",
    ] {
        assert!(
            source.contains(expected),
            "missing {expected:?} in generated bridge:\n{source}"
        );
    }
}

#[test]
fn multi_word_names_convert_across_all_boundaries() {
    let doc = compile(
        r#"
        component TodoList {
            state title: String = "hi"
            logic { fn fetchTitle(oldTitle: String) -> String }
            Button { label: "go", on_click: { title = fetchTitle(title) } }
        }
        "#,
    )
    .unwrap();
    let rust = rust_logic::generate(&doc);
    assert!(
        rust.contains("pub fn todo_list_fetch_title(old_title: String) -> String"),
        "{rust}"
    );
    assert!(
        rust.contains("let _: fn(String) -> String = crate::todo_list_fetch_title;"),
        "{rust}"
    );
    let bridge = swift_bridge::generate(&doc);
    assert!(
        bridge.contains("todoListFetchTitle(oldTitle: oldTitle)"),
        "{bridge}"
    );
    assert!(
        bridge.contains("func fetchTitle(oldTitle: String) async -> String {"),
        "{bridge}"
    );
}

#[test]
fn non_int_types_pass_through_without_conversion() {
    let doc = compile(
        r#"
        component X {
            state on: Bool = false
            logic { fn toggle(on: Bool) -> Bool }
            Button { label: "t", on_click: { on = toggle(on) } }
        }
        "#,
    )
    .unwrap();
    let bridge = swift_bridge::generate(&doc);
    assert!(bridge.contains("xToggle(on: on)"), "{bridge}");
}

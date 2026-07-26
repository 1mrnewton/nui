//! Tests for the Swift backend: transpile and check the emitted source.

use nui::{compile, swift};

const COUNTER: &str = include_str!("../examples/counter.nui");

#[test]
fn generates_swiftui_source_for_counter() {
    let doc = compile(COUNTER).unwrap();
    let source = swift::generate(&doc);
    for expected in [
        "import SwiftUI",
        "public struct CounterState: Equatable, Sendable {",
        "public var count: Int",
        "public init(count: Int = 0) {",
        "public protocol CounterLogic: Sendable {",
        "func increment(count: Int) async -> Int",
        "func decrement(count: Int) async -> Int",
        "@Observable @MainActor",
        "public final class CounterStore {",
        "public func incrementCount() {",
        "state.count = await logic.increment(count: state.count)",
        "public struct CounterView: View {",
        "VStack(spacing: 16) {",
        r#"Text("Count: \(store.state.count)")"#,
        ".font(.title)",
        r#"Button("-") { store.decrementCount() }"#,
        r#"Button("+") { store.incrementCount() }"#,
        ".padding(24)",
        "private struct CounterPreviewLogic: CounterLogic {",
        "func increment(count: Int) async -> Int { 0 }",
        "#Preview {",
    ] {
        assert!(
            source.contains(expected),
            "missing {expected:?} in generated Swift:\n{source}"
        );
    }
}

#[test]
fn container_modifiers_follow_the_closing_brace() {
    let doc = compile(COUNTER).unwrap();
    let source = swift::generate(&doc);
    // The root VStack's .padding must come after its closing brace, at the
    // same indent, so it applies to the whole stack.
    assert!(
        source.contains("        }\n        .padding(24)"),
        "padding not attached to the VStack:\n{source}"
    );
}

#[test]
fn identical_actions_share_one_store_method() {
    let source_nui = r#"
        component X {
            state n: Int = 0
            logic { fn bump(n: Int) -> Int }
            VStack {
                Button { label: "a", on_click: { n = bump(n) } }
                Button { label: "b", on_click: { n = bump(n) } }
            }
        }
    "#;
    let doc = compile(source_nui).unwrap();
    let source = swift::generate(&doc);
    assert_eq!(
        source.matches("public func bumpN() {").count(),
        1,
        "identical actions should be deduplicated:\n{source}"
    );
}

#[test]
fn literal_call_args_are_emitted() {
    let source_nui = r#"
        component X {
            state n: Int = 0
            logic { fn add(n: Int, amount: Int) -> Int }
            Button { label: "+10", on_click: { n = add(n, 10) } }
        }
    "#;
    let doc = compile(source_nui).unwrap();
    let source = swift::generate(&doc);
    assert!(
        source.contains("state.n = await logic.add(n: state.n, amount: 10)"),
        "bad call emission:\n{source}"
    );
}

#[test]
fn generates_every_view_kind() {
    let source_nui = r#"
        component Kitchen {
            state name: String = ""
            logic { fn shout(text: String) -> String }
            VStack {
                TextField { bind: name, placeholder: "Your name" }
                Image { source: "logo" }
                List {
                    Text { text: "a" }
                    Text { text: name }
                }
                Spacer
                Button {
                    label: "Go"
                    on_click: { name = shout(name) }
                    style: { color: blue }
                }
                HStack {
                    spacing: 4.5
                    Text { text: "x" }
                }
            }
        }
    "#;
    let doc = compile(source_nui).unwrap();
    let source = swift::generate(&doc);
    for expected in [
        r#"TextField("Your name", text: $store.state.name)"#,
        r#"Image("logo")"#,
        "List {",
        r#"Text("\(store.state.name)")"#,
        "Spacer()",
        ".foregroundStyle(.blue)",
        "HStack(spacing: 4.5) {",
        "public init(name: String = \"\") {",
        "state.name = await logic.shout(text: state.name)",
    ] {
        assert!(
            source.contains(expected),
            "missing {expected:?} in generated Swift:\n{source}"
        );
    }
}

#[test]
fn escapes_string_literals() {
    let doc = compile(r#"component X { Text { text: "say \"hi\"\n\{ok}" } }"#).unwrap();
    let source = swift::generate(&doc);
    assert!(
        source.contains(r#"Text("say \"hi\"\n{ok}")"#),
        "bad escaping:\n{source}"
    );
}

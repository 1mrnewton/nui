//! Tests for the experimental UIKit backend.

use nui::{compile, uikit};

const COUNTER: &str = include_str!("../examples/counter.nui");

#[test]
fn generates_uikit_source_for_counter() {
    let doc = compile(COUNTER).unwrap();
    let source = uikit::generate(&doc);
    for expected in [
        "import UIKit",
        // State and logic protocol are identical to the SwiftUI backend,
        // so the generated Rust bridge works with either UI.
        "public struct CounterState: Equatable, Sendable {",
        "public protocol CounterLogic: Sendable {",
        "func increment(count: Int) async -> Int",
        "@MainActor",
        "public final class CounterStore {",
        "public private(set) var state: CounterState",
        "public var onChange: (@MainActor () -> Void)?",
        "state.count = await logic.increment(count: state.count)",
        "onChange?()",
        "public final class CounterViewController: UIViewController {",
        "private let label0 = UILabel()",
        "UIFont.preferredFont(forTextStyle: .title1)",
        r#"UIButton(type: .system, primaryAction: UIAction(title: "+") { [weak self] _ in"#,
        "self?.store.incrementCount()",
        "UIStackView(arrangedSubviews: [button",
        ".axis = .vertical",
        ".axis = .horizontal",
        ".spacing = 16",
        "nuiPad(stack",
        r#"label0.text = "Count: \(store.state.count)""#,
        "store.onChange = { [weak self] in self?.applyState() }",
        "#Preview {",
        "CounterViewController(store: CounterStore(logic: CounterPreviewLogic()))",
    ] {
        assert!(
            source.contains(expected),
            "missing {expected:?} in generated UIKit source:\n{source}"
        );
    }
    // No SwiftUI anywhere.
    assert!(!source.contains("SwiftUI"), "UIKit backend must not import SwiftUI:\n{source}");
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
    let source = uikit::generate(&doc);
    for expected in [
        // TextField: stored property, placeholder, edit wiring, guarded apply.
        "private let textField0 = UITextField()",
        r#"textField0.placeholder = "Your name""#,
        "textField0.borderStyle = .roundedRect",
        "self.store.setName(self.textField0.text ?? \"\")",
        "}, for: .editingChanged)",
        "if textField0.text != store.state.name { textField0.text = store.state.name }",
        // The store grows a setter for the bound state.
        "public func setName(_ value: String) {",
        r#"let image1 = UIImageView(image: UIImage(named: "logo"))"#,
        // Dynamic text inside List still becomes a stored label.
        r#"label3.text = "\(store.state.name)""#,
        "setContentHuggingPriority(.defaultLow, for: .vertical)",
        "tintColor = .systemBlue",
        ".spacing = 4.5",
        ".alignment = .fill",
        "state.name = await logic.shout(text: state.name)",
    ] {
        assert!(
            source.contains(expected),
            "missing {expected:?} in generated UIKit source:\n{source}"
        );
    }
}

#[test]
fn dynamic_button_labels_update_in_apply_state() {
    let source_nui = r#"
        component X {
            state label: String = "tap"
            logic { fn next(label: String) -> String }
            Button {
                label: "{label}"
                on_click: { label = next(label) }
            }
        }
    "#;
    let doc = compile(source_nui).unwrap();
    let source = uikit::generate(&doc);
    for expected in [
        "private let button0 = UIButton(type: .system)",
        "button0.addAction(UIAction { [weak self] _ in self?.store.nextLabel() }, for: .touchUpInside)",
        r#"button0.setTitle("\(store.state.label)", for: .normal)"#,
    ] {
        assert!(
            source.contains(expected),
            "missing {expected:?} in generated UIKit source:\n{source}"
        );
    }
}

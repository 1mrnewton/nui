//! End-to-end tests: compile the counter example and check the IR.

use nui::{compile, ir};

const COUNTER: &str = include_str!("../examples/counter.nui");

#[test]
fn compiles_counter_example() {
    let doc = compile(COUNTER).expect("counter example should compile");
    let component = &doc.component;

    assert_eq!(doc.format_version, ir::FORMAT_VERSION);
    assert_eq!(component.name, "Counter");

    assert_eq!(component.state.len(), 1);
    assert_eq!(component.state[0].name, "count");
    assert_eq!(component.state[0].ty, ir::Type::Int);
    assert_eq!(component.state[0].initial, ir::Value::Int(0));

    assert_eq!(component.functions.len(), 2);
    assert_eq!(component.functions[0].name, "increment");
    assert_eq!(component.functions[0].params.len(), 1);
    assert_eq!(component.functions[0].params[0].name, "count");
    assert_eq!(component.functions[0].params[0].ty, ir::Type::Int);
    assert_eq!(component.functions[0].returns, ir::Type::Int);

    let ir::Node::VStack {
        spacing,
        children,
        modifiers,
    } = &component.root
    else {
        panic!("expected VStack root, got {:?}", component.root);
    };
    assert_eq!(*spacing, Some(16.0));
    assert_eq!(modifiers, &[ir::Modifier::Padding { value: 24.0 }]);
    assert_eq!(children.len(), 2);

    let ir::Node::Text { content, modifiers } = &children[0] else {
        panic!("expected Text, got {:?}", children[0]);
    };
    assert_eq!(
        content.0,
        [
            ir::TextSegment::Literal {
                value: "Count: ".into()
            },
            ir::TextSegment::State {
                name: "count".into()
            },
        ]
    );
    assert_eq!(
        modifiers,
        &[ir::Modifier::Font {
            style: ir::FontStyle::Title
        }]
    );

    let ir::Node::HStack { children, .. } = &children[1] else {
        panic!("expected HStack, got {:?}", children[1]);
    };
    assert_eq!(children.len(), 2);
    let ir::Node::Button { action, .. } = &children[1] else {
        panic!("expected Button, got {:?}", children[1]);
    };
    assert_eq!(
        action,
        &ir::Action {
            state: "count".into(),
            function: "increment".into(),
            args: vec![ir::CallArg::State {
                name: "count".into()
            }],
        }
    );
}

#[test]
fn ir_round_trips_through_json() {
    let doc = compile(COUNTER).unwrap();
    let json = serde_json::to_string(&doc).unwrap();
    let back: ir::Document = serde_json::from_str(&json).unwrap();
    assert_eq!(doc, back);
}

#[test]
fn rejects_unknown_state_reference() {
    let source = r#"component X { Text { text: "{missing}" } }"#;
    let err = compile(source).unwrap_err();
    assert!(err.message.contains("missing"), "got: {}", err.message);
}

#[test]
fn rejects_undeclared_function() {
    let source = r#"
        component X {
            state n: Int = 0
            Button { label: "hi", on_click: { n = nope(n) } }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(err.message.contains("nope"), "got: {}", err.message);
}

#[test]
fn rejects_return_type_mismatch() {
    let source = r#"
        component X {
            state n: Int = 0
            logic { fn label(n: Int) -> String }
            Button { label: "hi", on_click: { n = label(n) } }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("returns String") && err.message.contains("is Int"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_argument_type_mismatch() {
    let source = r#"
        component X {
            state n: Int = 0
            state title: String = ""
            logic { fn bump(n: Int) -> Int }
            Button { label: "hi", on_click: { n = bump(title) } }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(err.message.contains("expects Int"), "got: {}", err.message);
}

#[test]
fn rejects_wrong_argument_count() {
    let source = r#"
        component X {
            state n: Int = 0
            logic { fn bump(n: Int) -> Int }
            Button { label: "hi", on_click: { n = bump(n, 2) } }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("takes 1 argument(s), found 2"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_unknown_view() {
    let source = "component X { Carousel { Spacer } }";
    let err = compile(source).unwrap_err();
    assert!(err.message.contains("Carousel"), "got: {}", err.message);
}

#[test]
fn rejects_wrong_initial_value_type() {
    let source = r#"
        component X {
            state count: Int = "zero"
            Spacer
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(err.message.contains("Int"), "got: {}", err.message);
}

#[test]
fn textfield_binding_must_be_string_state() {
    let source = r#"
        component X {
            state count: Int = 0
            TextField { bind: count }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(err.message.contains("String"), "got: {}", err.message);
}

#[test]
fn rejects_unknown_property() {
    let source = r#"
        component X {
            state count: Int = 0
            Text { text: "hi", size: 12 }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(err.message.contains("size"), "got: {}", err.message);
}

#[test]
fn rejects_unknown_style_property() {
    let source = r#"
        component X {
            Text { text: "hi", style: { shadow: 4 } }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(err.message.contains("shadow"), "got: {}", err.message);
}

#[test]
fn literal_arguments_are_allowed() {
    let source = r#"
        component X {
            state n: Int = 0
            logic { fn add(n: Int, amount: Int) -> Int }
            Button { label: "+10", on_click: { n = add(n, 10) } }
        }
    "#;
    let doc = compile(source).unwrap();
    let ir::Node::Button { action, .. } = &doc.component.root else {
        panic!("expected Button root");
    };
    assert_eq!(
        action.args,
        vec![
            ir::CallArg::State { name: "n".into() },
            ir::CallArg::Value {
                value: ir::Value::Int(10)
            },
        ]
    );
}

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
fn compiles_toggle_example_with_if() {
    let source = include_str!("../examples/toggle.nui");
    let doc = compile(source).expect("toggle example should compile");
    let ir::Node::VStack { children, .. } = &doc.component.root else {
        panic!("expected VStack root");
    };
    let ir::Node::If {
        condition,
        then_children,
        else_children,
    } = &children[1]
    else {
        panic!("expected an if node, got {:?}", children[1]);
    };
    assert_eq!(condition, "showHint");
    assert_eq!(then_children.len(), 1);
    assert_eq!(else_children.len(), 1);

    // The if node survives a JSON round-trip like everything else.
    let json = serde_json::to_string(&doc).unwrap();
    let back: ir::Document = serde_json::from_str(&json).unwrap();
    assert_eq!(doc, back);
}

#[test]
fn rejects_unknown_if_condition() {
    let source = r#"
        component X {
            VStack {
                if missing { Spacer }
            }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(err.message.contains("missing"), "got: {}", err.message);
}

#[test]
fn rejects_non_bool_if_condition() {
    let source = r#"
        component X {
            state count: Int = 0
            VStack {
                if count { Spacer }
            }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("needs a Bool") && err.message.contains("is Int"),
        "got: {}",
        err.message
    );
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

// --- record types ---

#[test]
fn compiles_profile_example_with_records() {
    let source = include_str!("../examples/profile.nui");
    let doc = compile(source).expect("profile example should compile");
    let component = &doc.component;

    assert_eq!(component.types.len(), 1);
    assert_eq!(component.types[0].name, "Person");
    assert_eq!(component.types[0].fields.len(), 2);
    assert_eq!(component.types[0].fields[0].name, "name");
    assert_eq!(component.types[0].fields[0].ty, ir::Type::String);

    assert_eq!(component.state[0].name, "person");
    assert_eq!(component.state[0].ty, ir::Type::Record("Person".into()));
    let ir::Value::Record(fields) = &component.state[0].initial else {
        panic!("expected a record initial value");
    };
    assert_eq!(fields[0].name, "name");
    assert_eq!(fields[0].value, ir::Value::String("Ada Lovelace".into()));

    assert_eq!(
        component.functions[0].params[0].ty,
        ir::Type::Record("Person".into())
    );
    assert_eq!(
        component.functions[0].returns,
        ir::Type::Record("Person".into())
    );

    // Interpolation resolved the dotted path, and the action passes the
    // whole record state.
    let ir::Node::VStack { children, .. } = &component.root else {
        panic!("expected VStack root");
    };
    let ir::Node::Text { content, .. } = &children[0] else {
        panic!("expected Text, got {:?}", children[0]);
    };
    assert_eq!(
        content.0,
        [ir::TextSegment::State {
            name: "person.name".into()
        }]
    );
    let ir::Node::Button { action, .. } = &children[2] else {
        panic!("expected Button, got {:?}", children[2]);
    };
    assert_eq!(
        action.args,
        vec![ir::CallArg::State {
            name: "person".into()
        }]
    );

    // Records survive the JSON round-trip like everything else.
    let json = serde_json::to_string(&doc).unwrap();
    let back: ir::Document = serde_json::from_str(&json).unwrap();
    assert_eq!(doc, back);
}

#[test]
fn record_literal_fields_land_in_declaration_order() {
    let source = r#"
        type Point { x: Int  y: Int }
        component X {
            state p: Point = Point(y: 2, x: 1)
            Text { text: "{p.x}" }
        }
    "#;
    let doc = compile(source).unwrap();
    let ir::Value::Record(fields) = &doc.component.state[0].initial else {
        panic!("expected a record value");
    };
    assert_eq!(fields[0].name, "x");
    assert_eq!(fields[0].value, ir::Value::Int(1));
    assert_eq!(fields[1].name, "y");
    assert_eq!(fields[1].value, ir::Value::Int(2));
}

#[test]
fn rejects_unknown_field_in_record_literal() {
    let source = r#"
        type Person { name: String }
        component X {
            state p: Person = Person(name: "A", age: 3)
            Spacer
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("no field `age`"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_missing_field_in_record_literal() {
    let source = r#"
        type Person { name: String  bio: String }
        component X {
            state p: Person = Person(name: "A")
            Spacer
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("missing field `bio`"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_record_typed_fields() {
    let source = r#"
        type Inner { n: Int }
        type Outer { inner: Inner }
        component X { Spacer }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("must be primitives"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_interpolating_a_whole_record() {
    let source = r#"
        type Person { name: String }
        component X {
            state p: Person = Person(name: "A")
            Text { text: "{p}" }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("pick a field") && err.message.contains("p.name"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_unknown_field_path() {
    let source = r#"
        type Person { name: String }
        component X {
            state p: Person = Person(name: "A")
            Text { text: "{p.age}" }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("no field `age`"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_assigning_to_a_record_field() {
    let source = r#"
        type Person { name: String }
        component X {
            state p: Person = Person(name: "A")
            logic { fn rename(name: String) -> String }
            Button { label: "go", on_click: { p.name = rename(p.name) } }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("whole state"),
        "got: {}",
        err.message
    );
}

#[test]
fn record_fields_work_as_call_arguments() {
    let source = r#"
        type Person { name: String }
        component X {
            state p: Person = Person(name: "A")
            state greeting: String = ""
            logic { fn greet(name: String) -> String }
            Button { label: "go", on_click: { greeting = greet(p.name) } }
        }
    "#;
    let doc = compile(source).unwrap();
    let ir::Node::Button { action, .. } = &doc.component.root else {
        panic!("expected Button root");
    };
    assert_eq!(
        action.args,
        vec![ir::CallArg::State {
            name: "p.name".into()
        }]
    );
}

#[test]
fn bool_record_fields_drive_if_conditions() {
    let source = r#"
        type Flags { ready: Bool }
        component X {
            state flags: Flags = Flags(ready: false)
            VStack {
                if flags.ready { Spacer }
            }
        }
    "#;
    let doc = compile(source).unwrap();
    let ir::Node::VStack { children, .. } = &doc.component.root else {
        panic!("expected VStack root");
    };
    let ir::Node::If { condition, .. } = &children[0] else {
        panic!("expected an if node");
    };
    assert_eq!(condition, "flags.ready");
}

// --- lists and `for` ---

const TODOS: &str = include_str!("../examples/todos.nui");

#[test]
fn compiles_todos_example_with_lists() {
    let doc = compile(TODOS).expect("todos example should compile");
    let component = &doc.component;
    assert_eq!(component.name, "TodoList");

    let todo_list = ir::Type::List(Box::new(ir::Type::Record("Todo".into())));
    assert_eq!(component.state[0].name, "todos");
    assert_eq!(component.state[0].ty, todo_list);
    assert_eq!(
        component.state[0].initial,
        ir::Value::List(vec![ir::Value::Record(vec![ir::FieldValue {
            name: "title".into(),
            value: ir::Value::String("Learn nui".into()),
        }])])
    );
    assert_eq!(component.functions[0].params[0].ty, todo_list);
    assert_eq!(component.functions[0].returns, todo_list);

    let ir::Node::VStack { children, .. } = &component.root else {
        panic!("expected VStack root");
    };
    let ir::Node::List { children, .. } = &children[2] else {
        panic!("expected a List, got {:?}", children[2]);
    };
    let ir::Node::For {
        binding,
        source,
        children,
    } = &children[0]
    else {
        panic!("expected a for node, got {:?}", children[0]);
    };
    assert_eq!(binding, "todo");
    assert_eq!(source, "todos");
    let ir::Node::Text { content, .. } = &children[0] else {
        panic!("expected Text in the for body");
    };
    assert_eq!(
        content.0,
        [ir::TextSegment::Local {
            name: "todo.title".into()
        }]
    );

    // Empty and seeded list values survive the JSON round trip.
    let json = serde_json::to_string(&doc).unwrap();
    let back: ir::Document = serde_json::from_str(&json).unwrap();
    assert_eq!(doc, back);
}

#[test]
fn empty_list_initializers_round_trip() {
    let source = r#"
        component X {
            state names: [String] = []
            VStack {
                for name in names { Text { text: "{name}" } }
            }
        }
    "#;
    let doc = compile(source).unwrap();
    assert_eq!(doc.component.state[0].initial, ir::Value::List(vec![]));
    let json = serde_json::to_string(&doc).unwrap();
    let back: ir::Document = serde_json::from_str(&json).unwrap();
    assert_eq!(doc, back);
}

#[test]
fn rejects_for_over_a_non_list() {
    let source = r#"
        component X {
            state count: Int = 0
            VStack {
                for item in count { Spacer }
            }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("needs a list") && err.message.contains("Int"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_if_inside_a_for_body() {
    let source = r#"
        component X {
            state names: [String] = []
            state on: Bool = true
            VStack {
                for name in names {
                    if on { Spacer }
                }
            }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("`if` inside a `for`"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_nested_for_loops() {
    let source = r#"
        component X {
            state names: [String] = []
            VStack {
                for a in names {
                    for b in names { Spacer }
                }
            }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("`for` inside another `for`"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_textfield_inside_a_for_body() {
    let source = r#"
        component X {
            state names: [String] = []
            state draft: String = ""
            VStack {
                for name in names {
                    TextField { bind: draft }
                }
            }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("`TextField` inside a `for`"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_loop_variables_as_call_arguments() {
    let source = r#"
        component X {
            state names: [String] = []
            logic { fn remove(names: [String], name: String) -> [String] }
            VStack {
                for name in names {
                    Button { label: "x", on_click: { names = remove(names, name) } }
                }
            }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("loop variable"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_loop_variables_that_shadow_states() {
    let source = r#"
        component X {
            state name: String = ""
            state names: [String] = []
            VStack {
                for name in names { Spacer }
            }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(err.message.contains("shadows"), "got: {}", err.message);
}

#[test]
fn rejects_displaying_a_whole_list() {
    let source = r#"
        component X {
            state names: [String] = []
            Text { text: "{names}" }
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("whole list"),
        "got: {}",
        err.message
    );
}

#[test]
fn list_initializer_elements_are_type_checked() {
    let source = r#"
        component X {
            state counts: [Int] = [1, "two"]
            Spacer
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(err.message.contains("Int"), "got: {}", err.message);
}

#[test]
fn rejects_list_typed_record_fields() {
    let source = r#"
        type Todo { tags: [String] }
        component X {
            state todo: Todo = Todo(tags: [])
            Spacer
        }
    "#;
    let err = compile(source).unwrap_err();
    assert!(
        err.message.contains("primitives"),
        "got: {}",
        err.message
    );
}

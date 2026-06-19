//! SwiftUI emitter. AST -> Swift source string.

use crate::ast::*;
use std::collections::HashMap;

pub fn emit(c: &Component) -> String {
    let name = &c.name;
    let binds = collect_bind_events(&c.root);
    let mut out = String::new();

    out.push_str(&format!(
        "// GENERATED from {}.nui by nuic — do not edit.\n",
        name.to_lowercase()
    ));
    out.push_str("import SwiftUI\n\n");

    out.push_str(&format!("struct {name}State: Decodable {{\n"));
    for f in &c.state {
        let ty = map_type(&f.ty);
        match &f.default {
            Some(v) => out.push_str(&format!(
                "    var {}: {} = {}\n",
                f.name,
                ty,
                default_val(v, &f.ty)
            )),
            None => out.push_str(&format!("    var {}: {}\n", f.name, ty)),
        }
    }
    out.push_str("}\n\n");

    out.push_str("@MainActor\n");
    out.push_str(&format!("final class {name}Model: ObservableObject {{\n"));
    out.push_str(&format!("    @Published private(set) var state = {name}State()\n"));
    out.push_str("    private let bridge: Bridge\n\n");
    out.push_str("    init(bridge: Bridge) {\n");
    out.push_str("        self.bridge = bridge\n");
    out.push_str(&format!(
        "        bridge.onState {{ [weak self] (new: {name}State) in self?.state = new }}\n"
    ));
    out.push_str("    }\n\n");
    for e in &c.events {
        out.push_str(&emit_event_method(e, &binds, &c.state));
    }
    out.push_str("}\n\n");

    out.push_str(&format!("struct {name}View: View {{\n"));
    out.push_str(&format!("    @StateObject private var model: {name}Model\n"));
    out.push_str(&format!(
        "    init(bridge: Bridge) {{ _model = StateObject(wrappedValue: {name}Model(bridge: bridge)) }}\n\n"
    ));
    out.push_str("    var body: some View {\n");
    out.push_str(&emit_element(&c.root, 2, &c.state));
    out.push_str("\n    }\n}\n");

    out
}

fn emit_event_method(event: &str, binds: &HashMap<String, String>, state: &[Field]) -> String {
    if let Some(field) = binds.get(event) {
        let ty = state
            .iter()
            .find(|f| f.name == *field)
            .map(|f| f.ty.as_str())
            .unwrap_or("String");
        let swift_ty = map_type(ty);
        let cast = payload_cast(ty);
        format!(
            "    func {event}(_ value: {swift_ty}) {{ bridge.send(event: \"{event}\", payload: [\"value\": {cast}]) }}\n\n"
        )
    } else {
        format!("    func {event}() {{ bridge.send(event: \"{event}\") }}\n\n")
    }
}

fn payload_cast(ty: &str) -> &'static str {
    match ty {
        "Int" => "value",
        "Bool" => "value",
        _ => "value",
    }
}

fn emit_element(el: &Element, lvl: usize, state: &[Field]) -> String {
    match el.name.as_str() {
        "Column" => emit_stack(el, lvl, state, "VStack"),
        "Row" => emit_stack(el, lvl, state, "HStack"),
        "Scroll" => emit_scroll(el, lvl, state),
        "Card" => emit_card(el, lvl, state),
        "Spacer" => format!("{}Spacer()", pad(lvl)),
        "Divider" => format!("{}Divider()", pad(lvl)),
        "If" => emit_if(el, lvl, state),
        "Text" => emit_text(el, lvl, state),
        "Icon" => emit_icon(el, lvl),
        "IconButton" => emit_icon_button(el, lvl),
        "Button" => emit_button(el, lvl),
        "Switch" => emit_switch(el, lvl),
        "Slider" => emit_slider(el, lvl),
        "TextField" => emit_text_field(el, lvl),
        "Progress" => emit_progress(el, lvl),
        other => format!("{}// TODO: unsupported element `{other}`", pad(lvl)),
    }
}

fn emit_scroll(el: &Element, lvl: usize, state: &[Field]) -> String {
    let ind = pad(lvl);
    let inner = el
        .children
        .iter()
        .map(|ch| emit_element(ch, lvl + 1, state))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{ind}ScrollView {{\n{inner}\n{ind}}}")
}

fn emit_card(el: &Element, lvl: usize, state: &[Field]) -> String {
    let ind = pad(lvl);
    let inner = el
        .children
        .iter()
        .map(|ch| emit_element(ch, lvl + 1, state))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{ind}Group {{\n{inner}\n{ind}}}\n{ind}.padding()\n{ind}.background(.regularMaterial, in: RoundedRectangle(cornerRadius: 12))\n{ind}.frame(maxWidth: .infinity)"
    )
}

fn emit_if(el: &Element, lvl: usize, state: &[Field]) -> String {
    let ind = pad(lvl);
    let field = bind_field(el);
    let inner = el
        .children
        .iter()
        .map(|ch| emit_element(ch, lvl + 1, state))
        .collect::<Vec<_>>()
        .join("\n");
    if el.else_children.is_empty() {
        return format!("{ind}if model.state.{field} {{\n{inner}\n{ind}}}");
    }
    let else_inner = el
        .else_children
        .iter()
        .map(|ch| emit_element(ch, lvl + 1, state))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{ind}if model.state.{field} {{\n{inner}\n{ind}}} else {{\n{else_inner}\n{ind}}}")
}

fn emit_icon(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let icon = arg(&el.args, "icon").map(val).unwrap_or_default();
    let sf = map_icon(&icon);
    let mut s = format!("{ind}Image(systemName: \"{sf}\")");
    if let Some(t) = arg(&el.args, "tint") {
        s.push_str(&format!("\n{ind}.foregroundStyle(.{})", val(t)));
    }
    if let Some(sz) = arg(&el.args, "size") {
        s.push_str(&format!("\n{ind}.font(.system(size: {}))", val(sz)));
    }
    s
}

fn emit_progress(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let field = bind_field(el);
    let max = arg(&el.args, "max").map(val).unwrap_or_else(|| "1".into());
    format!(
        "{ind}ProgressView(\n\
         {ind}    value: Double(min(model.state.{field}, {max})),\n\
         {ind}    total: {max}\n\
         {ind})\n\
         {ind}.frame(maxWidth: .infinity)"
    )
}

fn emit_switch(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let label = arg(&el.args, "label").map(val).unwrap_or_default();
    let field = bind_field(el);
    let ev = el.event.clone().unwrap_or_default();
    format!(
        "{ind}Toggle(\"{label}\", isOn: Binding(\n\
         {ind}    get: {{ model.state.{field} }},\n\
         {ind}    set: {{ model.{ev}($0) }}\n\
         {ind}))"
    )
}

fn emit_slider(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let label = arg(&el.args, "label").map(val).unwrap_or_default();
    let field = bind_field(el);
    let ev = el.event.clone().unwrap_or_default();
    let min = arg(&el.args, "min").map(val).unwrap_or_else(|| "0".into());
    let max = arg(&el.args, "max").map(val).unwrap_or_else(|| "10".into());
    format!(
        "{ind}VStack(alignment: .leading, spacing: 4) {{\n\
         {ind}    Text(\"{label}\")\n\
         {ind}        .font(.subheadline)\n\
         {ind}        .foregroundStyle(.secondary)\n\
         {ind}    Slider(\n\
         {ind}        value: Binding(\n\
         {ind}            get: {{ Double(model.state.{field}) }},\n\
         {ind}            set: {{ model.{ev}(Int($0)) }}\n\
         {ind}        ),\n\
         {ind}        in: {min}...{max},\n\
         {ind}        step: 1\n\
         {ind}    )\n\
         {ind}}}\n\
         {ind}.frame(maxWidth: .infinity)"
    )
}

fn emit_text_field(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let label = arg(&el.args, "label").map(val).unwrap_or_default();
    let field = bind_field(el);
    let ev = el.event.clone().unwrap_or_default();
    format!(
        "{ind}VStack(alignment: .leading, spacing: 4) {{\n\
         {ind}    Text(\"{label}\")\n\
         {ind}        .font(.subheadline)\n\
         {ind}        .foregroundStyle(.secondary)\n\
         {ind}    TextField(\"{label}\", text: Binding(\n\
         {ind}        get: {{ model.state.{field} }},\n\
         {ind}        set: {{ model.{ev}($0) }}\n\
         {ind}    ))\n\
         {ind}    .textFieldStyle(.roundedBorder)\n\
         {ind}}}\n\
         {ind}.frame(maxWidth: .infinity)"
    )
}

fn emit_stack(el: &Element, lvl: usize, state: &[Field], kind: &str) -> String {
    let ind = pad(lvl);
    let header = match arg(&el.args, "spacing") {
        Some(s) => format!("{kind}(spacing: {})", val(s)),
        None => kind.to_string(),
    };

    let mut s = format!("{ind}{header} {{\n");
    let children: Vec<String> = el
        .children
        .iter()
        .map(|ch| emit_element(ch, lvl + 1, state))
        .collect();
    s.push_str(&children.join("\n"));
    s.push_str(&format!("\n{ind}}}"));

    if let Some(p) = arg(&el.args, "padding") {
        s.push_str(&format!("\n{ind}.padding({})", val(p)));
    }
    if arg(&el.args, "align").map(val).as_deref() == Some("center") {
        s.push_str(&format!("\n{ind}.frame(maxWidth: .infinity)"));
    }
    s
}

fn emit_text(el: &Element, lvl: usize, state: &[Field]) -> String {
    let ind = pad(lvl);
    let raw = positional(&el.args).map(val).unwrap_or_default();
    let interpolated = rewrite_interpolation(&raw, state);
    let mut s = format!("{ind}Text(\"{interpolated}\")");
    for m in &el.modifiers {
        s.push_str(&format!("\n{ind}{}", emit_modifier(m)));
    }
    s
}

fn emit_icon_button(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let icon = arg(&el.args, "icon").map(val).unwrap_or_default();
    let sf = map_icon(&icon);
    let ev = el.event.clone().unwrap_or_default();
    let mut s = format!(
        "{ind}Button(action: model.{ev}) {{\n{ind}    Image(systemName: \"{sf}\")\n{ind}}}"
    );
    if let Some(t) = arg(&el.args, "tint") {
        s.push_str(&format!("\n{ind}.tint(.{})", val(t)));
    }
    s
}

fn emit_button(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let label = positional(&el.args).map(val).unwrap_or_default();
    let ev = el.event.clone().unwrap_or_default();
    let mut s = format!("{ind}Button(\"{label}\", action: model.{ev})");
    for m in &el.modifiers {
        s.push_str(&format!("\n{ind}{}", emit_modifier(m)));
    }
    s
}

fn emit_modifier(m: &Modifier) -> String {
    match m.name.as_str() {
        "font" => {
            let size = arg(&m.args, "size").map(val);
            let weight = arg(&m.args, "weight").map(val);
            match (size, weight) {
                (Some(s), Some(w)) => format!(".font(.system(size: {s}, weight: .{w}))"),
                (Some(s), None) => format!(".font(.system(size: {s}))"),
                _ => ".font(.body)".to_string(),
            }
        }
        "color" => {
            let c = positional(&m.args).map(val).unwrap_or_else(|| "primary".into());
            format!(".foregroundStyle(.{c})")
        }
        "style" => {
            let st = positional(&m.args).map(val).unwrap_or_default();
            format!(".buttonStyle(.{st})")
        }
        "padding" => {
            let p = positional(&m.args).map(val).unwrap_or_default();
            format!(".padding({p})")
        }
        other => format!("// TODO: unsupported modifier `{other}`"),
    }
}

fn collect_bind_events(root: &Element) -> HashMap<String, String> {
    let mut map = HashMap::new();
    walk_bind_events(root, &mut map);
    map
}

fn walk_bind_events(el: &Element, map: &mut HashMap<String, String>) {
    if matches!(el.name.as_str(), "Switch" | "Slider" | "TextField") {
        if let Some(ev) = &el.event {
            map.insert(ev.clone(), bind_field(el));
        }
    }
    for ch in &el.children {
        walk_bind_events(ch, map);
    }
}

fn bind_field(el: &Element) -> String {
    arg(&el.args, "bind")
        .map(val)
        .or_else(|| positional(&el.args).map(val))
        .unwrap_or_default()
}

fn pad(lvl: usize) -> String {
    "    ".repeat(lvl)
}

fn val(v: &Value) -> String {
    match v {
        Value::Str(s) => s.clone(),
        Value::Num(n) => n.clone(),
        Value::Ident(i) => i.clone(),
    }
}

fn default_val(v: &Value, ty: &str) -> String {
    match (v, ty) {
        (Value::Ident(s), "Bool") if s == "true" => "true".into(),
        (Value::Ident(s), "Bool") if s == "false" => "false".into(),
        (Value::Str(s), "String") => format!("\"{s}\""),
        _ => val(v),
    }
}

fn arg<'a>(args: &'a [Arg], name: &str) -> Option<&'a Value> {
    args.iter()
        .find(|a| a.name.as_deref() == Some(name))
        .map(|a| &a.value)
}

fn positional(args: &[Arg]) -> Option<&Value> {
    args.iter().find(|a| a.name.is_none()).map(|a| &a.value)
}

fn rewrite_interpolation(s: &str, state: &[Field]) -> String {
    let mut out = s.to_string();
    for f in state {
        let from = format!("\\({})", f.name);
        let to = format!("\\(model.state.{})", f.name);
        out = out.replace(&from, &to);
    }
    out
}

fn map_type(ty: &str) -> &str {
    match ty {
        "Int" => "Int",
        "Double" => "Double",
        "Bool" => "Bool",
        "String" => "String",
        other => other,
    }
}

fn map_icon(icon: &str) -> String {
    match icon {
        "plus" => "plus",
        "minus" => "minus",
        "check" => "checkmark",
        "close" => "xmark",
        "refresh" => "arrow.clockwise",
        other => other,
    }
    .to_string()
}

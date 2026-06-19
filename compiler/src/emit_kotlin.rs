//! Jetpack Compose emitter. AST -> Kotlin source string.

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
    out.push_str("package dev.nui.generated\n\n");
    out.push_str(
        "import androidx.compose.foundation.layout.*\n\
         import androidx.compose.foundation.rememberScrollState\n\
         import androidx.compose.foundation.text.KeyboardOptions\n\
         import androidx.compose.foundation.verticalScroll\n\
         import androidx.compose.material.icons.Icons\n\
         import androidx.compose.material.icons.filled.Add\n\
         import androidx.compose.material.icons.filled.Check\n\
         import androidx.compose.material.icons.filled.Close\n\
         import androidx.compose.material.icons.filled.Refresh\n\
         import androidx.compose.material.icons.filled.Remove\n\
         import androidx.compose.material3.*\n\
         import androidx.compose.runtime.*\n\
         import androidx.compose.ui.Alignment\n\
         import androidx.compose.ui.Modifier\n\
         import androidx.compose.ui.graphics.Color\n\
         import androidx.compose.ui.text.font.FontWeight\n\
         import androidx.compose.ui.text.input.KeyboardCapitalization\n\
         import androidx.compose.ui.unit.dp\n\
         import androidx.compose.ui.unit.sp\n\
         import dev.nui.runtime.Bridge\n\
         import kotlinx.serialization.Serializable\n\n",
    );

    out.push_str("@Serializable\n");
    out.push_str(&format!("data class {name}State("));
    let fields: Vec<String> = c
        .state
        .iter()
        .map(|f| {
            let ty = map_type(&f.ty);
            match &f.default {
                Some(v) => format!("val {}: {} = {}", f.name, ty, default_val(v, &f.ty)),
                None => format!("val {}: {}", f.name, ty),
            }
        })
        .collect();
    out.push_str(&fields.join(", "));
    out.push_str(")\n\n");

    out.push_str(&format!("class {name}Model(private val bridge: Bridge) {{\n"));
    out.push_str(&format!("    var state by mutableStateOf({name}State())\n"));
    out.push_str("        private set\n\n");
    out.push_str(&format!(
        "    init {{ bridge.onState<{name}State> {{ new -> state = new }} }}\n\n"
    ));
    for e in &c.events {
        out.push_str(&emit_event_method(e, &binds, &c.state));
    }
    out.push_str("}\n\n");

    out.push_str("@Composable\n");
    out.push_str(&format!("fun {name}View(bridge: Bridge) {{\n"));
    out.push_str(&format!("    val model = remember {{ {name}Model(bridge) }}\n"));
    out.push_str(&emit_element(&c.root, 1, &c.state));
    out.push_str("\n}\n");

    out
}

fn emit_event_method(event: &str, binds: &HashMap<String, String>, state: &[Field]) -> String {
    if let Some(field) = binds.get(event) {
        let ty = state
            .iter()
            .find(|f| f.name == *field)
            .map(|f| f.ty.as_str())
            .unwrap_or("String");
        let kotlin_ty = map_type(ty);
        let arg_expr = match ty {
            "Int" => "value",
            "Bool" => "value",
            _ => "value",
        };
        format!(
            "    fun {event}(value: {kotlin_ty}) = bridge.send(\"{event}\", mapOf(\"value\" to {arg_expr}))\n\n"
        )
    } else {
        format!("    fun {event}() = bridge.send(\"{event}\")\n\n")
    }
}

fn emit_element(el: &Element, lvl: usize, state: &[Field]) -> String {
    match el.name.as_str() {
        "Column" => emit_stack(el, lvl, state, Axis::Vertical),
        "Row" => emit_stack(el, lvl, state, Axis::Horizontal),
        "Scroll" => emit_scroll(el, lvl, state),
        "Card" => emit_card(el, lvl, state),
        "Spacer" => format!("{}Spacer(Modifier.weight(1f))", pad(lvl)),
        "Divider" => format!("{}HorizontalDivider()", pad(lvl)),
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
    format!(
        "{ind}Column(\n\
         {ind}    modifier = Modifier\n\
         {ind}        .fillMaxSize()\n\
         {ind}        .verticalScroll(rememberScrollState())\n\
         {ind}) {{\n{inner}\n{ind}}}"
    )
}

fn emit_card(el: &Element, lvl: usize, state: &[Field]) -> String {
    let ind = pad(lvl);
    let inner = pad(lvl + 1);
    let body = el
        .children
        .iter()
        .map(|ch| emit_element(ch, lvl + 2, state))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{ind}Card(\n\
         {inner}modifier = Modifier.fillMaxWidth()\n\
         {ind}) {{\n{body}\n{ind}}}"
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
        return format!("{ind}if (model.state.{field}) {{\n{inner}\n{ind}}}");
    }
    let else_inner = el
        .else_children
        .iter()
        .map(|ch| emit_element(ch, lvl + 1, state))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{ind}if (model.state.{field}) {{\n{inner}\n{ind}}} else {{\n{else_inner}\n{ind}}}"
    )
}

fn emit_icon(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let inner = pad(lvl + 1);
    let icon = arg(&el.args, "icon").map(val).unwrap_or_default();
    let (material_icon, desc) = map_icon(&icon);
    let mut params = vec![format!("imageVector = {material_icon}")];
    params.push(format!("contentDescription = \"{desc}\""));
    if let Some(t) = arg(&el.args, "tint") {
        params.push(format!("tint = {}", map_tint(&val(t))));
    }
    let mut modifier = String::new();
    if let Some(sz) = arg(&el.args, "size") {
        modifier = format!(".size({}.dp)", val(sz));
    }
    if !modifier.is_empty() {
        params.push(format!("modifier = Modifier{modifier}"));
    }
    format!(
        "{ind}Icon(\n{}\n{ind})",
        params
            .iter()
            .map(|p| format!("{inner}{p}"))
            .collect::<Vec<_>>()
            .join(",\n")
    )
}

fn emit_progress(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let inner = pad(lvl + 1);
    let field = bind_field(el);
    let max = arg(&el.args, "max").map(val).unwrap_or_else(|| "1".into());
    format!(
        "{ind}LinearProgressIndicator(\n\
         {inner}progress = {{ (model.state.{field}.toFloat() / {max}f).coerceIn(0f, 1f) }},\n\
         {inner}modifier = Modifier.fillMaxWidth()\n\
         {ind})"
    )
}

fn emit_switch(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let inner = pad(lvl + 1);
    let label = arg(&el.args, "label").map(val).unwrap_or_default();
    let field = bind_field(el);
    let ev = el.event.clone().unwrap_or_default();
    format!(
        "{ind}Row(\n\
         {inner}modifier = Modifier.fillMaxWidth(),\n\
         {inner}horizontalArrangement = Arrangement.SpaceBetween,\n\
         {inner}verticalAlignment = Alignment.CenterVertically\n\
         {ind}) {{\n\
         {inner}Text(\"{label}\")\n\
         {inner}Switch(\n\
         {inner}    checked = model.state.{field},\n\
         {inner}    onCheckedChange = model::{ev}\n\
         {inner})\n\
         {ind}}}"
    )
}

fn emit_slider(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let inner = pad(lvl + 1);
    let label = arg(&el.args, "label").map(val).unwrap_or_default();
    let field = bind_field(el);
    let ev = el.event.clone().unwrap_or_default();
    let min = arg(&el.args, "min").map(val).unwrap_or_else(|| "0".into());
    let max = arg(&el.args, "max").map(val).unwrap_or_else(|| "10".into());
    format!(
        "{ind}Column(\n\
         {inner}modifier = Modifier.fillMaxWidth()\n\
         {ind}) {{\n\
         {inner}Text(\"{label}\", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)\n\
         {inner}Slider(\n\
         {inner}    value = model.state.{field}.toFloat(),\n\
         {inner}    onValueChange = {{ model.{ev}(it.toInt()) }},\n\
         {inner}    valueRange = {min}f..{max}f,\n\
         {inner}    steps = ({max} - {min} - 1).coerceAtLeast(0)\n\
         {inner})\n\
         {ind}}}"
    )
}

fn emit_text_field(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let inner = pad(lvl + 1);
    let label = arg(&el.args, "label").map(val).unwrap_or_default();
    let field = bind_field(el);
    let ev = el.event.clone().unwrap_or_default();
    format!(
        "{ind}OutlinedTextField(\n\
         {inner}value = model.state.{field},\n\
         {inner}onValueChange = model::{ev},\n\
         {inner}label = {{ Text(\"{label}\") }},\n\
         {inner}modifier = Modifier.fillMaxWidth(),\n\
         {inner}singleLine = true,\n\
         {inner}keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.Words)\n\
         {ind})"
    )
}

enum Axis {
    Vertical,
    Horizontal,
}

fn emit_stack(el: &Element, lvl: usize, state: &[Field], axis: Axis) -> String {
    let ind = pad(lvl);
    let inner = pad(lvl + 1);
    let (kind, arrangement_key, align_key, align_val) = match axis {
        Axis::Vertical => (
            "Column",
            "verticalArrangement",
            "horizontalAlignment",
            "Alignment.CenterHorizontally",
        ),
        Axis::Horizontal => (
            "Row",
            "horizontalArrangement",
            "verticalAlignment",
            "Alignment.CenterVertically",
        ),
    };

    let mut params: Vec<String> = Vec::new();
    let mut modifier = String::new();
    if let Some(p) = arg(&el.args, "padding") {
        modifier.push_str(&format!(".padding({}.dp)", val(p)));
    }
    let centered = arg(&el.args, "align").map(val).as_deref() == Some("center");
    if centered {
        modifier.push_str(".fillMaxWidth()");
    }
    if !modifier.is_empty() {
        params.push(format!("modifier = Modifier{modifier}"));
    }
    if let Some(s) = arg(&el.args, "spacing") {
        params.push(format!(
            "{arrangement_key} = Arrangement.spacedBy({}.dp)",
            val(s)
        ));
    }
    if centered {
        params.push(format!("{align_key} = {align_val}"));
    }

    let mut out = String::new();
    if params.is_empty() {
        out.push_str(&format!("{ind}{kind} {{\n"));
    } else {
        out.push_str(&format!("{ind}{kind}(\n"));
        out.push_str(
            &params
                .iter()
                .map(|p| format!("{inner}{p}"))
                .collect::<Vec<_>>()
                .join(",\n"),
        );
        out.push_str(&format!("\n{ind}) {{\n"));
    }

    let children: Vec<String> = el
        .children
        .iter()
        .map(|ch| emit_element(ch, lvl + 1, state))
        .collect();
    out.push_str(&children.join("\n"));
    out.push_str(&format!("\n{ind}}}"));
    out
}

fn emit_text(el: &Element, lvl: usize, state: &[Field]) -> String {
    let ind = pad(lvl);
    let inner = pad(lvl + 1);
    let raw = positional(&el.args).map(val).unwrap_or_default();
    let text = rewrite_interpolation(&raw, state);

    let mut params = vec![format!("text = \"{text}\"")];
    let mut modifier = String::new();
    for m in &el.modifiers {
        match m.name.as_str() {
            "font" => {
                if let Some(s) = arg(&m.args, "size") {
                    params.push(format!("fontSize = {}.sp", val(s)));
                }
                if let Some(w) = arg(&m.args, "weight") {
                    params.push(format!("fontWeight = FontWeight.{}", cap(&val(w))));
                }
            }
            "color" => {
                let c = positional(&m.args).map(val).unwrap_or_else(|| "primary".into());
                params.push(format!("color = {}", map_color_semantic(&c)));
            }
            "padding" => {
                if let Some(p) = positional(&m.args) {
                    modifier.push_str(&format!(".padding({}.dp)", val(p)));
                }
            }
            _ => {}
        }
    }
    if !modifier.is_empty() {
        params.push(format!("modifier = Modifier{modifier}"));
    }

    let mut out = format!("{ind}Text(\n");
    out.push_str(
        &params
            .iter()
            .map(|p| format!("{inner}{p}"))
            .collect::<Vec<_>>()
            .join(",\n"),
    );
    out.push_str(&format!("\n{ind})"));
    out
}

fn emit_icon_button(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let inner = pad(lvl + 1);
    let icon = arg(&el.args, "icon").map(val).unwrap_or_default();
    let ev = el.event.clone().unwrap_or_default();
    let (material_icon, desc) = map_icon(&icon);

    let mut params = vec![format!("onClick = model::{ev}")];
    if let Some(t) = arg(&el.args, "tint") {
        params.push(format!(
            "colors = IconButtonDefaults.filledIconButtonColors(containerColor = {})",
            map_color_value(&val(t))
        ));
    }

    let mut out = format!("{ind}FilledIconButton(\n");
    out.push_str(
        &params
            .iter()
            .map(|p| format!("{inner}{p}"))
            .collect::<Vec<_>>()
            .join(",\n"),
    );
    out.push_str(&format!(
        "\n{ind}) {{ Icon({material_icon}, contentDescription = \"{desc}\") }}"
    ));
    out
}

fn emit_button(el: &Element, lvl: usize) -> String {
    let ind = pad(lvl);
    let label = positional(&el.args).map(val).unwrap_or_default();
    let ev = el.event.clone().unwrap_or_default();
    let bordered = el
        .modifiers
        .iter()
        .any(|m| m.name == "style" && positional(&m.args).map(val).as_deref() == Some("bordered"));
    let widget = if bordered { "OutlinedButton" } else { "Button" };
    format!("{ind}{widget}(onClick = model::{ev}) {{ Text(\"{label}\") }}")
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

fn cap(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
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
        let to = format!("${{model.state.{}}}", f.name);
        out = out.replace(&from, &to);
    }
    out
}

fn map_type(ty: &str) -> &str {
    match ty {
        "Int" => "Int",
        "Double" => "Double",
        "Bool" => "Boolean",
        "String" => "String",
        other => other,
    }
}

fn map_icon(icon: &str) -> (String, String) {
    match icon {
        "plus" => ("Icons.Filled.Add".into(), "plus".into()),
        "minus" => ("Icons.Filled.Remove".into(), "minus".into()),
        "check" => ("Icons.Filled.Check".into(), "check".into()),
        "close" => ("Icons.Filled.Close".into(), "close".into()),
        "refresh" => ("Icons.Filled.Refresh".into(), "refresh".into()),
        other => ("Icons.Filled.Add".into(), other.into()),
    }
}

fn map_color_semantic(c: &str) -> String {
    match c {
        "secondary" => "MaterialTheme.colorScheme.onSurfaceVariant".into(),
        "primary" => "MaterialTheme.colorScheme.onSurface".into(),
        other => format!("Color.{}", cap(other)),
    }
}

fn map_color_value(c: &str) -> String {
    format!("Color.{}", cap(c))
}

fn map_tint(c: &str) -> String {
    match c {
        "primary" | "secondary" => map_color_semantic(c),
        other => map_color_value(other),
    }
}

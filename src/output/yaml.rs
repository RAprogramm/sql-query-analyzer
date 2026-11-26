//! Minimal, dependency-free YAML serializer for analysis output.
//!
//! Serializes any [`serde::Serialize`] value to YAML by routing through
//! [`serde_json::Value`]. It intentionally avoids the unmaintained
//! `libyaml`-backed crates while covering the data shapes the analyzer emits:
//! maps, sequences, strings, numbers, booleans, and null.

use std::fmt::Write;

use serde::Serialize;
use serde_json::Value;

/// Serialize a value to a YAML document string.
///
/// # Errors
///
/// Returns the serialization error when the value cannot be represented as
/// JSON, which is the intermediate representation used for emission.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let json = serde_json::to_value(value)?;
    let mut out = String::new();
    match &json {
        Value::Object(map) => emit_object(map, 0, true, &mut out),
        Value::Array(items) => emit_array(items, 0, true, &mut out),
        _ => emit_scalar(&json, &mut out),
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

/// Emit a scalar JSON value on the current line.
fn emit_scalar(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null\n"),
        Value::Bool(b) => {
            let _ = writeln!(out, "{b}");
        }
        Value::Number(n) => {
            let _ = writeln!(out, "{n}");
        }
        Value::String(s) => {
            out.push_str(&format_scalar(s));
            out.push('\n');
        }
        Value::Array(items) if items.is_empty() => out.push_str("[]\n"),
        Value::Object(map) if map.is_empty() => out.push_str("{}\n"),
        _ => out.push('\n'),
    }
}

/// Emit a JSON array as a YAML sequence.
///
/// When `first_on_current_line` is set, the cursor already sits where the first
/// `- ` marker belongs (e.g. right after a parent `- `), so no leading newline
/// or indent is written for the first item.
fn emit_array(items: &[Value], indent: usize, first_on_current_line: bool, out: &mut String) {
    if items.is_empty() {
        out.push_str("[]\n");
        return;
    }
    for (i, item) in items.iter().enumerate() {
        if i > 0 || !first_on_current_line {
            push_indent(out, indent);
        }
        out.push_str("- ");
        match item {
            Value::Object(map) => emit_object(map, indent + 2, true, out),
            Value::Array(inner) => emit_array(inner, indent + 2, true, out),
            _ => emit_scalar(item, out),
        }
    }
}

/// Emit a JSON object as a YAML mapping.
///
/// When `first_on_current_line` is set, the first key is written at the current
/// cursor position (used after a `- ` marker); otherwise every key is indented.
fn emit_object(
    map: &serde_json::Map<String, Value>,
    indent: usize,
    first_on_current_line: bool,
    out: &mut String,
) {
    if map.is_empty() {
        out.push_str("{}\n");
        return;
    }
    for (i, (key, value)) in map.iter().enumerate() {
        if !(i == 0 && first_on_current_line) {
            push_indent(out, indent);
        }
        out.push_str(&format_key(key));
        out.push(':');
        match value {
            Value::Object(inner) if inner.is_empty() => out.push_str(" {}\n"),
            Value::Array(inner) if inner.is_empty() => out.push_str(" []\n"),
            Value::Object(inner) => {
                out.push('\n');
                emit_object(inner, indent + 2, false, out);
            }
            Value::Array(inner) => {
                out.push('\n');
                emit_array(inner, indent + 2, false, out);
            }
            _ => {
                out.push(' ');
                emit_scalar(value, out);
            }
        }
    }
}

/// Append `indent` spaces to the output.
fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

/// Format a mapping key, quoting it when it is not a plain-safe scalar.
fn format_key(key: &str) -> String {
    if is_plain_safe(key) {
        key.to_string()
    } else {
        quote(key)
    }
}

/// Format a string scalar, choosing plain or double-quoted style.
fn format_scalar(s: &str) -> String {
    if is_plain_safe(s) {
        s.to_string()
    } else {
        quote(s)
    }
}

/// Decide whether a string can be emitted as a plain YAML scalar.
fn is_plain_safe(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.chars().any(char::is_control) {
        return false;
    }
    if s.starts_with([' ', '-']) || s.ends_with(' ') {
        return false;
    }
    if s.contains(": ") || s.contains(" #") {
        return false;
    }
    if s.contains([
        ':', '#', '[', ']', '{', '}', ',', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`',
    ]) {
        return false;
    }
    !matches!(
        s.to_lowercase().as_str(),
        "null" | "~" | "true" | "false" | "yes" | "no" | "on" | "off"
    ) && !looks_numeric(s)
}

/// Detect strings that YAML would reinterpret as numbers or booleans.
fn looks_numeric(s: &str) -> bool {
    !s.is_empty() && s.parse::<f64>().is_ok()
}

/// Emit a double-quoted YAML scalar with escapes.
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yaml<T: Serialize>(v: &T) -> String {
        to_string(v).unwrap()
    }

    #[test]
    fn scalar_map() {
        let mut map = serde_json::Map::new();
        map.insert("a".into(), Value::from(1));
        map.insert("b".into(), Value::from(true));
        map.insert("c".into(), Value::Null);
        assert_eq!(yaml(&Value::Object(map)), "a: 1\nb: true\nc: null\n");
    }

    #[test]
    fn nested_object_and_array() {
        let v: Value = serde_json::json!({
            "violations": [{"rule": "CH001", "severity": "Info"}],
            "count": 1
        });
        assert_eq!(
            yaml(&v),
            "count: 1\nviolations:\n  - rule: CH001\n    severity: Info\n"
        );
    }

    #[test]
    fn nested_mapping_value_starts_on_new_line() {
        let v: Value = serde_json::json!({"outer": {"a": 1}});
        assert_eq!(yaml(&v), "outer:\n  a: 1\n");
    }

    #[test]
    fn nested_sequence_value_starts_on_new_line() {
        let v: Value = serde_json::json!({"violations": [{"rule": "CH001"}]});
        assert_eq!(yaml(&v), "violations:\n  - rule: CH001\n");
    }

    #[test]
    fn no_line_carries_an_inlined_block_sequence() {
        let v: Value = serde_json::json!({
            "report": {
                "violations": [{"rule": "CH001", "details": {"table": "t"}}],
                "meta": {"generated": true}
            }
        });
        for line in yaml(&v).lines() {
            if let Some((_, value)) = line.trim_start().split_once(": ") {
                assert!(
                    !value.trim_start().starts_with("- "),
                    "line `{line}` embeds a block sequence after the colon"
                );
            }
        }
    }

    #[test]
    fn empty_containers() {
        let v: Value = serde_json::json!({"list": [], "obj": {}});
        let out = yaml(&v);
        assert!(out.contains("list: []"));
        assert!(out.contains("obj: {}"));
    }

    #[test]
    fn quotes_ambiguous_strings() {
        let v: Value = serde_json::json!({"n": "42", "b": "true", "s": "a: b", "e": ""});
        let out = yaml(&v);
        assert!(out.contains("n: \"42\""));
        assert!(out.contains("b: \"true\""));
        assert!(out.contains("s: \"a: b\""));
        assert!(out.contains("e: \"\""));
    }

    #[test]
    fn escapes_newlines() {
        let v: Value = serde_json::json!({"text": "line1\nline2"});
        assert!(yaml(&v).contains("text: \"line1\\nline2\""));
    }

    #[test]
    fn array_of_scalars() {
        let v: Value = serde_json::json!(["x", "y"]);
        assert_eq!(yaml(&v), "- x\n- y\n");
    }

    #[test]
    fn top_level_array_of_objects() {
        let v: Value = serde_json::json!([{"a": 1}, {"a": 2}]);
        assert_eq!(yaml(&v), "- a: 1\n- a: 2\n");
    }

    #[test]
    fn nested_array_in_array() {
        let v: Value = serde_json::json!([[1, 2], [3]]);
        assert!(yaml(&v).contains("- - 1"));
    }

    #[test]
    fn quotes_special_keys() {
        let mut map = serde_json::Map::new();
        map.insert("a: b".into(), Value::from(1));
        assert!(yaml(&Value::Object(map)).contains("\"a: b\": 1"));
    }

    #[test]
    fn escapes_control_characters() {
        let v: Value = serde_json::json!({"s": "a\tb"});
        assert!(yaml(&v).contains("s: \"a\\tb\""));
    }

    #[test]
    fn top_level_scalar() {
        assert_eq!(yaml(&Value::from(7)), "7\n");
    }
}

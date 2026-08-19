//! json_edit.rs — the pure JSON mini-library behind superdev's managed keys
//! and array elements: dotted paths with an optional trailing `[marker]`
//! naming an array element. No IO; callers read and write the files.

use crate::error::{Error, Result};

/// Set one dotted key path in a JSON document, creating missing objects on the
/// way. Returns the file content to write and the canonical value text, which
/// is what the lock hashes.
///
/// Every other key survives; their order does not, because serde_json sorts
/// object keys on the way out.
pub(crate) fn edit_json_key(
    path: &str,
    json: &str,
    pointer: &str,
    value_json: &str,
) -> Result<(String, String)> {
    let bad = |message: String| Error::Toml {
        path: path.into(),
        message,
    };
    let mut root: serde_json::Value = serde_json::from_str(json).map_err(|e| bad(e.to_string()))?;
    let value: serde_json::Value = serde_json::from_str(value_json)
        .map_err(|e| bad(format!("invalid value `{value_json}`: {e}")))?;

    let mut segments: Vec<&str> = pointer.split('.').collect();
    let key = segments.pop().expect("split yields at least one segment");
    // Names the container the walk is standing in, for the error message.
    let mut container = "the root".to_string();
    let mut cursor = &mut root;
    for segment in segments {
        cursor = match cursor.as_object_mut() {
            Some(map) => map
                .entry(segment)
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new())),
            None => return Err(bad(format!("{container} is not a JSON object"))),
        };
        container = format!("`{segment}`");
    }
    match cursor.as_object_mut() {
        Some(map) => map.insert(key.to_string(), value.clone()),
        None => return Err(bad(format!("{container} is not a JSON object"))),
    };

    let mut content = serde_json::to_string_pretty(&root).expect("a parsed value re-serialises");
    content.push('\n');
    Ok((content, value.to_string()))
}

/// Ensure the array at a dotted key path contains `value_json`: the first
/// element whose serialised form contains `marker` is replaced, else the
/// element is appended. Missing objects on the way — and the array itself —
/// are created. Returns the file content to write and the canonical element
/// text, which is what the lock hashes.
pub(crate) fn edit_json_array_element(
    path: &str,
    json: &str,
    pointer: &str,
    marker: &str,
    value_json: &str,
) -> Result<(String, String)> {
    let bad = |message: String| Error::Toml {
        path: path.into(),
        message,
    };
    let mut root: serde_json::Value = serde_json::from_str(json).map_err(|e| bad(e.to_string()))?;
    let value: serde_json::Value = serde_json::from_str(value_json)
        .map_err(|e| bad(format!("invalid value `{value_json}`: {e}")))?;

    let mut container = "the root".to_string();
    let mut segment_name = "the root";
    let mut cursor = &mut root;
    for segment in pointer.split('.') {
        cursor = match cursor.as_object_mut() {
            Some(map) => map
                .entry(segment)
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new())),
            None => return Err(bad(format!("{container} is not a JSON object"))),
        };
        container = format!("`{segment}`");
        segment_name = segment;
    }
    // The walk mints an empty object for a missing final segment; the pointer
    // names an array, so turn that placeholder into one.
    if cursor.as_object().is_some_and(serde_json::Map::is_empty) {
        *cursor = serde_json::Value::Array(Vec::new());
    }
    let Some(items) = cursor.as_array_mut() else {
        return Err(bad(format!("`{segment_name}` is not a JSON array")));
    };
    match items
        .iter_mut()
        .find(|item| item.to_string().contains(marker))
    {
        Some(item) => *item = value.clone(),
        None => items.push(value.clone()),
    }

    let mut content = serde_json::to_string_pretty(&root).expect("a parsed value re-serialises");
    content.push('\n');
    Ok((content, value.to_string()))
}

/// Split a lock-style pointer into dotted segments and the optional trailing
/// `[marker]` naming an array element.
pub(crate) fn parse_pointer(pointer: &str) -> (Vec<&str>, Option<&str>) {
    match pointer.split_once('[') {
        Some((dotted, rest)) => (
            dotted.split('.').collect(),
            Some(rest.strip_suffix(']').unwrap_or(rest)),
        ),
        None => (pointer.split('.').collect(), None),
    }
}

/// The canonical value text at `pointer`: the object key's value, or the
/// array element whose serialised form contains the marker — the same rule
/// `edit_json_array_element` matches by. `Ok(None)` when absent.
pub(crate) fn json_value_at(path: &str, json: &str, pointer: &str) -> Result<Option<String>> {
    let root: serde_json::Value = serde_json::from_str(json).map_err(|e| Error::Toml {
        path: path.into(),
        message: e.to_string(),
    })?;
    let (segments, marker) = parse_pointer(pointer);
    let mut cursor = &root;
    for segment in segments {
        match cursor.get(segment) {
            Some(next) => cursor = next,
            None => return Ok(None),
        }
    }
    let value = match marker {
        None => Some(cursor),
        Some(marker) => cursor
            .as_array()
            .and_then(|items| items.iter().find(|item| item.to_string().contains(marker))),
    };
    Ok(value.map(ToString::to_string))
}

/// Remove the entry `pointer` names. Returns the new file content and the
/// removed canonical value; `Ok(None)` when absent. Empty parents stay.
pub(crate) fn remove_json_pointer(
    path: &str,
    json: &str,
    pointer: &str,
) -> Result<Option<(String, String)>> {
    let bad = |message: String| Error::Toml {
        path: path.into(),
        message,
    };
    let mut root: serde_json::Value = serde_json::from_str(json).map_err(|e| bad(e.to_string()))?;
    let (mut segments, marker) = parse_pointer(pointer);
    let last = if marker.is_none() {
        segments.pop()
    } else {
        None
    };
    let mut cursor = &mut root;
    for segment in segments {
        match cursor.get_mut(segment) {
            Some(next) => cursor = next,
            None => return Ok(None),
        }
    }
    let removed = match (last, marker) {
        (Some(key), None) => cursor.as_object_mut().and_then(|map| map.remove(key)),
        (_, Some(marker)) => cursor.as_array_mut().and_then(|items| {
            let index = items
                .iter()
                .position(|item| item.to_string().contains(marker))?;
            Some(items.remove(index))
        }),
        (None, None) => None,
    };
    Ok(removed.map(|value| {
        let mut content =
            serde_json::to_string_pretty(&root).expect("a parsed value re-serialises");
        content.push('\n');
        (content, value.to_string())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointers_parse_navigate_and_remove() {
        assert_eq!(parse_pointer("a.b"), (vec!["a", "b"], None));
        assert_eq!(
            parse_pointer("hooks.PostToolUse[superdev aokf hook validate]"),
            (
                vec!["hooks", "PostToolUse"],
                Some("superdev aokf hook validate")
            )
        );

        let json = r#"{"mcpServers":{"superdev-aokf":{"command":"superdev"},"mine":{}}}"#;
        let value = json_value_at("f", json, "mcpServers.superdev-aokf")
            .unwrap()
            .unwrap();
        assert!(value.contains("superdev"));
        assert_eq!(json_value_at("f", json, "mcpServers.gone").unwrap(), None);
        assert!(json_value_at("f", "not json", "a").is_err());

        let (content, removed) = remove_json_pointer("f", json, "mcpServers.superdev-aokf")
            .unwrap()
            .unwrap();
        assert!(removed.contains("superdev"));
        let root: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(root["mcpServers"].get("superdev-aokf").is_none());
        // The user's key and the (possibly emptied) parent survive.
        assert!(root["mcpServers"].get("mine").is_some());
        assert_eq!(
            remove_json_pointer("f", json, "mcpServers.gone").unwrap(),
            None
        );

        let hooks = r#"{"hooks":{"PostToolUse":[{"matcher":"Agent","hooks":[]},{"matcher":"Edit|Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}]}}"#;
        let pointer = "hooks.PostToolUse[superdev aokf hook validate]";
        assert!(
            json_value_at("f", hooks, pointer)
                .unwrap()
                .unwrap()
                .contains("Edit|Write")
        );
        let (content, _) = remove_json_pointer("f", hooks, pointer).unwrap().unwrap();
        let root: serde_json::Value = serde_json::from_str(&content).unwrap();
        let items = root["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(items.len(), 1, "only superdev's element goes");
        assert_eq!(items[0]["matcher"], "Agent");
    }
}

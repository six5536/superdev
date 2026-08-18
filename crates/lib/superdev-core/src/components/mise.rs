//! Format-preserving edits of the `[tools]` table in a shared `.mise.toml`.
//! Only superdev-managed keys are ever touched.

use toml_edit::{DocumentMut, Item, Table, Value};

use crate::error::{Error, Result};

fn parse(mise_toml: &str) -> Result<DocumentMut> {
    mise_toml.parse::<DocumentMut>().map_err(|e| Error::Toml {
        path: ".mise.toml".into(),
        message: e.to_string(),
    })
}

/// Parse a TOML value fragment by wrapping it in a scratch document.
fn parse_fragment(value_toml: &str) -> Result<Item> {
    let doc = format!("x = {value_toml}")
        .parse::<DocumentMut>()
        .map_err(|e| Error::Toml {
            path: ".mise.toml".into(),
            message: format!("invalid pin value `{value_toml}`: {e}"),
        })?;
    Ok(doc["x"].clone())
}

/// Strip whitespace and comments from a value and everything inside it, so it
/// renders in toml_edit's default one-line form.
fn clear_decor(value: &mut Value) {
    value.decor_mut().clear();
    match value {
        Value::Array(array) => {
            for element in array.iter_mut() {
                clear_decor(element);
            }
            array.set_trailing("");
            array.set_trailing_comma(false);
        }
        Value::InlineTable(table) => {
            for (mut key, element) in table.iter_mut() {
                key.leaf_decor_mut().clear();
                key.dotted_decor_mut().clear();
                clear_decor(element);
            }
            table.set_trailing("");
            table.set_trailing_comma(false);
        }
        _ => {}
    }
}

/// Render a pin value on its own, without the surrounding line's decor.
///
/// Comments attach to values as decor, so without clearing they would end up
/// inside the fragment — a trailing `# keep` on the line, or one on any element
/// of an array or inline table.
fn render(item: &Item) -> String {
    match item {
        Item::Value(value) => {
            let mut value = value.clone();
            clear_decor(&mut value);
            value.to_string().trim().to_string()
        }
        other => other.to_string().trim().to_string(),
    }
}

/// The `[tools]` value for `tool`, rendered as a trimmed TOML fragment.
///
/// `None` when the tool is not pinned.
pub fn current_pin(mise_toml: &str, tool: &str) -> Result<Option<String>> {
    let doc = parse(mise_toml)?;
    Ok(doc.get("tools").and_then(|t| t.get(tool)).map(render))
}

/// Set one `[tools]` key, preserving everything else in the file.
///
/// Creates the `[tools]` table when it is missing. A `tools` key holding
/// anything else is an error: the file is valid TOML but not a mise config,
/// and indexing into it would panic.
pub fn set_pin(mise_toml: &str, tool: &str, value_toml: &str) -> Result<String> {
    let mut doc = parse(mise_toml)?;
    let item = parse_fragment(value_toml)?;
    match doc.get("tools") {
        None => doc["tools"] = Item::Table(Table::new()),
        Some(tools) if !tools.is_table_like() => {
            return Err(Error::Toml {
                path: ".mise.toml".into(),
                message: "`tools` is not a table".into(),
            });
        }
        Some(_) => {}
    }
    doc["tools"][tool] = item;
    Ok(doc.to_string())
}

/// Remove one `[tools]` key, preserving everything else. `None` when the tool
/// is not pinned. An emptied `[tools]` table stays: guessing which empty
/// containers a user wants gone is worse than the residue.
pub fn remove_pin(mise_toml: &str, tool: &str) -> Result<Option<String>> {
    let mut doc = parse(mise_toml)?;
    let removed = doc
        .get_mut("tools")
        .and_then(Item::as_table_like_mut)
        .and_then(|tools| tools.remove(tool));
    Ok(removed.map(|_| doc.to_string()))
}

/// Prefix of every managed pin's lock key. `Claim::parse_key` decodes by it,
/// so encode and decode share one definition.
pub(crate) const PIN_LOCK_PREFIX: &str = ".mise.toml:";

/// Lock `files` key under which a managed pin's value hash is recorded.
pub fn pin_lock_key(tool: &str) -> String {
    format!("{PIN_LOCK_PREFIX}{tool}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "# my tools\n[tools]\nnode = \"24\" # keep\nrust = ['1.96', 'nightly']\n";

    #[test]
    fn reads_existing_and_missing_pins() {
        assert_eq!(
            current_pin(SAMPLE, "node").unwrap().as_deref(),
            Some("\"24\"")
        );
        assert_eq!(current_pin(SAMPLE, "http:superpowers").unwrap(), None);
    }

    #[test]
    fn composite_pins_render_without_element_comments() {
        let multiline = "[tools]\nrust = [\n  '1.96', # first\n  'nightly', # second\n]\n";
        assert_eq!(
            current_pin(multiline, "rust").unwrap().as_deref(),
            Some("['1.96', 'nightly']")
        );

        let nested = "[tools]\n\"http:sp\" = { version = \"6.2.0\", # why\n  assets = [\n    'a', # one\n  ] }\n";
        assert_eq!(
            current_pin(nested, "http:sp").unwrap().as_deref(),
            Some("{ version = \"6.2.0\", assets = ['a'] }")
        );
    }

    #[test]
    fn set_pin_preserves_everything_else() {
        let out = set_pin(SAMPLE, "http:superpowers", "{ version = \"6.2.0\" }").unwrap();
        assert!(out.contains("# my tools"));
        assert!(out.contains("node = \"24\" # keep"));
        assert!(out.contains("rust = ['1.96', 'nightly']"));
        assert_eq!(
            current_pin(&out, "http:superpowers").unwrap().as_deref(),
            Some("{ version = \"6.2.0\" }")
        );
    }

    #[test]
    fn set_pin_creates_tools_table_in_empty_file() {
        let out = set_pin("", "node", "\"24\"").unwrap();
        assert_eq!(
            current_pin(&out, "node").unwrap().as_deref(),
            Some("\"24\"")
        );
    }

    #[test]
    fn remove_pin_takes_one_key_and_leaves_the_rest() {
        let with = set_pin(SAMPLE, "http:codegraph", "\"1.5.0\"").unwrap();
        let out = remove_pin(&with, "http:codegraph").unwrap().unwrap();
        assert_eq!(current_pin(&out, "http:codegraph").unwrap(), None);
        assert!(out.contains("# my tools"));
        assert!(out.contains("node = \"24\" # keep"));
        // Not pinned: nothing to write.
        assert!(remove_pin(SAMPLE, "http:codegraph").unwrap().is_none());
        // The emptied [tools] table stays.
        let only = set_pin("", "http:codegraph", "\"1.5.0\"").unwrap();
        let out = remove_pin(&only, "http:codegraph").unwrap().unwrap();
        assert!(out.contains("[tools]"), "{out}");
        // Malformed file: an error, never a guess.
        assert!(remove_pin("[tools\n", "http:codegraph").is_err());
    }

    #[test]
    fn invalid_fragment_is_a_toml_error() {
        assert!(set_pin(SAMPLE, "node", "{ not valid").is_err());
    }

    #[test]
    fn non_table_tools_key_is_a_toml_error() {
        // Valid TOML, but not a mise config: indexing `tools` would panic.
        let e = set_pin("tools = 3\n", "node", "\"24\"").unwrap_err();
        assert_eq!(e.to_string(), ".mise.toml: `tools` is not a table");
        // Reading the same file is harmless.
        assert_eq!(current_pin("tools = 3\n", "node").unwrap(), None);
        // An inline table is a table.
        assert!(set_pin("tools = { node = \"24\" }\n", "rust", "\"1.96\"").is_ok());
    }

    #[test]
    fn malformed_file_is_a_toml_error() {
        let e = current_pin("[tools\n", "node").unwrap_err();
        assert!(e.to_string().starts_with(".mise.toml:"));
    }

    #[test]
    fn lock_key_names_the_file_and_tool() {
        assert_eq!(
            pin_lock_key("http:superpowers"),
            ".mise.toml:http:superpowers"
        );
    }
}

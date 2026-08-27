//! doc.rs — the grammar rendered as prose.
//!
//! A behavioural port of the reference's `renderDoc`: the grammar is the only
//! statement of the language, and this prints it in a form a reader can follow
//! without reading YAML. `tests/fixtures/format/doc.golden.txt` holds the
//! reference's output for the shipped grammar, captured before any of this
//! existed.
//!
//! The reference builds a line vector and joins it with newlines, so a pushed
//! empty string is a blank line; this does the same, because the golden pins
//! the blank lines as much as the prose.

use serde_yaml_ng::Value;

use super::grammar::{Attr, Element, Grammar};

/// The grammar as markdown, ending in a newline.
///
/// The reference prints this with `console.log`, which adds one more; a caller
/// wanting the reference's bytes prints with `println!`.
#[must_use]
pub fn render(g: &Grammar) -> String {
    let mut out: Vec<String> = Vec::new();
    let push = |out: &mut Vec<String>, line: &str| out.push(line.to_string());

    push(&mut out, &format!("# {} grammar {}", g.grammar, g.version));
    push(&mut out, "");
    push(&mut out, &g.doc);
    push(&mut out, "");

    conditions(g, &mut out);
    frontmatter(g, &mut out);
    elements(g, &mut out);

    push(&mut out, "## Element order");
    push(&mut out, "");
    let order: Vec<String> = g
        .kinds
        .unit
        .order
        .iter()
        .map(|tag| format!("`{tag}`"))
        .collect();
    push(&mut out, &order.join(" → "));
    push(&mut out, "");

    out.join("\n")
}

/// The one condition attribute, its four forms, and the spellings it replaced.
fn conditions(g: &Grammar, out: &mut Vec<String>) {
    let c = &g.conditions;
    out.push("## Conditions".into());
    out.push(String::new());
    out.push(c.doc.clone());
    out.push(String::new());
    out.push(format!(
        "One attribute, `{}`, on every element that can bear a condition:",
        c.attribute
    ));
    out.push(String::new());
    for form in &c.forms {
        // The reference splits on every " —" and takes the second field, so a
        // form carrying two dashes keeps only the text between them.
        let mut parts = form.split(" —");
        let name = parts.next().unwrap_or_default();
        out.push(format!("- `{name}` —{}", parts.next().unwrap_or_default()));
    }
    out.push(String::new());
    for (from, note) in c.renamed_from.iter() {
        out.push(format!("`{from}` is not part of the grammar: {note}."));
    }
}

/// The host fields a unit carries: the profiles that require them, then the
/// keys themselves.
fn frontmatter(g: &Grammar, out: &mut Vec<String>) {
    let f = &g.kinds.unit.frontmatter;
    out.push(String::new());
    out.push("## Unit frontmatter".into());
    out.push(String::new());
    out.push(f.doc.clone());
    out.push(String::new());

    for p in &f.profiles {
        let where_ = p.matches.as_ref().map_or_else(String::new, |m| {
            m.basename
                .iter()
                .chain(m.suffix.iter())
                .map(|s| format!("`{s}`"))
                .collect::<Vec<_>>()
                .join(", ")
        });
        let matches = if where_.is_empty() {
            String::new()
        } else {
            format!(" Matches {where_}.")
        };
        let required: Vec<String> = p.required.iter().map(|k| format!("`{k}`")).collect();
        let required = if required.is_empty() {
            "nothing".to_string()
        } else {
            required.join(", ")
        };
        out.push(format!(
            "- **{}** — {}{matches} Requires {required}.",
            p.name, p.doc
        ));
    }
    out.push(String::new());

    for (key, def) in f.keys.iter() {
        let mut bits = vec![def.r#type.clone()];
        if !def.r#enum.is_empty() {
            bits.push(format!("one of {}", join_scalars(&def.r#enum)));
        }
        if let Some(pattern) = &def.pattern {
            bits.push(format!("matching `{pattern}`"));
        }
        if let Some(max) = def.max_length {
            bits.push(format!("at most {max} chars"));
        }
        if !def.portable {
            bits.push(format!("{}: no", f.portability.spec));
        }
        let doc = def
            .doc
            .as_ref()
            .map_or_else(String::new, |d| format!(". {d}"));
        out.push(format!("- `{key}` — {}{doc}", bits.join("; ")));
    }

    out.push(String::new());
    out.push(format!(
        "Keys marked \"{}: no\" are {} ({}).",
        f.portability.spec, f.portability.warn, f.portability.url
    ));
    out.push(String::new());
}

/// Every element: its signature, what it is, where it sits, and its attributes.
fn elements(g: &Grammar, out: &mut Vec<String>) {
    out.push(String::new());
    out.push("## Unit elements".into());
    out.push(String::new());

    for (tag, def) in g.kinds.unit.elements.iter() {
        out.push(format!("### `{tag}`"));
        out.push(String::new());
        out.push("```xml".into());
        out.push(signature(tag, def));
        out.push("```".into());
        out.push(String::new());
        out.push(def.doc.clone());

        let parents = def.parent.names();
        if !parents.is_empty() {
            let named: Vec<String> = parents.iter().map(|p| format!("`<{p}>`")).collect();
            out.push(format!("Sits inside {}.", named.join(" or ")));
        }
        if let Some(occurs) = &def.occurs {
            out.push(format!("Occurs {}–{} times.", occurs.min, occurs.max));
        }
        for (name, attr) in def.attrs.iter() {
            if attr.doc.is_none() && attr.r#enum.is_empty() && attr.r#const.is_none() {
                continue;
            }
            out.push(format!(
                "- `{name}`{} — {}{}",
                if attr.required { " (required)" } else { "" },
                attr.doc.as_deref().unwrap_or_default(),
                extra(attr)
            ));
        }
        out.push(String::new());
    }
}

/// The element as it is written: `<tag …/>`, `<tag …>…</tag>`, or both.
fn signature(tag: &str, def: &Element) -> String {
    let attrs: Vec<String> = def
        .attrs
        .iter()
        .map(|(name, a)| format!("{name}=\"…\"{}", if a.required { "" } else { "?" }))
        .collect();
    let attrs = attrs.join(" ");
    match def.form.as_str() {
        "self-closing" => format!("<{tag} {attrs} />"),
        "block" => {
            let attrs = if attrs.is_empty() {
                String::new()
            } else {
                format!(" {attrs}")
            };
            format!("<{tag}{attrs}>…</{tag}>")
        }
        _ => format!("<{tag} {attrs} />  |  <{tag} …>…</{tag}>"),
    }
}

/// What follows an attribute's own words: its closed set, or its one value.
fn extra(attr: &Attr) -> String {
    if !attr.r#enum.is_empty() {
        format!(" One of {}.", attr.r#enum.join(", "))
    } else if let Some(value) = &attr.r#const {
        format!(" Always `{value}`.")
    } else {
        String::new()
    }
}

/// YAML scalars joined the way the reference's `Array.join` renders them.
fn join_scalars(values: &[Value]) -> String {
    values.iter().map(scalar).collect::<Vec<_>>().join(", ")
}

/// One scalar as JavaScript's `String()` would print it. Anything that is not
/// a scalar has no place in an enum, and renders empty rather than as YAML.
fn scalar(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => "null".to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    /// The shipped grammar, read from the repository this crate lives in.
    fn live() -> Grammar {
        let path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "../../..",
            ".agents/format/grammar.yaml",
        ]
        .iter()
        .collect();
        super::super::parse_grammar(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    /// The render equals what the reference printed for the same grammar.
    ///
    /// The golden is the reference's stdout, so it carries the newline
    /// `console.log` adds on top of the one the render ends with. Treat it as
    /// fixed: the script that produced it is gone.
    #[test]
    fn the_render_equals_the_captured_golden() {
        let golden = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/format/doc.golden.txt"),
        )
        .unwrap();
        assert_eq!(format!("{}\n", render(&live())), golden);
    }

    #[test]
    fn scalars_render_as_javascript_prints_them() {
        assert_eq!(scalar(&Value::String("low".into())), "low");
        assert_eq!(scalar(&Value::Bool(true)), "true");
        assert_eq!(scalar(&Value::Number(2.into())), "2");
        assert_eq!(scalar(&Value::Null), "null");
        assert_eq!(scalar(&Value::Sequence(vec![])), "");
    }

    /// The three forms, since the shipped grammar exercises only what it uses.
    #[test]
    fn a_block_element_without_attributes_has_no_trailing_space() {
        let g = live();
        let rules = g.kinds.unit.elements.get("rules").unwrap();
        assert_eq!(signature("rules", rules), "<rules>…</rules>");
    }
}

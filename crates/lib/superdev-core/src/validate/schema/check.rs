//! check.rs — the per-kind and cross-file checks.
//!
//! Began as a behavioural port of the Node reference this replaced. Every
//! finding's text is the contract: `tests/format_snapshots.rs` records the
//! report for one tree per failure class and compares it verbatim.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use std::path::Path;

use super::grammar::{Frontmatter, Grammar};
use super::re;
use super::read::{
    Node, fence_map, fm_value, norm, parse_elements, parse_frontmatter, prose_only,
    split_frontmatter, unfenced,
};

/// Every tag the scanner reads, shared by the unit and core checks.
static TAG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<(/)?([A-Za-z_][\w-]*)((?:\s+[\w-]+\s*=\s*"[^"]*")*)\s*(/)?>"#).unwrap()
});

/// A YAML scalar as the reference renders it when it joins a list into a
/// message. Everything the grammar puts in such a list is a string; anything
/// else falls back to its own rendering rather than being dropped.
fn scalar(v: &serde_yaml_ng::Value) -> String {
    v.as_str().map_or_else(
        || {
            serde_yaml_ng::to_string(v)
                .unwrap_or_default()
                .trim()
                .to_string()
        },
        str::to_string,
    )
}

/// The first `n` characters, which is what the reference's `slice` gives for
/// everything the grammar puts in a message.
fn cut(text: &str, n: usize) -> String {
    text.chars().take(n).collect()
}

/// A unit's frontmatter belongs to its host, so it is checked against the
/// host's field table rather than against the format's own vocabulary.
pub fn check_frontmatter(
    file: &Path,
    fm: &[&str],
    f: &Frontmatter,
    errs: &mut Vec<String>,
    warns: &mut Vec<String>,
) {
    let base = file
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let by_basename = f.profiles.iter().find(|p| {
        p.matches
            .as_ref()
            .is_some_and(|m| m.basename.iter().any(|b| b == base))
    });
    let by_suffix = || {
        f.profiles.iter().find(|p| {
            p.matches
                .as_ref()
                .is_some_and(|m| m.suffix.iter().any(|s| base.ends_with(s)))
        })
    };
    let Some(profile) = by_basename
        .or_else(by_suffix)
        .or_else(|| f.profiles.iter().find(|p| p.default))
    else {
        return;
    };

    let entries = parse_frontmatter(fm);
    let bools: Vec<String> = f
        .boolean_values
        .iter()
        .map(|v| scalar(v).to_lowercase())
        .collect();
    let boolean_list = f
        .boolean_values
        .iter()
        .map(scalar)
        .collect::<Vec<String>>()
        .join(", ");
    let mut seen: Vec<(String, usize)> = Vec::new();

    for e in &entries {
        if let Some((_, first)) = seen.iter().find(|(k, _)| *k == e.key) {
            errs.push(format!(
                "frontmatter: duplicate key \"{}\" (lines {first} and {})",
                e.key, e.line
            ));
        }
        seen.push((e.key.clone(), e.line));

        let Some(def) = f.keys.get(&e.key) else {
            errs.push(format!(
                "frontmatter: unknown key \"{}\" — not a Claude Code skill field",
                e.key
            ));
            continue;
        };
        if !profile.allow.is_empty() && !profile.allow.contains(&e.key) {
            errs.push(format!(
                "frontmatter: \"{}\" is not accepted in a {} file",
                e.key, profile.name
            ));
            continue;
        }
        if e.scalar.is_none() && e.block.is_none() {
            errs.push(format!("frontmatter: \"{}\" has no value", e.key));
            continue;
        }

        let is_block = e.block.is_some() && e.scalar.is_none();
        match def.r#type.as_str() {
            "boolean" => {
                let v = e.scalar.clone().unwrap_or_default();
                if is_block || !bools.contains(&v.to_lowercase()) {
                    let shown = e.scalar.clone().unwrap_or_else(|| "(block)".to_string());
                    errs.push(format!(
                        "frontmatter: \"{}\" must be a boolean ({boolean_list}), got \"{shown}\"",
                        e.key
                    ));
                }
            }
            "map" => {
                if !is_block || e.is_list {
                    errs.push(format!("frontmatter: \"{}\" must be a map", e.key));
                }
            }
            "stringOrList" => {
                if is_block && !e.is_list {
                    errs.push(format!(
                        "frontmatter: \"{}\" must be a string or a YAML list",
                        e.key
                    ));
                }
            }
            _ => {
                if is_block && !e.is_folded {
                    errs.push(format!("frontmatter: \"{}\" must be a string", e.key));
                }
            }
        }

        if let Some(value) = &e.scalar {
            if !def.r#enum.is_empty() && !def.r#enum.iter().any(|x| scalar(x) == *value) {
                let allowed = def
                    .r#enum
                    .iter()
                    .map(scalar)
                    .collect::<Vec<String>>()
                    .join(", ");
                errs.push(format!(
                    "frontmatter: \"{}\" is \"{value}\", not one of {allowed}",
                    e.key
                ));
            }
            if let Some(pattern) = &def.pattern
                && !re::matches(pattern, value)
            {
                errs.push(format!(
                    "frontmatter: \"{}\" is \"{value}\", which does not match {pattern}",
                    e.key
                ));
            }
            if let Some(max) = def.max_length
                && value.chars().count() > max
            {
                warns.push(format!(
                    "frontmatter: \"{}\" is {} characters, over the {max} the host keeps",
                    e.key,
                    value.chars().count()
                ));
            }
        }
        if !def.portable {
            warns.push(format!(
                "frontmatter: \"{}\" is {}",
                e.key, f.portability.warn
            ));
        }
    }

    for key in &profile.required {
        if !seen.iter().any(|(k, _)| k == key) {
            errs.push(format!(
                "frontmatter: {key} missing (required in a {} file)",
                profile.name
            ));
        }
    }

    if profile.name_matches_directory && seen.iter().any(|(k, _)| k == "name") {
        let dir = file
            .parent()
            .and_then(Path::file_name)
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if let Some(name) = fm_value(fm, "name")
            && name != dir
        {
            errs.push(format!(
                "frontmatter: name \"{name}\" does not match the skill directory \"{dir}\", which is what the command is named after"
            ));
        }
    }
}

/// A unit file: frontmatter against the host's table, then the element
/// vocabulary.
///
/// Returns the elements read off the *unfenced* text — structure is read from
/// the prose, but a statement's wording includes the code spans the prose scan
/// blanks, so the comparables and the load check read the same elements again
/// from text that kept them.
pub fn check_unit(
    file: &Path,
    text: &str,
    errs: &mut Vec<String>,
    warns: &mut Vec<String>,
    g: &Grammar,
) -> Option<Vec<Node>> {
    let k = &g.kinds.unit;
    let lines: Vec<&str> = text.split('\n').collect();
    let fenced = fence_map(&lines);
    let Some(split) = split_frontmatter(&lines) else {
        errs.push("missing YAML frontmatter".to_string());
        return None;
    };

    check_frontmatter(file, &split.fm, &k.frontmatter, errs, warns);

    let prose = prose_only(&lines, &fenced);
    let nodes = parse_elements(&prose, errs);

    for (tag, message) in k.removed.iter() {
        if nodes.iter().any(|n| n.name == tag) {
            errs.push(message.clone());
        }
    }

    let element_names = k.elements.names().collect::<Vec<&str>>().join("/");
    let mut counts: Vec<(String, usize)> = Vec::new();

    for node in &nodes {
        let Some(def) = k.elements.get(&node.name) else {
            if !k.removed.has(&node.name) {
                errs.push(format!(
                    "unknown tag <{}> (unit files use only {element_names})",
                    node.name
                ));
            }
            continue;
        };
        match counts.iter_mut().find(|(n, _)| *n == node.name) {
            Some((_, c)) => *c += 1,
            None => counts.push((node.name.clone(), 1)),
        }

        let want = def.parent.names();
        let here = node.parent.as_deref();
        // An element with no declared parent belongs at the file root.
        let nested_correctly = if want.is_empty() {
            here.is_none()
        } else {
            here.is_some_and(|p| want.contains(&p))
        };
        if !nested_correctly {
            let names = if want.is_empty() {
                "(the file root)".to_string()
            } else {
                want.iter()
                    .map(|w| format!("<{w}>"))
                    .collect::<Vec<String>>()
                    .join(" or ")
            };
            let found = here.map_or_else(|| "(the file root)".to_string(), |p| p.to_string());
            errs.push(format!(
                "<{}> must sit directly inside {names}, found inside <{found}>",
                node.name
            ));
        }

        if def.form == "self-closing" && !node.self_closing {
            errs.push(format!(
                "<{}> must be self-closing: {}",
                node.name,
                cut(&node.raw, 70)
            ));
        }
        if def.form == "block" && node.self_closing {
            errs.push(format!(
                "<{}> must have a body: {}",
                node.name,
                cut(&node.raw, 70)
            ));
        }
        let body = norm(&node.body);
        if def.body_required == Some(true) && body.is_empty() {
            errs.push(format!("<{}> is empty", node.name));
        }
        if let Some(forbid) = &def.body_forbid
            && !body.is_empty()
        {
            let pattern = if forbid.flags.as_deref() == Some("i") {
                format!("(?i){}", forbid.pattern)
            } else {
                forbid.pattern.clone()
            };
            if re::matches(&pattern, &body) {
                errs.push(format!(
                    "<{}>: {}: \"{}\"",
                    node.name,
                    forbid.message,
                    cut(&body, 60)
                ));
            }
        }

        for (key, value) in &node.attrs {
            let Some(ad) = def.attrs.get(key) else {
                let renamed = g.conditions.renamed_from.get(key);
                if let Some(note) = renamed
                    && def.attrs.has(&g.conditions.attribute)
                {
                    errs.push(format!(
                        "<{}> attribute \"{key}\" {note}: {}",
                        node.name,
                        cut(&node.raw, 70)
                    ));
                } else {
                    errs.push(format!(
                        "<{}> unknown attribute \"{key}\": {}",
                        node.name,
                        cut(&node.raw, 70)
                    ));
                }
                continue;
            };
            if let Some(want) = &ad.r#const
                && value != want
            {
                errs.push(format!(
                    "<{}> {key} must be {want}, got \"{value}\"",
                    node.name
                ));
            }
            if !ad.r#enum.is_empty() && !ad.r#enum.contains(value) {
                errs.push(format!(
                    "bad {} {key} \"{value}\": {}",
                    node.name,
                    cut(&node.raw, 70)
                ));
            }
            if ad.condition && !re::matches(&g.conditions.pattern, value) {
                let forms = g
                    .conditions
                    .forms
                    .iter()
                    .map(|f| f.split(" —").next().unwrap_or(f).to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                errs.push(format!(
                    "bad {} value \"{value}\" on <{}> (use {forms})",
                    g.conditions.attribute, node.name
                ));
            }
            let angle = &k.checks.attributes_free_of_angle_brackets;
            if angle.enabled && (value.contains('<') || value.contains('>')) {
                errs.push(format!("{}: {key}=\"{}\"", angle.message, cut(value, 60)));
            }
        }
        for (key, ad) in def.attrs.iter() {
            if ad.required && node.attr(key).is_none() {
                errs.push(format!(
                    "<{}> missing {key} attribute: {}",
                    node.name,
                    cut(&node.raw, 70)
                ));
            }
        }

        if !def.at_most_one_of.is_empty() {
            let present = def
                .at_most_one_of
                .iter()
                .filter(|a| node.attr(a).is_some())
                .count();
            if present > 1 {
                errs.push(format!(
                    "<{}> takes at most one of {}: {}",
                    node.name,
                    def.at_most_one_of.join("/"),
                    cut(&node.raw, 70)
                ));
            }
        }
        if !def.exactly_one_of.is_empty() {
            let label = |k: &str| {
                if k == "body" {
                    "a body".to_string()
                } else {
                    format!("`{}`", &k[1..])
                }
            };
            let present: Vec<&String> = def
                .exactly_one_of
                .iter()
                .filter(|k| {
                    if *k == "body" {
                        !body.is_empty()
                    } else {
                        node.attr(&k[1..]).is_some()
                    }
                })
                .collect();
            let ident = node.attr("name").map_or_else(
                || format!("<{}>", node.name),
                |n| format!("<{} name=\"{n}\">", node.name),
            );
            if present.len() != 1 {
                let what = if present.is_empty() {
                    format!(
                        "needs exactly one of {}",
                        def.exactly_one_of
                            .iter()
                            .map(|k| label(k))
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                } else {
                    format!(
                        "has {}; use one",
                        present
                            .iter()
                            .map(|k| label(k))
                            .collect::<Vec<String>>()
                            .join(" and ")
                    )
                };
                errs.push(format!("{ident} {what}: {}", cut(&node.raw, 60)));
            }
        }
        if let Some(must) = &def.must_contain {
            let end = node.index + node.raw.len() + node.body.len();
            let ok = must.any_of.iter().any(|t| {
                nodes.iter().any(|n| {
                    n.parent.as_deref() == Some(node.name.as_str())
                        && n.index > node.index
                        && n.index < end
                        && n.name == *t
                })
            });
            if !ok {
                let wanted = must
                    .any_of
                    .iter()
                    .map(|t| format!("<{t}>"))
                    .collect::<Vec<String>>()
                    .join(", ");
                errs.push(format!("<{}> has no {wanted} entries", node.name));
            }
        }
    }

    for (tag, def) in k.elements.iter() {
        let Some(occ) = &def.occurs else { continue };
        let n = counts.iter().find(|(t, _)| t == tag).map_or(0, |(_, c)| *c);
        if n < occ.min {
            errs.push(format!("missing <{tag}> block"));
        }
        if n > occ.max {
            errs.push(format!(
                "expected at most {} <{tag}> block, found {n}",
                occ.max
            ));
        }
    }

    let base = file
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    for mirror in &k.frontmatter.mirrors {
        if !mirror.basename.is_empty() && base != mirror.basename {
            continue;
        }
        let Some(node) = nodes.iter().find(|n| n.name == mirror.element) else {
            continue;
        };
        let (Some(fv), Some(av)) = (fm_value(&split.fm, &mirror.key), node.attr(&mirror.attr))
        else {
            continue;
        };
        if fv != av {
            errs.push(format!(
                "frontmatter {} \"{fv}\" does not match <{} {}=\"{av}\">",
                mirror.key, mirror.element, mirror.attr
            ));
        }
    }

    let seen: Vec<(&String, usize)> = k
        .order
        .iter()
        .filter_map(|t| nodes.iter().find(|n| n.name == *t).map(|n| (t, n.index)))
        .collect();
    for pair in seen.windows(2) {
        if pair[1].1 < pair[0].1 {
            errs.push(format!("<{}> must come after <{}>", pair[1].0, pair[0].0));
            break;
        }
    }

    let raw_nodes = parse_elements(&unfenced(&lines, &fenced), &mut Vec::new());

    let load = &k.checks.steps_are_not_pure_loads;
    if load.enabled {
        let verbs = load
            .verbs
            .iter()
            .map(|v| v.split_whitespace().collect::<Vec<&str>>().join(r"\s+"))
            .collect::<Vec<String>>()
            .join("|");
        let starts = re::compile(&format!(r"^({verbs})\b"));
        let mentions = re::compile(&format!("`?({})`?", g.tools.roster.join("|")));
        if let (Some(starts), Some(mentions)) = (starts, mentions) {
            for node in &raw_nodes {
                if node.name != "step" {
                    continue;
                }
                let body = norm(node.attr("task").unwrap_or(&node.body));
                if starts.is_match(&body) && mentions.is_match(&body) {
                    errs.push(format!("{}: \"{}\"", load.message, cut(&body, 80)));
                }
            }
        }
    }

    Some(raw_nodes)
}

/// One map against a declared key table: unknown keys, missing required keys,
/// declared types, enums, regex validity, and cross-key requirements.
fn check_keys(
    obj: &serde_yaml_ng::Mapping,
    table: &crate::validate::schema::grammar::Ordered<crate::validate::schema::grammar::KeyDef>,
    where_: &str,
    errs: &mut Vec<String>,
) {
    let declared = table.names().collect::<Vec<&str>>().join(", ");
    for (k, v) in obj {
        let Some(key) = k.as_str() else { continue };
        let Some(def) = table.get(key) else {
            errs.push(format!(
                "{where_}: unknown key \"{key}\" (the grammar declares {declared})"
            ));
            continue;
        };
        let got = if v.is_sequence() {
            "list"
        } else if v.is_null() {
            "null"
        } else if v.as_i64().is_some() || v.as_u64().is_some() {
            "integer"
        } else if v.is_mapping() {
            "map"
        } else if v.is_string() {
            "string"
        } else if v.is_bool() {
            "boolean"
        } else {
            "number"
        };
        if !def.r#type.is_empty()
            && got != def.r#type
            && !(def.r#type == "string" && got == "integer")
        {
            errs.push(format!(
                "{where_}.{key}: expected {}, got {got}",
                def.r#type
            ));
            continue;
        }
        if !def.r#enum.is_empty() && !def.r#enum.contains(v) {
            let allowed = def
                .r#enum
                .iter()
                .map(scalar)
                .collect::<Vec<String>>()
                .join(", ");
            errs.push(format!(
                "{where_}.{key}: {} is not one of {allowed}",
                json_like(v)
            ));
        }
        if def.format.as_deref() == Some("regex")
            && let Some(pattern) = v.as_str()
            && let Err(e) = Regex::new(pattern)
        {
            errs.push(format!("{where_}.{key}: not a valid regex — {e}"));
        }
        if let Some(requires) = &def.requires {
            for (rk, rv) in requires.iter() {
                if obj.get(rk).and_then(serde_yaml_ng::Value::as_str) != Some(rv) {
                    errs.push(format!("{where_}.{key}: only allowed with {rk}: {rv}"));
                }
            }
        }
    }
    for (key, def) in table.iter() {
        if def.required && obj.get(key).is_none() {
            errs.push(format!("{where_}: missing required key \"{key}\""));
        }
    }
}

/// A value as `JSON.stringify` renders it, which is what the reference quotes.
fn json_like(v: &serde_yaml_ng::Value) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| scalar(v))
}

/// How many backticks or tildes, worded as the reference words it.
fn marker_size(marker: &str) -> String {
    let n = marker.len();
    let what = if marker.starts_with('`') {
        "backtick"
    } else {
        "tilde"
    };
    let plural = if n == 1 { "" } else { "s" };
    format!("{n} {what}{plural}")
}

/// A schema file: its own SOKF frontmatter, then the fenced contract against
/// the document vocabulary.
pub fn check_schema(file: &Path, text: &str, errs: &mut Vec<String>, g: &Grammar) {
    let _ = file;
    let k = &g.kinds.schema;
    let d = &k.document;
    let lines: Vec<&str> = text.split('\n').collect();
    let Some(split) = split_frontmatter(&lines) else {
        errs.push("missing SOKF frontmatter".to_string());
        return;
    };
    for key in &k.frontmatter.required {
        if !super::read::fm_has(&split.fm, key) {
            errs.push(format!("frontmatter: {key} missing"));
        }
    }
    for key in &k.frontmatter.slug {
        let head = re::compile(&format!("^{key}:")).expect("a key name compiles");
        let slug = re::compile(&format!(r"^{key}:\s*[a-z0-9]+(-[a-z0-9]+)*\s*$"))
            .expect("a key name compiles");
        if let Some(line) = split.fm.iter().find(|l| head.is_match(l))
            && !slug.is_match(line)
        {
            errs.push(format!("frontmatter: {key} is not a slug: \"{line}\""));
        }
    }

    let fences = super::read::extract_yaml(text);
    if fences.len() != d.fences {
        errs.push(format!(
            "expected exactly {} yaml fence, found {}",
            d.fences,
            fences.len()
        ));
        return;
    }
    for fence in &fences {
        if fence.marker != d.fence_marker {
            let why = if fence.marker.len() < d.fence_marker.len() {
                " — the example inside it must be able to carry a fenced block of its own, and a marker this short is closed by the first one"
            } else {
                ""
            };
            errs.push(format!(
                "line {}: the yaml contract opens with {}, expected {}{why}",
                fence.line,
                marker_size(&fence.marker),
                marker_size(&d.fence_marker)
            ));
        }
    }

    let y: serde_yaml_ng::Value = match serde_yaml_ng::from_str(&fences[0].text) {
        Ok(v) => v,
        Err(e) => {
            errs.push(format!("schema yaml: {e}"));
            return;
        }
    };
    let Some(map) = y.as_mapping() else {
        errs.push("schema yaml: not a map".to_string());
        return;
    };

    check_keys(map, &d.keys, "schema yaml", errs);
    let preamble = map.get("preamble");
    let sections = map.get("sections");
    if preamble.is_none() && sections.is_none() {
        errs.push(
            "schema yaml: declares neither preamble nor sections — a governed document is one, the other, or both"
                .to_string(),
        );
    }
    if let Some(p) = preamble
        && let Some(pm) = p.as_mapping()
    {
        check_keys(pm, &d.preamble.keys, "schema yaml: preamble", errs);
    }
    if let Some(list) = sections.and_then(serde_yaml_ng::Value::as_sequence) {
        if list.is_empty() {
            errs.push("schema yaml: no sections entries".to_string());
        }
        for (i, section) in list.iter().enumerate() {
            let where_ = format!("schema yaml: sections[{i}]");
            let Some(sm) = section.as_mapping() else {
                errs.push(format!("{where_}: not a map"));
                continue;
            };
            check_keys(sm, &d.section.keys, &where_, errs);
            let present: Vec<&String> = d
                .section
                .exactly_one_of
                .iter()
                .filter(|k| sm.get(k.as_str()).is_some())
                .collect();
            if present.len() != 1 {
                let found = if present.is_empty() {
                    "neither".to_string()
                } else {
                    present
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>()
                        .join(" and ")
                };
                errs.push(format!(
                    "{where_}: needs exactly one of {}, found {found}",
                    d.section.exactly_one_of.join("/")
                ));
            }
        }
        if !list
            .iter()
            .any(|s| s.get("required") == Some(&serde_yaml_ng::Value::Bool(true)))
        {
            errs.push("schema yaml: no section marked required: true".to_string());
        }
    }
    if let Some(fm) = map
        .get("frontmatter")
        .and_then(serde_yaml_ng::Value::as_mapping)
    {
        for (key, c) in fm {
            let Some(key) = key.as_str() else { continue };
            let where_ = format!("schema yaml: frontmatter.{key}");
            match c.as_mapping() {
                Some(cm) => check_keys(cm, &d.frontmatter_constraint.keys, &where_, errs),
                None => errs.push(format!("{where_}: not a map")),
            }
        }
    }
}

/// One statement a file makes, in the form the duplication check compares.
#[derive(Debug, Clone)]
pub struct Comparable {
    /// The file it came from.
    pub file: String,
    /// The kind of file, which decides what it is compared against.
    pub kind: String,
    /// The element that produced it, for the cross-unit exemption.
    pub element: String,
    /// Where in the file, as the finding names it.
    pub where_: String,
    /// The statement itself.
    pub text: String,
    /// Its tokens, stemmed and stripped of stop words.
    pub tokens: BTreeSet<String>,
}

/// A statement's tokens: lowercased, split on anything not alphanumeric, stop
/// words dropped, and a trailing `s` trimmed off anything over three letters.
fn tokset(text: &str, stop: &[String]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for raw in text
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
    {
        if raw.is_empty() || stop.iter().any(|s| s == raw) {
            continue;
        }
        let t = if raw.len() > 3 && raw.ends_with('s') {
            &raw[..raw.len() - 1]
        } else {
            raw
        };
        out.insert(t.to_string());
    }
    out
}

/// How much of the smaller set the larger one contains, over two sorted token
/// id lists. Sets of strings say the same thing and are what a `Comparable`
/// carries; the pair loop runs this often enough that it walks integers.
fn containment(a: &[u32], b: &[u32]) -> f64 {
    let (mut i, mut j, mut shared) = (0, 0, 0usize);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                shared += 1;
                i += 1;
                j += 1;
            }
        }
    }
    let smaller = a.len().min(b.len());
    if smaller == 0 {
        0.0
    } else {
        shared as f64 / smaller as f64
    }
}

/// Resolve a compare part: `@attr`, `body`, or `@attr|body` — the attribute if
/// it carries anything, else the body.
fn compare_part(part: &str, node: &Node) -> String {
    for alt in part.split('|') {
        let v = if alt == "body" {
            norm(&node.body)
        } else {
            node.attr(&alt[1..]).map(norm).unwrap_or_default()
        };
        if !v.is_empty() {
            return v;
        }
    }
    String::new()
}

/// Fill a compare label: `{@attr}`, `{text}`, each optionally cut to `{x:40}`.
fn fmt_label(template: &str, node: &Node, text: &str) -> String {
    static FIELD: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\{(@?[\w-]+)(?::(\d+))?\}").unwrap());
    FIELD
        .replace_all(template, |c: &regex::Captures| {
            let key = &c[1];
            let v = if key == "text" {
                text.to_string()
            } else if let Some(attr) = key.strip_prefix('@') {
                node.attr(attr).unwrap_or_default().to_string()
            } else {
                String::new()
            };
            c.get(2)
                .and_then(|n| n.as_str().parse::<usize>().ok())
                .map_or(v.clone(), |n| cut(&v, n))
        })
        .to_string()
}

/// What a unit says, one statement per element that declares a `compare`.
#[must_use]
pub fn unit_comparables(file: &str, nodes: &[Node], g: &Grammar) -> Vec<Comparable> {
    let mut items = Vec::new();
    for node in nodes {
        let Some(def) = g.kinds.unit.elements.get(&node.name) else {
            continue;
        };
        let Some(cmp) = &def.compare else { continue };
        if let Some(skip) = &cmp.skip_if {
            let v = compare_part(&skip.part, node);
            if re::matches(&skip.pattern, &v) {
                continue;
            }
        }
        let parts: Vec<String> = cmp
            .parts
            .iter()
            .map(|p| compare_part(p, node))
            .filter(|p| !p.is_empty())
            .collect();
        let text = parts.join(" ");
        if text.is_empty() || g.duplication.skeleton_constants.contains(&text) {
            continue;
        }
        items.push(Comparable {
            file: file.to_string(),
            kind: "unit".to_string(),
            element: node.name.clone(),
            where_: fmt_label(&cmp.label, node, &text),
            tokens: tokset(&text, &g.duplication.stop_words),
            text,
        });
    }
    items
}

/// The core file's statements: every non-heading line outside a fence, with
/// its tags stripped.
#[must_use]
pub fn core_comparables(file: &str, text: &str, g: &Grammar) -> Vec<Comparable> {
    static TAGS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]*>").unwrap());
    let lines: Vec<&str> = text.split('\n').collect();
    let fenced = fence_map(&lines);
    let mut items = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if fenced[i] || line.trim_start().starts_with('#') {
            continue;
        }
        let stripped = TAGS.replace_all(line, "").trim().to_string();
        if stripped.is_empty() {
            continue;
        }
        items.push(Comparable {
            file: file.to_string(),
            kind: "core".to_string(),
            element: "line".to_string(),
            where_: format!("line {}", i + 1),
            tokens: tokset(&stripped, &g.duplication.stop_words),
            text: stripped,
        });
    }
    items
}

/// What a schema says: the prose of each `description` in its contract, up to
/// the example, which is illustration rather than a statement.
#[must_use]
pub fn schema_comparables(file: &str, text: &str, g: &Grammar) -> Vec<Comparable> {
    let fences = super::read::extract_yaml(text);
    if fences.len() != 1 {
        return Vec::new();
    }
    let c = &g.kinds.schema.compare;
    let all: Vec<&str> = fences[0].text.split('\n').collect();
    let stop_at = re::compile(&format!("^{}:", c.stop_at_key)).expect("a key name compiles");
    let lines = match all.iter().position(|l| stop_at.is_match(l)) {
        Some(at) => &all[..at],
        None => &all[..],
    };

    let head = re::compile(&format!(r"^(\s*){}:\s*([>|])?\s*(.*)$", c.description_key))
        .expect("a key name compiles");
    let mut items = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some(m) = head.captures(line) else {
            continue;
        };
        let indent = m[1].len();
        let where_ = format!("yaml {} (line {})", c.description_key, i + 1);
        let inline = m.get(3).map_or("", |x| x.as_str());
        if !inline.is_empty() && m.get(2).is_none() {
            items.push(Comparable {
                file: file.to_string(),
                kind: "schema".to_string(),
                element: "description".to_string(),
                where_,
                tokens: tokset(inline, &g.duplication.stop_words),
                text: inline.to_string(),
            });
            continue;
        }
        let mut buf = Vec::new();
        for next in lines.iter().skip(i + 1) {
            if next.trim().is_empty() {
                break;
            }
            if next.len() - next.trim_start().len() <= indent {
                break;
            }
            buf.push(next.trim());
        }
        if !buf.is_empty() {
            let text = buf.join(" ");
            items.push(Comparable {
                file: file.to_string(),
                kind: "schema".to_string(),
                element: "description".to_string(),
                where_,
                tokens: tokset(&text, &g.duplication.stop_words),
                text,
            });
        }
    }
    items
}

/// Strings the duplication check decides by, numbered so the pair loop can
/// compare them without touching a byte of text.
#[derive(Default)]
struct Interner(HashMap<String, u32>);

impl Interner {
    /// This string's number, assigning one if it has none.
    fn of(&mut self, s: &str) -> u32 {
        if let Some(id) = self.0.get(s) {
            return *id;
        }
        let id = u32::try_from(self.0.len()).unwrap_or(u32::MAX);
        self.0.insert(s.to_string(), id);
        id
    }
}

/// One comparable reduced to what the pair loop reads.
struct Indexed {
    file: u32,
    kind: u32,
    /// Its kind is compared against others in the same file.
    within_file: bool,
    /// Its element is exempt from cross-unit comparison.
    exempt: bool,
    /// Its tokens, numbered and sorted, for the containment walk.
    tokens: Vec<u32>,
}

/// One home per statement. A flagged pair means one occurrence must become a
/// reference to the other's home.
#[must_use]
pub fn check_duplication(items: &[Comparable], g: &Grammar) -> Vec<(String, String)> {
    let d = &g.duplication;
    let mut ids: Interner = Interner::default();
    let mut cross: HashSet<(u32, u32)> = HashSet::new();
    for pair in &d.cross_pairs {
        if let Some((a, b)) = pair.split_once('|') {
            let (a, b) = (ids.of(a), ids.of(b));
            cross.insert((a, b));
            cross.insert((b, a));
        }
    }
    let unit = ids.of("unit");

    // Every pair is considered, so the loop runs in the tens of thousands for
    // a whole-set run and everything it decides by is interned first: the
    // file, the kind and the tokens compare as integers.
    let indexed: Vec<Indexed> = items
        .iter()
        .map(|c| {
            let mut tokens: Vec<u32> = c.tokens.iter().map(|t| ids.of(t)).collect();
            tokens.sort_unstable();
            Indexed {
                file: ids.of(&c.file),
                kind: ids.of(&c.kind),
                within_file: d.within_file_kinds.contains(&c.kind),
                exempt: d.exempt_cross_unit_elements.contains(&c.element),
                tokens,
            }
        })
        .collect();

    let mut out = Vec::new();
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            let (a, b) = (&indexed[i], &indexed[j]);
            if a.file == b.file {
                if !a.within_file {
                    continue;
                }
            } else if !cross.contains(&(a.kind, b.kind)) {
                continue;
            } else if a.kind == unit && b.kind == unit && (a.exempt || b.exempt) {
                // d.exempt_cross_unit_reason
                continue;
            }
            if a.tokens.len().min(b.tokens.len()) < d.min_tokens {
                continue;
            }
            let sim = containment(&a.tokens, &b.tokens);
            if sim >= d.threshold {
                let (a, b) = (&items[i], &items[j]);
                let pct = (sim * 100.0).round() as u64;
                out.push((a.file.clone(), format!(
                    "{pct}% overlap — one occurrence must become a reference: {} ({}): \"{}\" | {} ({}): \"{}\"",
                    a.file,
                    a.where_,
                    cut(&a.text, 90),
                    b.file,
                    b.where_,
                    cut(&b.text, 90)
                )));
            }
        }
    }
    out
}

/// The core file: an H1, balanced block tags, and the block names every other
/// file may refer to.
///
/// Returns the block names it defines, which the reference collects into a
/// process-wide set and the port hands back instead — a validator that has to
/// be run twice in one process should not remember the first run's core.
pub fn check_core(text: &str, errs: &mut Vec<String>, g: &Grammar) -> BTreeSet<String> {
    static BLOCK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"<([a-z][a-z0-9_]*)(?:\s[^>]*)?>").unwrap());

    let k = &g.kinds.core;
    let mut blocks = BTreeSet::new();
    if k.collect_blocks {
        for c in BLOCK.captures_iter(text) {
            blocks.insert(c[1].to_string());
        }
    }

    let lines: Vec<&str> = text.split('\n').collect();
    let fenced = fence_map(&lines);
    if k.require_h1
        && !lines
            .iter()
            .enumerate()
            .any(|(i, l)| !fenced[i] && l.starts_with("# "))
    {
        errs.push("core: missing H1".to_string());
    }
    if !k.balanced_tags {
        return blocks;
    }

    let prose = prose_only(&lines, &fenced);
    let mut stack: Vec<String> = Vec::new();
    for m in TAG.captures_iter(&prose) {
        if m.get(4).is_some() {
            continue;
        }
        let name = m[2].to_string();
        if m.get(1).is_some() {
            if stack.last() != Some(&name) {
                errs.push(format!("core: unbalanced </{name}>"));
                return blocks;
            }
            stack.pop();
        } else {
            stack.push(name);
        }
    }
    if let Some(open) = stack.last() {
        errs.push(format!("core: unclosed <{open}>"));
    }
    blocks
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::validate::schema::parse_grammar;

    pub(super) fn grammar() -> Grammar {
        let path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "../../..",
            ".agents/sokf/grammar.yaml",
        ]
        .iter()
        .collect();
        parse_grammar(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn the_live_core_passes_and_defines_its_blocks() {
        let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "../../..", ".agents/core.md"]
            .iter()
            .collect();
        let text = std::fs::read_to_string(path).unwrap();
        let mut errs = Vec::new();
        let blocks = check_core(&text, &mut errs, &grammar());
        assert!(errs.is_empty(), "{errs:?}");
        assert!(blocks.contains("workflow"), "{blocks:?}");
        assert!(blocks.contains("core_principles"), "{blocks:?}");
    }

    #[test]
    fn a_core_with_no_h1_and_an_unclosed_block_is_reported() {
        let mut errs = Vec::new();
        check_core(
            "<superdev>\n<workflow>\n</superdev>\n",
            &mut errs,
            &grammar(),
        );
        assert_eq!(errs, ["core: missing H1", "core: unbalanced </superdev>"]);
    }

    #[test]
    fn an_unclosed_block_at_the_end_is_reported() {
        let mut errs = Vec::new();
        check_core("# T\n\n<superdev>\n", &mut errs, &grammar());
        assert_eq!(errs, ["core: unclosed <superdev>"]);
    }
}

#[cfg(test)]
mod unit_parity {
    use std::path::PathBuf;

    use super::tests::grammar;
    use super::*;

    fn fixtures() -> PathBuf {
        [env!("CARGO_MANIFEST_DIR"), "tests/fixtures/schema"]
            .iter()
            .collect()
    }

    /// What the reference recorded for `file` in `case`, errors then warnings,
    /// which is the order it pushes them.
    fn golden(case: &str, file: &str) -> Vec<(String, String)> {
        let path = fixtures().join(format!("{case}.golden.json"));
        let g: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        g["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["file"] == file)
            .map(|f| {
                (
                    f["severity"].as_str().unwrap().to_string(),
                    f["message"].as_str().unwrap().to_string(),
                )
            })
            .collect()
    }

    fn ours(case: &str, file: &str) -> Vec<(String, String)> {
        let path = fixtures().join(case).join(file);
        let text = std::fs::read_to_string(&path).unwrap();
        let (mut errs, mut warns) = (Vec::new(), Vec::new());
        check_unit(&path, &text, &mut errs, &mut warns, &grammar());
        errs.into_iter()
            .map(|m| ("error".to_string(), m))
            .chain(warns.into_iter().map(|m| ("warning".to_string(), m)))
            .collect()
    }

    fn parity(case: &str, file: &str) {
        assert_eq!(
            ours(case, file),
            golden(case, file),
            "unit parity for {case}/{file}"
        );
    }

    #[test]
    fn clean() {
        parity("clean", "sd-clean/SKILL.md");
    }

    #[test]
    fn elements() {
        parity("unit-elements", "SKILL.md");
    }

    #[test]
    fn attrs() {
        parity("unit-attrs", "SKILL.md");
    }

    #[test]
    fn structure() {
        parity("unit-structure", "SKILL.md");
    }

    #[test]
    fn frontmatter() {
        parity("unit-frontmatter", "SKILL.md");
    }
}

#[cfg(test)]
mod schema_parity {
    use std::path::PathBuf;

    use super::tests::grammar;
    use super::*;

    fn fixtures() -> PathBuf {
        [env!("CARGO_MANIFEST_DIR"), "tests/fixtures/schema"]
            .iter()
            .collect()
    }

    fn parity(case: &str, file: &str) {
        let path = fixtures().join(case).join(file);
        let mut errs = Vec::new();
        check_schema(
            &path,
            &std::fs::read_to_string(&path).unwrap(),
            &mut errs,
            &grammar(),
        );

        let g: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(fixtures().join(format!("{case}.golden.json"))).unwrap(),
        )
        .unwrap();
        let want: Vec<String> = g["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["file"] == file && f["severity"] == "error")
            .map(|f| f["message"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(errs, want, "schema parity for {case}/{file}");
    }

    #[test]
    fn clean() {
        parity("clean", "schemas/thing.md");
    }

    #[test]
    fn contract() {
        parity("schema-contract", "schemas/thing.md");
    }

    #[test]
    fn sections() {
        parity("schema-sections", "schemas/thing.md");
    }

    /// The 39 schemas that ship all pass, which is the check the live tree
    /// depends on and the widest input the port has.
    #[test]
    fn every_shipped_schema_passes() {
        let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "../../..", "knowledge/schemas"]
            .iter()
            .collect();
        let g = grammar();
        let mut checked = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "md")
                || path.file_name().is_some_and(|n| n == "index.md")
            {
                continue;
            }
            let mut errs = Vec::new();
            check_schema(
                &path,
                &std::fs::read_to_string(&path).unwrap(),
                &mut errs,
                &g,
            );
            assert!(errs.is_empty(), "{}: {errs:?}", path.display());
            checked += 1;
        }
        assert_eq!(checked, 54, "every schema is checked");
    }
}

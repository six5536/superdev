//! validate.rs — the document check of SPEC §10 and the conformance
//! decision of SPEC §11.
//!
//! Began as a behavioural port of the Python reference validator this
//! replaced — the same findings, with the same severities, in the same order —
//! and the divergences that were taken deliberately are noted at the check.
//! Every finding's text is the contract: `tests/validate_snapshots.rs` records
//! the report for one bundle per failure class and compares it verbatim.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use pulldown_cmark::{Event, Options, Parser, Tag};
use serde_yaml_ng::Value;

use super::bundle::Bundle;
use super::concept::Concept;

/// Relationship types the spec defines; anything else reads as `relates-to`.
const CORE_RELS: [&str; 12] = [
    "relates-to",
    "part-of",
    "has-part",
    "depends-on",
    "depended-on-by",
    "references",
    "referenced-by",
    "supersedes",
    "superseded-by",
    "contradicts",
    "implements",
    "implemented-by",
];

const MANIFEST: &str = "manifest.aokf.yaml";

/// One thing the check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Bundle-relative path of the file it was found in.
    pub path: String,
    /// What is wrong.
    pub message: String,
    /// Whether this fails the bundle. `false` for the spec's always-warn
    /// items (SPEC §10 item 5), which report without failing.
    pub fatal: bool,
}

impl Finding {
    /// `"error"` when the finding fails the bundle, otherwise `"warning"`.
    #[must_use]
    pub fn severity(&self) -> &'static str {
        if self.fatal { "error" } else { "warning" }
    }
}

/// The outcome of one validation run.
#[derive(Debug, Clone)]
pub struct Report {
    /// Everything found, in file order.
    pub findings: Vec<Finding>,
    /// Concepts that parsed.
    pub concept_count: usize,
}

impl Report {
    /// Whether the bundle conforms (SPEC §11). There is no partial
    /// conformance: one fatal finding is the whole verdict.
    #[must_use]
    pub fn passed(&self) -> bool {
        !self.findings.iter().any(|f| f.fatal)
    }

    /// The report as JSON.
    ///
    /// The reference validator also emits the bundle path as given on its
    /// command line; that is the caller's string, so the caller adds it.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        let findings: Vec<serde_json::Value> = self
            .findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "severity": f.severity(),
                    "file": f.path,
                    "message": f.message,
                })
            })
            .collect();
        serde_json::json!({
            "concepts": self.concept_count,
            "passed": self.passed(),
            "findings": findings,
        })
    }

    /// The report as CLI text, one finding per line. The caller prints the
    /// bundle path above it.
    #[must_use]
    pub fn render_human(&self) -> String {
        let mut lines = vec![format!("  concepts: {}", self.concept_count)];
        if self.findings.is_empty() {
            lines.push("  ✓ no findings".to_string());
        }
        let mut errors = 0;
        for finding in &self.findings {
            let severity = finding.severity();
            let mark = if severity == "error" {
                errors += 1;
                "✗"
            } else {
                "!"
            };
            lines.push(format!(
                "  {mark} [{severity}] {}: {}",
                finding.path, finding.message
            ));
        }
        let verdict = if errors == 0 { "PASS" } else { "FAIL" };
        let warnings = self.findings.len() - errors;
        lines.push(String::new());
        lines.push(format!(
            "{verdict} ({errors} error(s), {warnings} warning(s))"
        ));
        lines.join("\n") + "\n"
    }
}

/// What every per-concept check needs beyond the concept itself.
struct Context<'a> {
    bundle_root: &'a Path,
    repo_root: &'a Path,
    /// Concept `id` to bundle-relative path, for the ids that are unique and
    /// well-formed.
    ids: &'a HashMap<String, String>,
}

/// Run the document check (SPEC §10) and decide conformance (SPEC §11).
///
/// `repo_root` resolves `/`-rooted targets.
#[must_use]
pub fn validate(bundle: &Bundle, repo_root: &Path) -> Report {
    let mut findings = Vec::new();
    let ids = check_identities(bundle, &mut findings);
    check_manifest(bundle, &mut findings);

    let context = Context {
        bundle_root: &bundle.root,
        repo_root,
        ids: &ids,
    };
    for concept in &bundle.concepts {
        check_concept(concept, &context, &mut findings);
    }
    check_indexes(bundle, repo_root, &mut findings);

    Report {
        findings,
        concept_count: bundle.concepts.len(),
    }
}

/// First pass, in bundle path order: report the files that did not parse and
/// index the ids of the files that did.
fn check_identities(bundle: &Bundle, findings: &mut Vec<Finding>) -> HashMap<String, String> {
    let mut files: Vec<(&str, Result<&Concept, &str>)> = bundle
        .concepts
        .iter()
        .map(|c| (c.path.as_str(), Ok(c)))
        .chain(
            bundle
                .broken
                .iter()
                .map(|e| (e.path.as_str(), Err(e.message.as_str()))),
        )
        .collect();
    files.sort_by_key(|(path, _)| *path);

    let mut ids: HashMap<String, String> = HashMap::new();
    for (path, entry) in files {
        let concept = match entry {
            Ok(concept) => concept,
            Err(message) => {
                findings.push(error(path, message.to_string()));
                continue;
            }
        };
        let id = &concept.raw["id"];
        if id.is_null() {
            continue;
        }
        let id = py_str(id);
        if !is_slug(&id) {
            findings.push(error(
                path,
                format!("`id` is not a valid slug: {}", repr_str(&id)),
            ));
        } else if let Some(first) = ids.get(&id) {
            findings.push(error(
                path,
                format!("duplicate `id` {} (also in {first})", repr_str(&id)),
            ));
        } else {
            ids.insert(id, path.to_string());
        }
    }
    ids
}

/// The manifest parses and carries no stamped keys (document check); it
/// exists and declares `aokf` and `name`.
fn check_manifest(bundle: &Bundle, findings: &mut Vec<Finding>) {
    if let Some(message) = &bundle.manifest_error {
        findings.push(error(MANIFEST, format!("manifest parse error: {message}")));
        return;
    }
    let Some(manifest) = &bundle.manifest else {
        findings.push(error(MANIFEST, "no manifest (required)".to_string()));
        return;
    };
    // A manifest that parses to something other than a mapping has no keys
    // to check; an empty file reads as an empty mapping.
    if !manifest.raw.is_mapping() && !manifest.raw.is_null() {
        return;
    }
    for key in ["producer", "generated", "counts"] {
        if has_key(&manifest.raw, key) {
            findings.push(error(
                MANIFEST,
                format!("stamped key `{key}` present in the working tree"),
            ));
        }
    }
    for key in ["aokf", "name"] {
        if !truthy(&manifest.raw[key]) {
            findings.push(error(MANIFEST, format!("manifest missing `{key}`")));
        }
    }
}

/// Every per-concept rule, in the reference validator's order.
fn check_concept(concept: &Concept, context: &Context, findings: &mut Vec<Finding>) {
    let path = concept.path.as_str();
    let fm = &concept.raw;

    if !truthy(&fm["type"]) || py_str(&fm["type"]).trim().is_empty() {
        findings.push(error(
            path,
            "missing or empty required field `type`".to_string(),
        ));
    }
    if fm["id"].is_null() {
        findings.push(error(path, "no `id` (required)".to_string()));
    }
    if has_key(fm, "generated") {
        findings.push(error(
            path,
            "stamped field `generated` present in the working tree".to_string(),
        ));
    }
    check_verified(path, &fm["verified"], findings);

    let directory = context
        .bundle_root
        .join(path)
        .parent()
        .map_or_else(|| context.bundle_root.to_path_buf(), Path::to_path_buf);
    let (links, footnotes) = markdown_links_and_footnotes(&concept.body);
    let mut body_targets: HashSet<PathBuf> = HashSet::new();
    for target in &links {
        let Some(file) = link_path(target) else {
            continue;
        };
        match resolve_target(file, &directory, context.repo_root) {
            Some(resolved) => {
                body_targets.insert(resolved);
            }
            None => findings.push(warning(path, format!("broken body link: {target}"))),
        }
    }

    if let Some(resource) = fm["resource"].as_str()
        && resource.starts_with('/')
        && !context
            .repo_root
            .join(resource.trim_start_matches('/'))
            .exists()
    {
        findings.push(warning(
            path,
            format!("`resource` path does not exist: {resource}"),
        ));
    }

    let source_ids = check_sources(path, &fm["sources"], context.repo_root, findings);
    for label in footnotes {
        if !source_ids.contains(&label) {
            findings.push(warning(
                path,
                format!("footnote [^{label}] has no matching sources[].id"),
            ));
        }
    }

    check_links(
        path,
        &fm["links"],
        context,
        &directory,
        &body_targets,
        findings,
    );
}

/// `verified` is a mapping or a list of mappings, each with a well-formed
/// actor and an ISO 8601 timestamp (SPEC §7).
fn check_verified(path: &str, value: &Value, findings: &mut Vec<Finding>) {
    if value.is_null() {
        return;
    }
    // A bare mapping reads as a one-element list.
    let entries: Vec<&Value> = if value.is_mapping() {
        vec![value]
    } else if let Some(sequence) = value.as_sequence() {
        sequence.iter().collect()
    } else {
        findings.push(error(
            path,
            "`verified` must be a mapping or a list of mappings".to_string(),
        ));
        return;
    };
    for (index, entry) in entries.iter().enumerate() {
        if !entry.is_mapping() {
            findings.push(error(path, format!("verified[{index}] is not a mapping")));
            continue;
        }
        let by = &entry["by"];
        if !by.as_str().is_some_and(is_actor) {
            findings.push(error(
                path,
                format!(
                    "verified[{index}].by must be `human:<id>` or `process:<id>`, got {}",
                    py_repr(by)
                ),
            ));
        }
        if !is_iso8601(&entry["at"]) {
            findings.push(error(
                path,
                format!(
                    "verified[{index}].at is not ISO 8601: {}",
                    py_repr(&entry["at"])
                ),
            ));
        }
    }
}

/// `sources` entries are mappings carrying a `resource`; returns the ids the
/// body may cite in footnotes.
fn check_sources(
    path: &str,
    value: &Value,
    repo_root: &Path,
    findings: &mut Vec<Finding>,
) -> HashSet<String> {
    let empty = Vec::new();
    let sources = if !truthy(value) {
        &empty
    } else if let Some(sequence) = value.as_sequence() {
        sequence
    } else {
        findings.push(error(path, "`sources` must be a list".to_string()));
        &empty
    };

    let mut ids = HashSet::new();
    for (index, source) in sources.iter().enumerate() {
        if !source.is_mapping() {
            findings.push(error(path, format!("sources[{index}] is not a mapping")));
            continue;
        }
        if !truthy(&source["resource"]) {
            findings.push(error(path, format!("sources[{index}] missing `resource`")));
        }
        if !source["id"].is_null() {
            ids.insert(py_str(&source["id"]));
        }
        if let Some(resource) = source["resource"].as_str()
            && resource.starts_with('/')
            && !repo_root.join(resource.trim_start_matches('/')).exists()
        {
            findings.push(warning(
                path,
                format!("sources[{index}].resource does not exist: {resource}"),
            ));
        }
    }
    ids
}

/// `links` entries carry a `rel` and a `to` (document check); the `to`
/// resolves and a body link mirrors it (level 2).
fn check_links(
    path: &str,
    value: &Value,
    context: &Context,
    directory: &Path,
    body_targets: &HashSet<PathBuf>,
    findings: &mut Vec<Finding>,
) {
    if value.is_null() {
        return;
    }
    let Some(entries) = value.as_sequence() else {
        findings.push(error(path, "`links` must be a list".to_string()));
        return;
    };

    for (index, link) in entries.iter().enumerate() {
        let at = format!("links[{index}]");
        if !link.is_mapping() {
            findings.push(error(path, format!("{at} is not a mapping")));
            continue;
        }
        let rel = &link["rel"];
        if !truthy(rel) {
            findings.push(error(path, format!("{at} missing `rel`")));
        } else if !rel.as_str().is_some_and(is_slug) {
            findings.push(error(
                path,
                format!("{at} `rel` is not lowercase kebab-case: {}", py_repr(rel)),
            ));
        } else if !CORE_RELS.contains(&rel.as_str().unwrap_or_default()) {
            findings.push(warning(
                path,
                format!("{at} non-core rel `{}` (read as relates-to)", py_str(rel)),
            ));
        }

        let to = &link["to"];
        if !truthy(to) {
            findings.push(error(path, format!("{at} missing `to`")));
            continue;
        }
        // An `id` first, then a path (SPEC §8).
        let to = py_str(to);
        let target = match context.ids.get(&to) {
            Some(concept_path) => canonical(&context.bundle_root.join(concept_path)),
            None => resolve_target(&to, directory, context.repo_root),
        };
        let Some(target) = target else {
            findings.push(error(
                path,
                format!("{at} `to: {to}` resolves to no concept id or path"),
            ));
            continue;
        };
        if !body_targets.contains(&target) {
            findings.push(error(
                path,
                format!("{at} `to: {to}` has no mirroring body link"),
            ));
        }
    }
}

/// `index.md` entries point at files that exist.
fn check_indexes(bundle: &Bundle, repo_root: &Path, findings: &mut Vec<Finding>) {
    for (path, text) in &bundle.indexes {
        let directory = bundle
            .root
            .join(path)
            .parent()
            .map_or_else(|| bundle.root.clone(), Path::to_path_buf);
        for target in markdown_links_and_footnotes(text).0 {
            let Some(file) = link_path(&target) else {
                continue;
            };
            if resolve_target(file, &directory, repo_root).is_none() {
                findings.push(warning(
                    path,
                    format!("index entry points at missing file: {target}"),
                ));
            }
        }
    }
}

fn error(path: &str, message: String) -> Finding {
    Finding {
        path: path.to_string(),
        message,
        fatal: true,
    }
}

fn warning(path: &str, message: String) -> Finding {
    Finding {
        path: path.to_string(),
        message,
        fatal: false,
    }
}

/// Every link destination in document order, and every footnote label.
///
/// The reference validator matches inline links with a regular expression;
/// reading them from the markdown parser instead drops the ones inside code
/// fences and picks up reference-style links.
fn markdown_links_and_footnotes(text: &str) -> (Vec<String>, BTreeSet<String>) {
    let mut links = Vec::new();
    let mut labels = BTreeSet::new();
    for event in Parser::new_ext(
        text,
        Options::ENABLE_FOOTNOTES | Options::ENABLE_OLD_FOOTNOTES,
    ) {
        match event {
            Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => {
                links.push(dest_url.to_string());
            }
            Event::Start(Tag::FootnoteDefinition(label)) | Event::FootnoteReference(label) => {
                labels.insert(label.to_string());
            }
            _ => {}
        }
    }
    (links, labels)
}

/// The file part of a link target, or `None` when it names no file.
fn link_path(target: &str) -> Option<&str> {
    let path = target.split('#').next().unwrap_or_default();
    (!path.is_empty()
        && !path.starts_with("http://")
        && !path.starts_with("https://")
        && !path.starts_with("mailto:"))
    .then_some(path)
}

/// Resolve a link or `to` path to an existing file (SPEC §9): `/` from the
/// repository root, anything else from the linking file's directory.
fn resolve_target(target: &str, directory: &Path, repo_root: &Path) -> Option<PathBuf> {
    let file = link_path(target)?;
    let path = match file.strip_prefix('/') {
        Some(rooted) => repo_root.join(rooted.trim_start_matches('/')),
        None => directory.join(file),
    };
    canonical(&path)
}

/// The canonical path, or `None` when nothing is there.
fn canonical(path: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(path).ok()
}

/// Whether `text` matches the spec's slug form, `[a-z0-9]+(-[a-z0-9]+)*`.
fn is_slug(text: &str) -> bool {
    text.split('-').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    })
}

/// Whether `text` is a `human:` or `process:` actor (SPEC §7).
fn is_actor(text: &str) -> bool {
    ["human:", "process:"]
        .iter()
        .any(|prefix| text.strip_prefix(prefix).is_some_and(|id| !id.is_empty()))
}

/// Whether the value is an ISO 8601 datetime.
///
/// Covers the extended forms `datetime.fromisoformat` accepts: a calendar
/// date, an optional time, and an optional `Z` or `±HH[:MM[:SS]]` offset.
/// Basic (unpunctuated) and week-date forms are rejected — no AOKF bundle
/// writes them, and accepting them costs more than it is worth.
fn is_iso8601(value: &Value) -> bool {
    let Some(text) = value.as_str() else {
        return false;
    };
    if !text.is_ascii() || text.len() < 10 {
        return false;
    }
    let (date, rest) = text.split_at(10);
    if !is_date(date) {
        return false;
    }
    if rest.is_empty() {
        return true;
    }
    if !rest.starts_with(['T', 't', ' ']) {
        return false;
    }
    let (time, offset) = split_offset(&rest[1..]);
    is_time(time) && offset.is_none_or(is_offset)
}

fn is_date(text: &str) -> bool {
    let [year, month, day] = text.split('-').collect::<Vec<_>>()[..] else {
        return false;
    };
    let (Some(year), Some(month), Some(day)) = (number(year, 4), number(month, 2), number(day, 2))
    else {
        return false;
    };
    (1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month)
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn is_time(text: &str) -> bool {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    let fields: Vec<&str> = whole.split(':').collect();
    if fields.len() > 3
        || (!fraction.is_empty()
            && (fields.len() != 3 || number(fraction, fraction.len()).is_none()))
    {
        return false;
    }
    let (Some(hour), Some(minute), Some(second)) = (
        number(fields[0], 2),
        fields.get(1).map_or(Some(0), |f| number(f, 2)),
        fields.get(2).map_or(Some(0), |f| number(f, 2)),
    ) else {
        return false;
    };
    hour < 24 && minute < 60 && second < 60
}

fn is_offset(text: &str) -> bool {
    matches!(text, "Z" | "z") || text.strip_prefix(['+', '-']).is_some_and(is_time)
}

/// Split a time from its trailing offset, if it has one.
fn split_offset(text: &str) -> (&str, Option<&str>) {
    match text.rfind(['Z', 'z', '+', '-']) {
        Some(at) => (&text[..at], Some(&text[at..])),
        None => (text, None),
    }
}

/// `text` as a number, when it is exactly `width` digits.
fn number(text: &str, width: usize) -> Option<u32> {
    (text.len() == width && text.bytes().all(|b| b.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}

/// Whether a YAML value is truthy the way Python is: absent, null, false,
/// zero, and empty strings and collections are not.
fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Sequence(items) => !items.is_empty(),
        Value::Mapping(map) => !map.is_empty(),
        Value::Tagged(tagged) => truthy(&tagged.value),
    }
}

/// Whether a mapping carries `key` at all, however it is valued — a stamped
/// field set to null is still present.
fn has_key(value: &Value, key: &str) -> bool {
    value
        .as_mapping()
        .is_some_and(|map| map.contains_key(Value::String(key.to_string())))
}

/// A value as Python's `str` would render it, so findings read the same as
/// the reference validator's.
fn py_str(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => py_repr(other),
    }
}

/// A value as Python's `repr` would render it.
fn py_repr(value: &Value) -> String {
    match value {
        Value::Null => "None".to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => repr_str(s),
        Value::Sequence(items) => format!("[{}]", join(items.iter().map(py_repr))),
        Value::Mapping(map) => format!(
            "{{{}}}",
            join(
                map.iter()
                    .map(|(k, v)| format!("{}: {}", py_repr(k), py_repr(v)))
            )
        ),
        Value::Tagged(tagged) => py_repr(&tagged.value),
    }
}

/// A string as Python's `repr` would quote it: single quotes, unless that
/// would need escaping and double quotes would not.
fn repr_str(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    if escaped.contains('\'') && !escaped.contains('"') {
        format!("\"{escaped}\"")
    } else {
        format!("'{}'", escaped.replace('\'', "\\'"))
    }
}

fn join(parts: impl Iterator<Item = String>) -> String {
    parts.collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aokf::load_bundle;

    fn bundle_with(files: &[(&str, &str)]) -> (Bundle, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        for (p, t) in files {
            let path = dir.path().join(p);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, t).unwrap();
        }
        (load_bundle(dir.path()).unwrap(), dir)
    }

    const MANIFEST_YAML: &str = "aokf: \"0.1\"\nname: t\n";
    const A_MIRRORED: &str = "---\ntype: T\nid: alpha\nlinks:\n  - rel: depends-on\n    to: beta\n---\nSee [beta](beta.md).\n";
    const B: &str = "---\ntype: T\nid: beta\n---\nx\n";

    #[test]
    fn clean_bundle_passes() {
        let (b, _dir) = bundle_with(&[
            ("manifest.aokf.yaml", MANIFEST_YAML),
            ("a.md", A_MIRRORED),
            ("beta.md", B),
        ]);
        let r = validate(&b, &b.root);
        assert!(r.passed(), "{:?}", r.findings);
        assert!(r.passed());
        assert_eq!(r.concept_count, 2);
        assert!(r.render_human().contains("no findings"));
        assert_eq!(r.to_json()["passed"], serde_json::json!(true));
    }

    #[test]
    fn implements_is_core_and_warns_nothing() {
        let (b, _dir) = bundle_with(&[
            ("manifest.aokf.yaml", MANIFEST_YAML),
            (
                "plan.md",
                "---\ntype: Plan\nid: alpha\nlinks:\n  - rel: implements\n    to: beta\n---\nSee [beta](spec.md).\n",
            ),
            (
                "spec.md",
                "---\ntype: Spec\nid: beta\nlinks:\n  - rel: implemented-by\n    to: alpha\n---\nSee [alpha](plan.md).\n",
            ),
        ]);
        let r = validate(&b, &b.root);
        assert!(r.passed(), "{:?}", r.findings);
        assert!(r.findings.is_empty(), "{:?}", r.findings);
    }

    #[test]
    fn duplicate_ids_fail() {
        let (b, _dir) = bundle_with(&[
            ("a.md", "---\ntype: T\nid: dup\n---\nx\n"),
            ("b.md", "---\ntype: T\nid: dup\n---\nx\n"),
        ]);
        let r = validate(&b, &b.root);
        assert!(!r.passed());
        assert!(r.findings.iter().any(|f| f.message.contains("dup")));
    }

    #[test]
    fn unmirrored_link_fails() {
        let files = [
            ("manifest.aokf.yaml", MANIFEST_YAML),
            (
                "a.md",
                "---\ntype: T\nid: alpha\nlinks:\n  - rel: depends-on\n    to: beta\n---\nno body link\n",
            ),
            ("beta.md", B),
        ];
        let (b, _dir) = bundle_with(&files);
        let r = validate(&b, &b.root);
        assert!(!r.passed());
        assert!(r.findings[0].message.contains("no mirroring body link"));
    }

    /// One bundle breaking every rule at once, so the finding list — order,
    /// wording and level — is pinned against the reference validator, which
    /// reports exactly this for the same files.
    #[test]
    fn every_check_fires_on_a_deliberately_broken_bundle() {
        let broken_a = "---\ntype: \"\"\nid: Bad_Slug\ngenerated: {by: agent}\nresource: /nowhere.rs\nverified: {by: nobody, at: yesterday}\nsources:\n  - id: ok-src\n    resource: /beta.md\n  - title: no resource\n  - \"not a mapping\"\n  - resource: /missing.rs\n    id: gone-src\nlinks:\n  - to: beta\n  - rel: Bad Rel\n    to: beta\n  - rel: made-up\n    to: beta\n  - rel: depends-on\n  - rel: depends-on\n    to: nowhere-at-all\n  - rel: depends-on\n    to: beta\n  - \"not a mapping\"\n---\nCites [^ok-src] and [^unknown].\n\nA [broken](nope.md) link and a good [beta](beta.md).\n\n[^ok-src]: Beta\n[^unknown]: nothing\n";
        let (b, _dir) = bundle_with(&[
            (
                "manifest.aokf.yaml",
                "aokf: \"0.1\"\ncounts: 3\ngenerated: {by: x}\n",
            ),
            (
                "index.md",
                "* [beta](beta.md)\n* [gone](missing.md)\n* [ext](https://example.com)\n",
            ),
            ("a.md", broken_a),
            ("beta.md", B),
            (
                "dup.md",
                "---\ntype: T\nid: beta\nverified:\n  - by: human:rsewell\n    at: 2026-08-04T09:00:00Z\n  - by: process:x\n    at: \"2026-13-45\"\n  - \"not a mapping\"\nlinks: nope\n---\nx\n",
            ),
            (
                "other.md",
                "---\ntype: T\nverified: 3\nsources: nope\n---\nx\n",
            ),
            ("nofm.md", "no frontmatter\n"),
        ]);
        let r = validate(&b, &b.root);
        let found: Vec<String> = r
            .findings
            .iter()
            .map(|f| format!("{}|{}|{}", f.severity(), f.path, f.message))
            .collect();
        assert_eq!(
            found,
            [
                "error|a.md|`id` is not a valid slug: 'Bad_Slug'",
                "error|dup.md|duplicate `id` 'beta' (also in beta.md)",
                "error|nofm.md|no frontmatter: expected a `---` line, then a closing `---`",
                "error|manifest.aokf.yaml|stamped key `generated` present in the working tree",
                "error|manifest.aokf.yaml|stamped key `counts` present in the working tree",
                "error|manifest.aokf.yaml|manifest missing `name`",
                "error|a.md|missing or empty required field `type`",
                "error|a.md|stamped field `generated` present in the working tree",
                "error|a.md|verified[0].by must be `human:<id>` or `process:<id>`, got 'nobody'",
                "error|a.md|verified[0].at is not ISO 8601: 'yesterday'",
                "warning|a.md|broken body link: nope.md",
                "warning|a.md|`resource` path does not exist: /nowhere.rs",
                "error|a.md|sources[1] missing `resource`",
                "error|a.md|sources[2] is not a mapping",
                "warning|a.md|sources[3].resource does not exist: /missing.rs",
                "warning|a.md|footnote [^unknown] has no matching sources[].id",
                "error|a.md|links[0] missing `rel`",
                "error|a.md|links[1] `rel` is not lowercase kebab-case: 'Bad Rel'",
                "warning|a.md|links[2] non-core rel `made-up` (read as relates-to)",
                "error|a.md|links[3] missing `to`",
                "error|a.md|links[4] `to: nowhere-at-all` resolves to no concept id or path",
                "error|a.md|links[6] is not a mapping",
                "error|dup.md|verified[1].at is not ISO 8601: '2026-13-45'",
                "error|dup.md|verified[2] is not a mapping",
                "error|dup.md|`links` must be a list",
                "error|other.md|no `id` (required)",
                "error|other.md|`verified` must be a mapping or a list of mappings",
                "error|other.md|`sources` must be a list",
                "warning|index.md|index entry points at missing file: missing.md",
            ]
        );
        assert!(!r.passed());
        assert!(
            r.render_human()
                .ends_with("FAIL (23 error(s), 6 warning(s))\n")
        );
        assert_eq!(
            r.to_json()["findings"][0]["file"],
            serde_json::json!("a.md")
        );
    }

    #[test]
    fn a_manifest_that_is_not_a_mapping_or_will_not_parse() {
        let (b, _dir) = bundle_with(&[("manifest.aokf.yaml", "- one\n")]);
        assert!(validate(&b, &b.root).findings.is_empty());

        let (b, _dir) = bundle_with(&[("manifest.aokf.yaml", "aokf: [unclosed\n")]);
        let r = validate(&b, &b.root);
        assert_eq!(r.findings.len(), 1);
        assert!(r.findings[0].message.starts_with("manifest parse error:"));
        assert!(!r.passed());
    }

    #[test]
    fn iso_8601_accepts_the_forms_a_bundle_writes() {
        let ok = |s: &str| is_iso8601(&Value::String(s.to_string()));
        for good in [
            "2026-08-04",
            "2026-08-04T09:00:00Z",
            "2026-08-04 09:00",
            "2024-02-29T23:59:59.123456+01:00",
            "2024-12-31t00:00:00-05:30",
        ] {
            assert!(ok(good), "{good}");
        }
        for bad in [
            "yesterday",
            "2026-13-01",
            "2026-02-30",
            "2025-02-29",
            "20260804",
            "2026-08-04X09:00",
            "2026-08-04T24:00",
            "2026-08-04T09:60",
            "2026-08-04T09:00:00.12x",
            "2026-08-04T09:00:00+ab:00",
            "2026-08-04T09:00:00:00:00",
            "2026-8-04",
            "日本語です日本語です",
        ] {
            assert!(!ok(bad), "{bad}");
        }
        assert!(!is_iso8601(&Value::Number(2026.into())));
    }

    #[test]
    fn values_render_the_way_python_does() {
        let value: Value = serde_yaml_ng::from_str("[1, true, null, {a: b}, \"it's\"]").unwrap();
        assert_eq!(py_repr(&value), "[1, True, None, {'a': 'b'}, \"it's\"]");
        assert_eq!(py_str(&Value::String("plain".into())), "plain");
        assert!(!truthy(&Value::Number(0.into())));
        assert!(truthy(&Value::Number(1.into())));
    }
}

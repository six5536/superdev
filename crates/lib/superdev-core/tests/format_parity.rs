//! format_parity.rs — `format::check_files` against the Node reference, one
//! fixture tree per failure class.
//!
//! Each `tests/fixtures/format/<case>/` holds the files, and
//! `<case>.golden.json` holds what the reference emitted for them under
//! `--json`. The reference — a Node script at `scripts/superdev-format/` — is
//! gone; the goldens are the only remaining record of the behaviour it
//! defined, and there is nothing left to regenerate them from. They were
//! captured before a line of the port existed, for exactly that reason: the
//! AOKF port had already been in this position once.
//!
//! Treat a golden as fixed. Editing one changes the contract these tests exist
//! to hold, so it needs the same argument a deliberate behaviour change would.
//!
//! Every message compares verbatim, and so does the order: the two
//! implementations walk files and checks the same way, so neither side is
//! sorted here. One normalisation, and nothing else:
//!
//! - A `schema yaml:` finding quoting a YAML parse error compares on its
//!   prefix only. Both sides quote their own parser, and the two word the same
//!   complaint differently — the reference uses the `yaml` package, the port
//!   uses `serde_yaml_ng`. No fixture exercises it today; the normalisation is
//!   here so that adding one later reports the parse, not the wording.

use std::path::{Path, PathBuf};

use superdev_core::format::{Grammar, check_files, parse_grammar};

/// The fixture root.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/format")
}

/// The grammar as it ships.
fn grammar() -> Grammar {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(".agents/format/grammar.yaml");
    parse_grammar(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Every `.md` under `dir`, named relative to it and sorted, which is how the
/// goldens were captured.
fn files(dir: &Path) -> Vec<(String, String)> {
    fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "md") {
                // The full path, because the checks resolve it to find the
                // skill's directory — the reference does `dirname(resolve())`.
                // The parity comparison strips the root back off.
                let name = path.to_string_lossy().replace('\\', "/");
                out.push((name, std::fs::read_to_string(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, &mut out);
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// A parse-error message reduced to its prefix. See the header.
fn normalise(message: &str) -> String {
    if message.starts_with("schema yaml: ")
        && !message.starts_with("schema yaml: unknown key")
        && !message.starts_with("schema yaml: missing")
        && !message.starts_with("schema yaml: sections")
        && !message.starts_with("schema yaml: no ")
        && !message.starts_with("schema yaml: declares")
        && !message.starts_with("schema yaml: not a map")
        && !message.starts_with("schema yaml: frontmatter")
        && !message.starts_with("schema yaml: preamble")
    {
        return "schema yaml: <parse error>".to_string();
    }
    message.to_string()
}

fn parity(case: &str) {
    let golden: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixtures().join(format!("{case}.golden.json"))).unwrap(),
    )
    .unwrap();

    let dir = fixtures().join(case);
    let ours = check_files(&files(&dir), &grammar());
    // The goldens name files relative to the case directory, because that is
    // where the reference was run from.
    let prefix = format!("{}/", dir.to_string_lossy().replace('\\', "/"));
    let mine: Vec<(String, String, String)> = ours
        .iter()
        .map(|f| {
            (
                if f.fatal { "error" } else { "warning" }.to_string(),
                f.file.strip_prefix(&prefix).unwrap_or(&f.file).to_string(),
                // The duplication finding names both files inside its own
                // message, so the prefix comes off there too.
                normalise(&f.message.replace(&prefix, "")),
            )
        })
        .collect();
    let theirs: Vec<(String, String, String)> = golden["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["severity"].as_str().unwrap().to_string(),
                f["file"].as_str().unwrap().to_string(),
                normalise(f["message"].as_str().unwrap()),
            )
        })
        .collect();

    assert_eq!(mine, theirs, "parity failure for {case}");
    assert_eq!(
        ours.iter().all(|f| !f.fatal),
        golden["passed"].as_bool().unwrap(),
        "verdict for {case}"
    );
}

#[test]
fn clean() {
    parity("clean");
}

#[test]
fn unit_elements() {
    parity("unit-elements");
}

#[test]
fn unit_attrs() {
    parity("unit-attrs");
}

#[test]
fn unit_structure() {
    parity("unit-structure");
}

#[test]
fn unit_frontmatter() {
    parity("unit-frontmatter");
}

#[test]
fn schema_contract() {
    parity("schema-contract");
}

#[test]
fn schema_sections() {
    parity("schema-sections");
}

#[test]
fn core_blocks() {
    parity("core-blocks");
}

#[test]
fn core_block_reference() {
    parity("core-block-reference");
}

#[test]
fn duplication() {
    parity("duplication");
}

/// The live tree passes, which is the check CI gates on and the widest input
/// the port has: one core file, 39 schemas and 21 skills.
#[test]
fn the_live_tree_passes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let g = grammar();
    let mut inputs = Vec::new();
    for rel in &g.roots.paths {
        let dir = root.join(rel);
        if !dir.is_dir() {
            continue;
        }
        for (name, text) in files(&dir) {
            if superdev_core::format::detect_kind(Path::new(&name), &g, false).is_some() {
                inputs.push((name, text));
            }
        }
    }
    assert_eq!(inputs.len(), 61, "the roots hold 61 claimed files");
    let findings = check_files(&inputs, &g);
    // Warnings are expected and do not fail a run: five skills carry frontmatter
    // keys Claude Code reads but the portable Agent Skills spec does not.
    let fatal: Vec<&superdev_core::format::Finding> = findings.iter().filter(|f| f.fatal).collect();
    assert!(fatal.is_empty(), "{fatal:#?}");
    assert_eq!(
        findings.len(),
        5,
        "the five portability warnings, and nothing else"
    );
}

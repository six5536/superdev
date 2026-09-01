//! schema_snapshots.rs — `validate::schema::check_files` over one fixture
//! tree per failure class, compared to a recorded report.
//!
//! Each `tests/fixtures/schema/<case>/` holds the files and
//! `<case>.golden.json` holds what the checks report for them. The goldens
//! began as captures from the Node script this code replaced, taken before a
//! line of the port existed. That script is no longer the authority — these
//! checks are — and a golden is simply their recorded output.
//!
//! What they still are is the contract over the finding texts, their
//! severities, the verdict and the order findings arrive in. The inline tests
//! in `check.rs` pin behaviour; these pin wording.
//!
//! Regenerate with:
//!
//! ```sh
//! UPDATE_GOLDENS=1 cargo test -p superdev-core --test schema_snapshots
//! ```
//!
//! Then read the diff. A reworded message shows up as a wording diff; a moved
//! severity, a changed verdict or a finding that appears or vanishes is a
//! behaviour change, and wants the argument any behaviour change wants.
//!
//! A golden names its files relative to the case directory. The checks are
//! given absolute paths — they resolve a skill's directory from its file's —
//! so the prefix comes off before the report is written or compared, which is
//! what keeps a golden the same on every machine.

use std::path::{Path, PathBuf};

use superdev_core::validate::schema::{Grammar, check_files, parse_grammar};

/// The fixture root.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/schema")
}

/// The grammar as it ships.
fn grammar() -> Grammar {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(".agents/sokf/grammar.yaml");
    parse_grammar(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Every `.md` under `dir`, sorted, named by its full path because the checks
/// resolve a skill's directory from it.
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

/// Check `case` and compare the report to the golden, or rewrite the golden
/// when `UPDATE_GOLDENS` is set.
fn snapshot(case: &str) {
    let dir = fixtures().join(case);
    let inputs = files(&dir);
    let findings = check_files(&inputs, &grammar());

    // The prefix every path carries, off again so the golden is machine
    // independent. The duplication finding names both files inside its own
    // message, so it comes off there too.
    let prefix = format!("{}/", dir.to_string_lossy().replace('\\', "/"));
    let ours = serde_json::json!({
        "files": inputs.len(),
        "passed": findings.iter().all(|f| !f.fatal),
        "findings": findings
            .iter()
            .map(|f| serde_json::json!({
                "severity": if f.fatal { "error" } else { "warning" },
                "file": f.file.strip_prefix(&prefix).unwrap_or(&f.file),
                "message": f.message.replace(&prefix, ""),
            }))
            .collect::<Vec<_>>(),
    });

    let path = fixtures().join(format!("{case}.golden.json"));
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        let mut text = serde_json::to_string_pretty(&ours).unwrap();
        text.push('\n');
        std::fs::write(&path, text).unwrap();
        return;
    }

    let golden: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(ours, golden, "{case} does not match its golden");
}

#[test]
fn clean() {
    snapshot("clean");
}

#[test]
fn unit_elements() {
    snapshot("unit-elements");
}

#[test]
fn unit_attrs() {
    snapshot("unit-attrs");
}

#[test]
fn unit_structure() {
    snapshot("unit-structure");
}

#[test]
fn unit_frontmatter() {
    snapshot("unit-frontmatter");
}

#[test]
fn schema_contract() {
    snapshot("schema-contract");
}

#[test]
fn schema_sections() {
    snapshot("schema-sections");
}

#[test]
fn core_blocks() {
    snapshot("core-blocks");
}

#[test]
fn core_block_reference() {
    snapshot("core-block-reference");
}

#[test]
fn duplication() {
    snapshot("duplication");
}

/// The live tree passes, which is the check CI gates on and the widest input
/// these checks have: one core file, 55 schemas and 23 skills.
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
            if superdev_core::validate::schema::detect_kind(Path::new(&name), &g, false).is_some() {
                inputs.push((name, text));
            }
        }
    }
    assert_eq!(inputs.len(), 79, "the roots hold 79 claimed files");
    let findings = check_files(&inputs, &g);
    // Warnings are expected and do not fail a run: five skills carry frontmatter
    // keys Claude Code reads but the portable Agent Skills spec does not.
    let fatal: Vec<&superdev_core::validate::schema::Finding> =
        findings.iter().filter(|f| f.fatal).collect();
    assert!(fatal.is_empty(), "{fatal:#?}");
    assert_eq!(
        findings.len(),
        5,
        "the five portability warnings, and nothing else"
    );
}

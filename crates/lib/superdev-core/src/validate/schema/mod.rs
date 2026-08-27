//! validate::schema — the schema half of the validator.
//!
//! Documents are governed by schemas, and skills and the core file by the
//! grammar those schemas are themselves written in. Both live here because
//! both are driven from one grammar file carried as data rather than from
//! code, which is what lets a repository superdev writes into change the
//! language without changing the binary.
//!
//! The other half, [`super::sokf`], checks the SOKF knowledge against the
//! specification. Neither calls the other; [`super`] runs both and reports
//! once.

pub mod check;
pub mod doc;
pub mod grammar;
pub mod re;
pub mod read;

use std::collections::BTreeSet;
use std::path::Path;

pub use grammar::Grammar;

use crate::error::{Error, Result};

/// Where a repository keeps the grammar it is checked against.
pub const GRAMMAR_PATH: &str = ".agents/sokf/grammar.yaml";

/// The grammar as it ships, so a repository without its own copy still
/// validates. It is a copy of the repository's `.agents/sokf/grammar.yaml`
/// — `include_str!` cannot reach outside the crate and still be packaged —
/// and a test asserts the two are byte for byte the same.
pub const EMBEDDED_GRAMMAR: &str = include_str!("grammar.yaml");

/// Read a grammar from YAML.
///
/// # Errors
/// Returns the deserialisation error, which names the offending key: the
/// types are `deny_unknown_fields`, so a typo in the grammar fails here
/// rather than silently switching a rule off.
pub fn parse_grammar(yaml: &str) -> std::result::Result<Grammar, serde_yaml_ng::Error> {
    serde_yaml_ng::from_str(yaml)
}

/// The grammar `root` is checked against: its own copy when it has one, and
/// the embedded copy otherwise.
///
/// # Errors
/// The file is unreadable, or it does not deserialise — which happens before
/// any file is read, and names the key at fault.
pub fn load_grammar(root: &Path) -> Result<Grammar> {
    let path = root.join(GRAMMAR_PATH);
    let yaml = match std::fs::read_to_string(&path) {
        Ok(yaml) => yaml,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => EMBEDDED_GRAMMAR.to_string(),
        Err(source) => return Err(Error::Io { path, source }),
    };
    parse_grammar(&yaml).map_err(|e| Error::Toml {
        path,
        message: e.to_string(),
    })
}

/// What a file is checked as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A skill or prompt, written in the element vocabulary.
    Unit,
    /// The structural contract of one produced artifact.
    Schema,
    /// The repository's one core file.
    Core,
}

impl Kind {
    /// The name the reference prints for this kind.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Schema => "schema",
            Self::Core => "core",
        }
    }
}

/// Which kind claims `path`.
///
/// `Some(kind)` when a kind claims it, `None` when one claims the name but
/// excepts it — a directory index sits beside the concepts it lists — and,
/// when `by_fallback` is false, `None` again when nothing claims it
/// positively. A bare run walks the grammar's roots with `by_fallback` off, so
/// an unrelated markdown file beside a claimed one is passed over rather than
/// read as the default kind.
#[must_use]
pub fn detect_kind(path: &Path, g: &Grammar, by_fallback: bool) -> Option<Kind> {
    let base = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let parent = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|s| s.to_str())
        .unwrap_or_default();

    let kinds = [
        (Kind::Unit, &g.kinds.unit.matches),
        (Kind::Schema, &g.kinds.schema.matches),
        (Kind::Core, &g.kinds.core.matches),
    ];
    let mut fallback = None;
    for (kind, m) in kinds {
        let claims = m.basename.iter().any(|b| b == base)
            || m.suffix.iter().any(|s| base.ends_with(s))
            || m.dir.iter().any(|d| d == parent);
        if claims {
            return if m.except.iter().any(|e| e == base) {
                None
            } else {
                Some(kind)
            };
        }
        if m.default {
            fallback = Some(kind);
        }
    }
    if by_fallback { fallback } else { None }
}

/// One thing the schema half found. Distinct from [`super::Finding`], which
/// is what a whole run reports: this one names the file as the caller gave
/// it, before the paths of the two halves are reconciled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The file it was found in, as the caller named it.
    pub file: String,
    /// What is wrong.
    pub message: String,
    /// Whether it fails the run.
    pub fatal: bool,
}

/// Check a set of files against the grammar.
///
/// Findings come back in a fixed order: every file's errors then its
/// warnings, in the order the files were given, then the core-block
/// references, then duplication. Order is part of the contract — the snapshot
/// goldens compare the list as emitted.
#[must_use]
pub fn check_files(files: &[(String, String)], g: &Grammar) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut comparables = Vec::new();
    let mut core_blocks: BTreeSet<String> = BTreeSet::new();
    let mut kinds: Vec<(String, Kind, &str)> = Vec::new();

    for (name, text) in files {
        let Some(kind) = detect_kind(Path::new(name), g, true) else {
            continue;
        };
        let (mut errs, mut warns) = (Vec::new(), Vec::new());
        match kind {
            Kind::Unit => {
                if let Some(nodes) =
                    check::check_unit(Path::new(name), text, &mut errs, &mut warns, g)
                {
                    comparables.extend(check::unit_comparables(name, &nodes, g));
                }
            }
            Kind::Schema => {
                check::check_schema(Path::new(name), text, &mut errs, g);
                comparables.extend(check::schema_comparables(name, text, g));
            }
            Kind::Core => {
                core_blocks.extend(check::check_core(text, &mut errs, g));
                comparables.extend(check::core_comparables(name, text, g));
            }
        }
        kinds.push((name.clone(), kind, text.as_str()));
        for message in errs {
            findings.push(Finding {
                file: name.clone(),
                message,
                fatal: true,
            });
        }
        for message in warns {
            findings.push(Finding {
                file: name.clone(),
                message,
                fatal: false,
            });
        }
    }

    // A "core <x> block" reference must resolve to a block the core defines.
    // It runs only when a core file was among the inputs: without one there is
    // nothing to resolve against, and every reference would be a false finding.
    let refs = &g.kinds.unit.checks.core_block_references;
    if !core_blocks.is_empty()
        && refs.enabled
        && let Some(pattern) = re::compile(&refs.pattern)
    {
        for (name, kind, text) in &kinds {
            if !refs.applies_to.iter().any(|k| k == kind.as_str()) {
                continue;
            }
            for c in pattern.captures_iter(text) {
                if !core_blocks.contains(&c[1]) {
                    findings.push(Finding {
                        file: name.clone(),
                        message: format!("{}: <{}>", refs.message, &c[1]),
                        fatal: true,
                    });
                }
            }
        }
    }

    for (file, message) in check::check_duplication(&comparables, g) {
        findings.push(Finding {
            file,
            message,
            fatal: true,
        });
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The repository this crate lives in, which carries the grammar the
    /// embedded copy must match.
    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap()
    }

    #[test]
    fn the_embedded_grammar_equals_the_repository_copy() {
        let file = std::fs::read_to_string(repo().join(GRAMMAR_PATH)).unwrap();
        assert_eq!(
            EMBEDDED_GRAMMAR, file,
            "copy {GRAMMAR_PATH} to crates/lib/superdev-core/src/format/grammar.yaml"
        );
    }

    #[test]
    fn a_repository_without_the_grammar_validates_from_the_embedded_copy() {
        let empty = tempfile::tempdir().unwrap();
        assert_eq!(load_grammar(empty.path()).unwrap().grammar, "superdev");
    }

    #[test]
    fn an_unreadable_grammar_fails_naming_the_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(GRAMMAR_PATH);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, format!("{EMBEDDED_GRAMMAR}\ninvented-key: no\n")).unwrap();
        let message = load_grammar(dir.path()).unwrap_err().to_string();
        assert!(message.contains("invented-key"), "{message}");
    }
}

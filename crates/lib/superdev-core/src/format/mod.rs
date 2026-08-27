//! format — the superdev-format checks: skills, schemas and the core file,
//! validated against the grammar that defines the language they are written
//! in.
//!
//! The AOKF side of the validator ([`crate::aokf`]) checks the knowledge
//! bundle against the AOKF spec. This side checks a wider set — the bundle's
//! schemas, but also `.claude/skills/` and `.agents/` — against a grammar
//! carried as data. One command runs both and reports once.

pub mod check;
pub mod grammar;
pub mod read;

use std::collections::BTreeSet;
use std::path::Path;

pub use grammar::Grammar;
use regex::Regex;

/// Read a grammar from YAML.
///
/// # Errors
/// Returns the deserialisation error, which names the offending key: the
/// types are `deny_unknown_fields`, so a typo in the grammar fails here
/// rather than silently switching a rule off.
pub fn parse_grammar(yaml: &str) -> Result<Grammar, serde_yaml_ng::Error> {
    serde_yaml_ng::from_str(yaml)
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

/// One thing the format check found.
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
/// Findings come back in the reference's order: every file's errors then its
/// warnings, in the order the files were given, then the core-block
/// references, then duplication. Order is part of the contract — the parity
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
        && let Ok(pattern) = Regex::new(&refs.pattern)
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

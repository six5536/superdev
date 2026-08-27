//! format — the superdev-format checks: skills, schemas and the core file,
//! validated against the grammar that defines the language they are written
//! in.
//!
//! The AOKF side of the validator ([`crate::aokf`]) checks the knowledge
//! bundle against the AOKF spec. This side checks a wider set — the bundle's
//! schemas, but also `.claude/skills/` and `.agents/` — against a grammar
//! carried as data. One command runs both and reports once.

pub mod check;
pub mod doc;
pub mod grammar;
pub mod re;
pub mod read;

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

pub use grammar::Grammar;

use crate::aokf::validate::Report;
use crate::aokf::{self, load_bundle};
use crate::error::{Error, Result};

/// Where a repository keeps the grammar it is checked against.
pub const GRAMMAR_PATH: &str = ".agents/format/grammar.yaml";

/// The grammar as it ships, so a repository without its own copy still
/// validates. It is a copy of the repository's `.agents/format/grammar.yaml`
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

/// One run's report, and what each half of it covered.
///
/// The concept count rides in [`Report`], where the AOKF half puts it. The
/// format half's count rides here instead: [`Report`] is what
/// `aokf::validate` returns, and it knows nothing about format files. Putting
/// the count there would push a format concern into the AOKF type, which is
/// the boundary D-18 exists to hold.
#[derive(Debug)]
pub struct RepoReport {
    /// The findings from both halves, grouped by file.
    pub report: Report,
    /// Format files read, which the caller emits as `files` — the key the
    /// Node reference put at the top of its own JSON. Zero and clean is a run
    /// that found nothing to check, indistinguishable from a pass unless the
    /// number is shown.
    pub files: usize,
}

/// Validate a repository — its AOKF bundle and its superdev-format files — as
/// one report.
///
/// With `paths` empty the run covers the whole repository: the bundle, and
/// every tree the grammar's `roots` names. A non-empty `paths` replaces both.
/// The bundle is then validated only when one of the given paths is the
/// bundle or contains it, so naming one skill checks that skill and nothing
/// else. Findings are grouped by file, so a file both checks have something to
/// say about is reported once.
///
/// `aokf::validate` is called, not changed: its findings arrive exactly as it
/// emits them, with their bundle-relative paths respelt against the repository
/// root so the two halves name the same file the same way.
///
/// # Errors
/// The bundle is unreadable, or a file named on the command line is.
pub fn validate_repo(
    repo_root: &Path,
    bundle: &Path,
    paths: &[PathBuf],
    g: &Grammar,
) -> Result<RepoReport> {
    let repo_root = normalise(&std::env::current_dir().unwrap_or_default(), repo_root);
    let bundle = normalise(&repo_root, bundle);
    let paths: Vec<PathBuf> = paths.iter().map(|p| normalise(&repo_root, p)).collect();

    let mut findings = Vec::new();
    let mut concept_count = 0;

    // Named explicitly, so an unreadable bundle is an error rather than a
    // silent skip; unnamed, so a repository without one is simply a repository
    // whose format files are still worth checking.
    let named = paths.iter().any(|p| bundle.starts_with(p));
    if named || (paths.is_empty() && bundle.is_dir()) {
        let report = aokf::validate(&load_bundle(&bundle)?, &repo_root);
        concept_count = report.concept_count;
        let prefix = relative(&repo_root, &bundle);
        findings.extend(report.findings.into_iter().map(|f| aokf::Finding {
            path: if prefix.is_empty() {
                f.path
            } else {
                format!("{prefix}/{}", f.path)
            },
            message: f.message,
            fatal: f.fatal,
        }));
    }

    let mut files = Vec::new();
    if paths.is_empty() {
        for root in &g.roots.paths {
            let dir = repo_root.join(root);
            if dir.is_dir() {
                collect(&repo_root, &dir, g, &mut files)?;
            }
        }
    } else {
        for path in &paths {
            if path.is_dir() {
                collect(&repo_root, path, g, &mut files)?;
            } else if detect_kind(path, g, true).is_some() {
                // A file named on the command line is checked whatever it is
                // called, as the reference does: the fallback kind applies
                // where the walk would have passed it over.
                files.push((relative(&repo_root, path), read(path)?));
            }
        }
    }

    findings.extend(check_files(&files, g).into_iter().map(|f| aokf::Finding {
        path: f.file,
        message: f.message,
        fatal: f.fatal,
    }));
    // Stable, so each half keeps the order it emitted while the two interleave
    // by file.
    findings.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(RepoReport {
        report: Report {
            findings,
            concept_count,
        },
        files: files.len(),
    })
}

/// Every claimed file under `dir`, named relative to `repo_root` and sorted,
/// which is the order the reference walks a root in.
fn collect(
    repo_root: &Path,
    dir: &Path,
    g: &Grammar,
    files: &mut Vec<(String, String)>,
) -> Result<()> {
    let mut found = Vec::new();
    walk(dir, g, &mut found)?;
    found.sort();
    for path in found {
        let text = read(&path)?;
        files.push((relative(repo_root, &path), text));
    }
    Ok(())
}

/// The claimed files under `dir`, unsorted. Only a kind that claims a file by
/// name, suffix or directory takes it: the fallback would read every stray
/// markdown file beside a skill as a malformed unit.
fn walk(dir: &Path, g: &Grammar, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(|source| Error::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let path = entry
            .map_err(|source| Error::Io {
                path: dir.to_path_buf(),
                source,
            })?
            .path();
        if path.is_dir() {
            walk(&path, g, out)?;
        } else if detect_kind(&path, g, false).is_some() {
            out.push(path);
        }
    }
    Ok(())
}

fn read(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// `path` under `root`, forward-slashed, or its whole spelling when it is not
/// under `root` at all.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// `path` made absolute against `base`, with `.` components dropped so that
/// `superdev validate .` compares against the bundle the way a reader expects.
/// `..` is left alone: resolving it needs the filesystem, and a path that
/// keeps one simply fails to match rather than matching the wrong thing.
fn normalise(base: &Path, path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    joined
        .components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The repository this crate lives in: a real bundle beside real roots,
    /// which is the input the merged command was designed around.
    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap()
    }

    fn live() -> Grammar {
        load_grammar(&repo()).unwrap()
    }

    /// The two copies of the grammar are one file. D-6 keeps a copy inside the
    /// binary so a repository without `.agents/format/grammar.yaml` still
    /// validates; this is what stops the two drifting.
    #[test]
    fn the_embedded_grammar_equals_the_repository_copy() {
        let file = std::fs::read_to_string(repo().join(GRAMMAR_PATH)).unwrap();
        assert_eq!(
            EMBEDDED_GRAMMAR, file,
            "copy {GRAMMAR_PATH} to crates/lib/superdev-core/src/format/grammar.yaml"
        );
    }

    /// With no file to read, the embedded copy is used.
    #[test]
    fn a_repository_without_the_grammar_validates_from_the_embedded_copy() {
        let empty = tempfile::tempdir().unwrap();
        assert_eq!(load_grammar(empty.path()).unwrap().grammar, "superdev");
    }

    /// A grammar that violates its own constraints fails before any file is
    /// read, naming the key at fault.
    #[test]
    fn an_unreadable_grammar_fails_naming_the_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(GRAMMAR_PATH);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, format!("{EMBEDDED_GRAMMAR}\ninvented-key: no\n")).unwrap();
        let message = load_grammar(dir.path()).unwrap_err().to_string();
        assert!(message.contains("invented-key"), "{message}");
    }

    /// A bare run covers the bundle and every root, names every file the same
    /// way, and reports each file once.
    #[test]
    fn a_bare_run_covers_the_bundle_and_the_roots() {
        let root = repo();
        let run = validate_repo(&root, &root.join("knowledge"), &[], &live()).unwrap();

        assert!(run.report.concept_count > 0, "the bundle was validated");
        assert!(run.files > 0, "the roots were walked");
        assert!(run.report.passed(), "{:#?}", run.report.findings);
        // The five portability warnings, from files under .claude/skills —
        // which is a root the AOKF half never walks.
        assert_eq!(run.report.findings.len(), 5);
        assert!(
            run.report
                .findings
                .iter()
                .all(|f| f.path.starts_with(".claude/skills/")),
            "{:#?}",
            run.report.findings
        );

        let paths: Vec<&str> = run
            .report
            .findings
            .iter()
            .map(|f| f.path.as_str())
            .collect();
        let mut sorted = paths.clone();
        sorted.sort_unstable();
        assert_eq!(paths, sorted, "findings are grouped by file");
    }

    /// A positional path replaces both defaults: the bundle is not loaded, and
    /// only what was named is read.
    #[test]
    fn a_named_path_replaces_the_bundle_and_the_roots() {
        let root = repo();
        let skills = vec![PathBuf::from(".claude/skills")];
        let run = validate_repo(&root, &root.join("knowledge"), &skills, &live()).unwrap();
        assert_eq!(run.report.concept_count, 0, "no bundle was covered");
        assert_eq!(run.report.findings.len(), 5);
    }

    /// Naming the bundle covers it, and `.` covers everything.
    #[test]
    fn naming_the_bundle_or_the_repository_covers_it() {
        let root = repo();
        let bundle = root.join("knowledge");
        for path in [PathBuf::from("knowledge"), PathBuf::from(".")] {
            let run = validate_repo(&root, &bundle, std::slice::from_ref(&path), &live()).unwrap();
            assert!(
                run.report.concept_count > 0,
                "`{}` covers the bundle",
                path.display()
            );
        }
    }

    /// One file is checked on its own, whatever it is called: the fallback
    /// kind applies to a file named on the command line, where the walk would
    /// have passed it over.
    #[test]
    fn one_named_file_is_checked_alone() {
        let root = repo();
        let one = vec![PathBuf::from(".agents/core.md")];
        let run = validate_repo(&root, &root.join("knowledge"), &one, &live()).unwrap();
        assert_eq!(run.report.concept_count, 0);
        assert_eq!(run.files, 1);
        assert!(run.report.passed(), "{:#?}", run.report.findings);
    }

    /// A repository the roots find nothing in reports zero files rather than
    /// the clean pass it would otherwise be indistinguishable from.
    #[test]
    fn a_repository_with_no_format_files_reports_none() {
        let dir = tempfile::tempdir().unwrap();
        let run = validate_repo(dir.path(), &dir.path().join("knowledge"), &[], &live()).unwrap();
        assert_eq!(run.files, 0);
        assert_eq!(run.report.concept_count, 0);
        assert!(run.report.passed(), "{:#?}", run.report.findings);
    }

    /// A path that names nothing is a caller error, not a finding: the run
    /// stops rather than reporting a file it never read.
    #[test]
    fn a_path_that_names_nothing_fails_the_run() {
        let root = repo();
        let missing = vec![PathBuf::from("no/such/SKILL.md")];
        let error = validate_repo(&root, &root.join("knowledge"), &missing, &live()).unwrap_err();
        assert!(error.to_string().contains("no/such/SKILL.md"));
    }

    /// A format finding fails the run without touching what AOKF reported.
    #[test]
    fn a_format_error_fails_the_run_beside_the_aokf_findings() {
        let root = repo();
        let bundle = root.join("knowledge");
        let g = live();
        let clean = validate_repo(&root, &bundle, &[], &g).unwrap();

        let broken = tempfile::tempdir().unwrap();
        let skill = broken.path().join(".claude/skills/broken");
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(skill.join("SKILL.md"), "no frontmatter here\n").unwrap();
        let both = validate_repo(
            &root,
            &bundle,
            &[bundle.clone(), skill.join("SKILL.md")],
            &g,
        )
        .unwrap();

        assert!(!both.report.passed());
        assert_eq!(both.report.concept_count, clean.report.concept_count);
        let aokf: Vec<&aokf::Finding> = both
            .report
            .findings
            .iter()
            .filter(|f| f.path.starts_with("knowledge/"))
            .collect();
        assert!(aokf.is_empty(), "{aokf:#?}");
    }

    #[test]
    fn a_path_outside_the_repository_keeps_its_own_spelling() {
        assert_eq!(relative(Path::new("/a"), Path::new("/b/c.md")), "/b/c.md");
    }
}

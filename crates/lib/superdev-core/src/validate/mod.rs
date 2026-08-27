//! validate — the whole check over a repository, as one report.
//!
//! Two halves, and this module is where they meet. [`sokf`] checks the SOKF
//! knowledge against the specification: frontmatter, ids, links, footnotes.
//! [`schema`] checks documents against the schemas that govern them, and the
//! skills and core file against the grammar they are written in.
//!
//! Neither half calls the other. The merge is here, above both, so the
//! boundary D-18 draws is a module boundary rather than a promise in a
//! comment — and so one command, one report and one exit code cover a
//! repository, which is what stops the hook and the merge gate reaching
//! different verdicts about the same tree (D-17).

pub mod schema;
pub mod sokf;

use std::path::{Component, Path, PathBuf};

pub use schema::Grammar;
pub use sokf::{Finding, Report, validate};

use crate::error::{Error, Result};
use crate::sokf::load_bundle;

/// One run's report, and what each half of it covered.
///
/// The concept count rides in [`Report`], where the SOKF half puts it. The
/// schema half's count rides here instead: [`Report`] is what [`sokf::validate`]
/// returns, and it knows nothing about the files the grammar governs. Putting
/// the count there would push a schema concern into the SOKF type, which is
/// the boundary D-18 exists to hold.
#[derive(Debug)]
pub struct RepoReport {
    /// The findings from both halves, grouped by file.
    pub report: Report,
    /// Files the grammar governs, read this run, which the caller emits as
    /// `files`. Zero and clean is a run that found nothing to check,
    /// indistinguishable from a pass unless the number is shown.
    pub files: usize,
}

/// Validate a repository — its SOKF knowledge and the files the grammar
/// governs — as one report.
///
/// With `paths` empty the run covers the whole repository: the bundle, and
/// every tree the grammar's `roots` names. A non-empty `paths` replaces both.
/// The bundle is then validated only when one of the given paths is the
/// bundle or contains it, so naming one skill checks that skill and nothing
/// else. Findings are grouped by file, so a file both checks have something to
/// say about is reported once.
///
/// Each half is called, not changed: its findings arrive exactly as it emits
/// them, with the SOKF half's knowledge-relative paths respelt against the
/// repository root so the two name the same file the same way.
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
    let mut documents: Vec<(String, String, Option<String>)> = Vec::new();

    // Named explicitly, so unreadable knowledge is an error rather than a
    // silent skip; unnamed, so a repository without any is simply a repository
    // whose governed files are still worth checking.
    let named = paths.iter().any(|p| bundle.starts_with(p));
    if named || (paths.is_empty() && bundle.is_dir()) {
        let knowledge = load_bundle(&bundle)?;
        let prefix = relative(&repo_root, &bundle);
        let spell = |path: &str| {
            if prefix.is_empty() {
                path.to_string()
            } else {
                format!("{prefix}/{path}")
            }
        };
        // The candidates for schema checking: every concept the knowledge
        // holds. Schemas are excluded — they answer to the grammar, not to
        // each other — and so are indexes, which SPEC §9 governs.
        for concept in &knowledge.concepts {
            let path = spell(&concept.path);
            if path.contains("/schemas/") || path.ends_with("/index.md") || path == "index.md" {
                continue;
            }
            documents.push((
                path,
                read(&bundle.join(&concept.path))?,
                Some(concept.kind.clone()),
            ));
        }
        let report = sokf::validate(&knowledge, &repo_root);
        concept_count = report.concept_count;
        findings.extend(report.findings.into_iter().map(|f| Finding {
            path: spell(&f.path),
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
            } else if schema::detect_kind(path, g, true).is_some() {
                // A file named on the command line is checked whatever it is
                // called, as the reference does: the fallback kind applies
                // where the walk would have passed it over.
                files.push((relative(&repo_root, path), read(path)?));
            }
        }
    }

    findings.extend(schema::check_files(&files, g).into_iter().map(|f| Finding {
        path: f.file,
        message: f.message,
        fatal: f.fatal,
    }));

    // Documents against the schemas that govern them. The schemas come from
    // the files just walked — `knowledge/schemas` is one of the grammar's
    // roots — so a repository whose schemas were not read simply checks no
    // documents, rather than reporting every one of them as ungoverned.
    let schema_files: Vec<(String, String)> = files
        .iter()
        .filter(|(name, _)| name.contains("/schemas/") && !name.ends_with("/index.md"))
        .cloned()
        .collect();
    if !schema_files.is_empty() {
        // Two documents carry no frontmatter and are named by glob instead.
        for name in ["README.md", "CHANGELOG.md"] {
            let path = repo_root.join(name);
            if path.is_file() {
                documents.push((name.to_string(), read(&path)?, None));
            }
        }
        let (set, mut schema_findings) = schema::document::SchemaSet::load(&schema_files);
        let candidates: Vec<schema::document::Document<'_>> = documents
            .iter()
            .map(|(path, text, doc_type)| schema::document::Document {
                path,
                text,
                doc_type: doc_type.as_deref(),
            })
            .collect();
        schema_findings.extend(schema::document::check_documents(&candidates, &set));
        findings.extend(schema_findings.into_iter().map(|f| Finding {
            path: f.file,
            message: f.message,
            fatal: f.fatal,
        }));
    }
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
/// which fixes the order findings arrive in.
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
        } else if schema::detect_kind(&path, g, false).is_some() {
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

    /// The repository this crate lives in: real SOKF knowledge beside real
    /// roots, which is the input the merged command was designed around.
    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap()
    }

    fn live() -> Grammar {
        schema::load_grammar(&repo()).unwrap()
    }

    /// The two copies of the grammar are one file. D-6 keeps a copy inside the
    /// binary so a repository without `.agents/sokf/grammar.yaml` still
    /// validates; this is what stops the two drifting.
    /// With no file to read, the embedded copy is used.
    /// A grammar that violates its own constraints fails before any file is
    /// read, naming the key at fault.
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
    fn a_repository_with_no_governed_files_reports_none() {
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
    fn a_schema_error_fails_the_run_beside_the_sokf_findings() {
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
        let sokf: Vec<&Finding> = both
            .report
            .findings
            .iter()
            .filter(|f| f.path.starts_with("knowledge/"))
            .collect();
        assert!(sokf.is_empty(), "{sokf:#?}");
    }

    #[test]
    fn a_path_outside_the_repository_keeps_its_own_spelling() {
        assert_eq!(relative(Path::new("/a"), Path::new("/b/c.md")), "/b/c.md");
    }
}

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

pub mod fix;
pub mod lifecycle;
pub mod schema;
pub mod sokf;

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

pub use fix::Repair;
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
    /// Files the grammar governs that the paths cover, which the caller
    /// emits as `files`. Zero and clean is a run that found nothing to
    /// check, indistinguishable from a pass unless the number is shown.
    pub files: usize,
    /// Schemas read, and documents checked against one. Both for the same
    /// reason `files` exists: a repository with no `knowledge/schemas/`
    /// checks no document against any contract, and without the numbers that
    /// run is indistinguishable from one where every document conformed.
    pub schemas: usize,
    /// Documents resolved to a schema and checked.
    pub documents: usize,
}

/// Validate a repository — its SOKF knowledge and the files the grammar
/// governs — as one report.
///
/// With `paths` empty the run covers the whole repository: the bundle, and
/// every tree the grammar's `roots` names. A non-empty `paths` replaces both
/// for what is reported: the run is the bare run with its report scoped to
/// the files the paths cover, so a named document gets exactly the findings
/// a bare run gives it (ADR-026). A named file the bare pipeline never
/// reaches is checked as what it is: the grammar kind that claims it, the
/// schema its `type` or a glob dispatches it to, or the fallback kind when
/// nothing claims it. The bundle is validated as a whole only when one of
/// the given paths is the bundle or contains it. Findings are grouped by
/// file, so a file both checks have something to say about is reported once.
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
    // What the paths cover, spelt the way findings are; empty covers all.
    let scopes: Vec<String> = paths.iter().map(|p| relative(&repo_root, p)).collect();

    let mut findings = Vec::new();
    let mut concept_count = 0;
    let mut documents: Vec<(String, String, Option<String>)> = Vec::new();
    let mut subjects: Vec<lifecycle::Subject> = Vec::new();

    // Named explicitly, so unreadable knowledge is an error rather than a
    // silent skip. A named run loads the knowledge too — a named document
    // gets bare-run parity only with the candidates and the links in hand
    // (ADR-026) — but the knowledge is reported as a whole only when
    // `covered`; the coverage filter below drops the rest. A run whose paths
    // never touch the bundle drops every knowledge finding, so a fault
    // reading the knowledge is not its error either: the context is skipped
    // rather than failing the run about a file nobody named.
    let named = paths.iter().any(|p| bundle.starts_with(p));
    let covered = named || paths.is_empty();
    let touches = covered || paths.iter().any(|p| p.starts_with(&bundle));
    let prefix = relative(&repo_root, &bundle);
    if named || bundle.is_dir() {
        let knowledge = match load_bundle(&bundle) {
            Ok(knowledge) => Some(knowledge),
            Err(_) if !touches => None,
            Err(e) => return Err(e),
        };
        let spell = |path: &str| {
            if prefix.is_empty() {
                path.to_string()
            } else {
                format!("{prefix}/{path}")
            }
        };
        if let Some(knowledge) = knowledge {
            // The candidates for schema checking: every concept the knowledge
            // holds. Schemas are excluded by their kind — they answer to the
            // grammar, not to each other — and so are indexes, which SPEC §9
            // governs. A fragment is a document like any other: the grammar's
            // parent-directory match never claims it.
            for concept in &knowledge.concepts {
                if concept.kind == "Schema"
                    || concept.path.ends_with("/index.md")
                    || concept.path == "index.md"
                {
                    continue;
                }
                let text = match read(&bundle.join(&concept.path)) {
                    Ok(text) => text,
                    Err(_) if !touches => continue,
                    Err(e) => return Err(e),
                };
                let path = spell(&concept.path);
                subjects.push(lifecycle::Subject {
                    path: path.clone(),
                    doc_type: concept.kind.clone(),
                    lifecycle: concept.lifecycle.clone(),
                });
                documents.push((path, text, Some(concept.kind.clone())));
            }
            let report = sokf::validate(&knowledge, &repo_root);
            if covered {
                concept_count = report.concept_count;
            }
            findings.extend(report.findings.into_iter().map(|f| Finding {
                path: spell(&f.path),
                message: f.message,
                fatal: f.fatal,
            }));
        }
    }

    // The schema set, from the resolved knowledge's own schemas directory —
    // one source for a bare and a named run, so the two cannot disagree
    // about which schemas govern (ADR-026). A repository with no such
    // directory checks no documents, rather than reporting every one of
    // them as ungoverned. Loaded ahead of the loop below, because dispatch
    // decides what a named file is before the grammar sees it.
    let schema_dir = bundle.join("schemas");
    let mut schema_files: Vec<(String, String)> = Vec::new();
    if schema_dir.is_dir() {
        collect(&repo_root, &schema_dir, g, &mut schema_files)?;
    }
    let (set, mut schema_findings) = schema::document::SchemaSet::load(&schema_files);

    // The grammar's roots are walked whatever the paths say: a named run is
    // the bare run with its report scoped, which is what makes parity a
    // filter rather than a promise (ADR-026). The named loop adds only what
    // the bare pipeline never reaches.
    let mut files = Vec::new();
    for root in &g.roots.paths {
        let dir = repo_root.join(root);
        if dir.is_dir() {
            collect(&repo_root, &dir, g, &mut files)?;
        }
    }
    for path in &paths {
        if path.is_dir() {
            collect(&repo_root, path, g, &mut files)?;
            continue;
        }
        let name = relative(&repo_root, path);
        // A named path that cannot be read is reported the way every finding
        // spells a path — repository-relative and forward-slashed — rather
        // than as the absolute, platform-separated spelling the reader failed
        // on. The two differ on Windows, where the caller would otherwise be
        // handed back a path they did not type (I041).
        let text = read(path).map_err(|e| match e {
            Error::Io { source, .. } => Error::Io {
                path: PathBuf::from(&name),
                source,
            },
            other => other,
        })?;
        // A file the bare pipeline already treats keeps that treatment: the
        // walk brought it in, the knowledge holds it — a concept is already
        // a candidate, and a file under the bundle that is no concept gets
        // the SOKF half's say — or the glob-named pair below takes it with
        // the bare run's spelling.
        if files.iter().any(|(n, _)| *n == name)
            || covers(std::slice::from_ref(&prefix), &name)
            || name == "README.md"
            || name == "CHANGELOG.md"
        {
            continue;
        }
        // A file the bare run never reaches. A grammar kind that claims it
        // by name keeps it; a frontmatter `type` or a schema's glob makes
        // it a document candidate — never the grammar's fallback kind
        // (ADR-026), though dispatch means nothing with no schema set to
        // resolve against; the fallback takes a file nothing claims, which
        // keeps a skill outside the roots checkable.
        let doc_type = frontmatter_type(&text);
        if schema::detect_kind(path, g, false).is_some() {
            files.push((name, text));
        } else if !schema_files.is_empty() && (doc_type.is_some() || set.governs(&name, None)) {
            if !documents.iter().any(|(p, ..)| *p == name) {
                documents.push((name, text, doc_type));
            }
        } else if schema::detect_kind(path, g, true).is_some() {
            files.push((name, text));
        }
    }
    // One check per file, however many arguments reach it: a named
    // directory overlapping a root, or a file inside a named directory,
    // adds nothing.
    let mut seen = BTreeSet::new();
    files.retain(|(name, _)| seen.insert(name.clone()));

    findings.extend(schema::check_files(&files, g).into_iter().map(|f| Finding {
        path: f.file,
        message: f.message,
        fatal: f.fatal,
    }));

    // Documents against the schemas that govern them.
    let mut schemas = 0;
    let mut checked = 0;
    if !schema_files.is_empty() {
        // Two documents carry no frontmatter and are named by glob instead —
        // with no `type` to dispatch on, whatever frontmatter they carry.
        // Skipped when the paths do not cover them: their findings would be
        // dropped, so an unreadable one is not this run's error.
        for name in ["README.md", "CHANGELOG.md"] {
            let path = repo_root.join(name);
            if covers(&scopes, name) && path.is_file() && !documents.iter().any(|(p, ..)| p == name)
            {
                documents.push((name.to_string(), read(&path)?, None));
            }
        }
        schemas = schema_files.len();
        // Each schema's example against the schema declaring it, in place —
        // findings land on the schema file (ADR-024).
        schema_findings.extend(schema::document::check_examples(&schema_files));
        let candidates: Vec<schema::document::Document<'_>> = documents
            .iter()
            .map(|(path, text, doc_type)| schema::document::Document {
                path,
                text,
                doc_type: doc_type.as_deref(),
            })
            .collect();
        checked = candidates
            .iter()
            .filter(|d| set.governs(d.path, d.doc_type) && covers(&scopes, d.path))
            .count();
        schema_findings.extend(schema::document::check_documents(&candidates, &set));
        // The filing check: a document's `lifecycle` against the folder
        // carrying it, in scope wherever its schema declares the enum.
        findings.extend(lifecycle::check(&subjects, &set));
        findings.extend(schema_findings.into_iter().map(|f| Finding {
            path: f.file,
            message: f.message,
            fatal: f.fatal,
        }));
    }
    // Findings name only what the paths cover (ADR-026): the run reads the
    // whole knowledge for parity, and reports the files it was asked about.
    findings.retain(|f| covers(&scopes, &f.path));
    // Stable, so each half keeps the order it emitted while the two interleave
    // by file.
    findings.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(RepoReport {
        report: Report {
            findings,
            concept_count,
        },
        files: files
            .iter()
            .filter(|(name, _)| covers(&scopes, name))
            .count(),
        schemas,
        documents: checked,
    })
}

/// Repair the SOKF knowledge's links in place, ahead of validating it.
///
/// The repairs are [`fix`]'s; this is where the run resolves the same paths
/// [`validate_repo`] resolves, so `--fix` and the check that follows it read
/// one knowledge directory — and cover it on the same condition the check
/// reports knowledge findings on: a path that is the knowledge, contains it,
/// or names a file inside it. Naming a skill on the command line checks that
/// skill and repairs nothing.
///
/// # Errors
/// The knowledge is unreadable, or a document cannot be written.
pub fn fix_repo(repo_root: &Path, bundle: &Path, paths: &[PathBuf]) -> Result<Repair> {
    let repo_root = normalise(&std::env::current_dir().unwrap_or_default(), repo_root);
    let bundle = normalise(&repo_root, bundle);
    let paths: Vec<PathBuf> = paths.iter().map(|p| normalise(&repo_root, p)).collect();
    let covered = paths.is_empty()
        || paths
            .iter()
            .any(|p| bundle.starts_with(p) || p.starts_with(&bundle));
    if !covered || !bundle.is_dir() {
        return Ok(Repair::default());
    }
    let knowledge = load_bundle(&bundle)?;
    let prefix = relative(&repo_root, &bundle);
    let mut repair = fix::fix(&knowledge, &repo_root)?;
    if !prefix.is_empty() {
        for path in &mut repair.written {
            // A move entry is `from -> to`; both sides get the prefix.
            *path = path
                .split(" -> ")
                .map(|p| format!("{prefix}/{p}"))
                .collect::<Vec<_>>()
                .join(" -> ");
        }
    }
    Ok(repair)
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

/// Whether the named scopes cover `file` — it is one of them, or under one.
/// An empty list is a bare run and covers everything, as does a scope that
/// names the repository root itself.
fn covers(scopes: &[String], file: &str) -> bool {
    scopes.is_empty()
        || scopes.iter().any(|s| {
            s.is_empty()
                || file == s
                || file
                    .strip_prefix(s.as_str())
                    .is_some_and(|rest| rest.starts_with('/'))
        })
}

/// The frontmatter `type` of `text`, when the file opens with one — the
/// dispatch key a file named on the command line joins the candidates with.
fn frontmatter_type(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let split = schema::read::split_frontmatter(&lines)?;
    schema::read::fm_value(&split.fm, "type")
}

/// `text` split into lines, each without its terminator, so a CRLF document
/// reads as its LF twin: the same frontmatter, the same fences, the same
/// headings, the same generated block.
///
/// superdev governs repositories whose checkout settings it does not own, and
/// git hands a Windows checkout CRLF for every path `.gitattributes` does not
/// pin. A line is the same line either way, so the checks compare lines with
/// the terminator already gone rather than every reader remembering to
/// normalise first — which is the trap that left the validator reporting a
/// Windows checkout as ungoverned (I040).
///
/// Unlike [`str::lines`] the empty final element a trailing newline produces
/// is kept, because the checks index by line number and report on it.
#[must_use]
pub(crate) fn lines(text: &str) -> Vec<&str> {
    text.split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect()
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

/// `path` made absolute against `base`, with `.` components dropped and `..`
/// resolved against the component before it, so `superdev validate .` and a
/// `knowledge/../knowledge` spelling both compare equal to the paths the run
/// resolves. The resolution is lexical: a `..` with nothing before it to
/// consume is kept as written.
fn normalise(base: &Path, path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    let mut out = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(out.components().next_back(), Some(Component::Normal(_))) {
                    out.pop();
                } else {
                    out.push(component);
                }
            }
            component => out.push(component),
        }
    }
    out
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

    /// A bare run covers the knowledge and every root, names every file the
    /// same way, and reports each file once.
    #[test]
    fn a_bare_run_covers_the_bundle_and_the_roots() {
        let root = repo();
        let run = validate_repo(&root, &root.join("knowledge"), &[], &live()).unwrap();

        assert!(run.report.concept_count > 0, "the bundle was validated");
        assert!(run.files > 0, "the roots were walked");
        assert!(run.report.passed(), "{:#?}", run.report.findings);
        // The five portability warnings, from files under .claude/skills —
        // which is a root the SOKF half never walks.
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

    /// A positional path replaces both defaults for what is reported: the
    /// run still reads everything for parity, and the report names only
    /// what the path covers (ADR-026).
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

    /// A named schema file keeps its grammar kind: the schema kind claims it
    /// positively, so it is checked as a schema and never as a document
    /// candidate (I019 criterion 1).
    #[test]
    fn one_named_file_is_checked_alone() {
        let root = repo();
        let one = vec![PathBuf::from("knowledge/schemas/adr.md")];
        let run = validate_repo(&root, &root.join("knowledge"), &one, &live()).unwrap();
        assert_eq!(run.report.concept_count, 0);
        assert_eq!(run.files, 1, "the schema kind claimed the file");
        assert_eq!(run.documents, 0, "never a document candidate");
        assert!(run.report.passed(), "{:#?}", run.report.findings);
    }

    /// A named file with no frontmatter that no glob and no grammar kind
    /// claims takes the fallback kind, so a skill outside the roots stays
    /// checkable (I019 criterion 4).
    #[test]
    fn a_named_file_nothing_claims_takes_the_fallback_kind() {
        let dir = parity_repo();
        std::fs::write(dir.path().join("notes.md"), "no frontmatter here\n").unwrap();
        let one = vec![PathBuf::from("notes.md")];
        let run = validate_repo(dir.path(), &dir.path().join("knowledge"), &one, &live()).unwrap();
        assert_eq!(run.files, 1, "the fallback kind claimed the file");
        assert!(
            run.report
                .findings
                .iter()
                .any(|f| f.path == "notes.md" && f.fatal),
            "the grammar checked the file as a unit: {:#?}",
            run.report.findings
        );
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

    /// A repository with no `knowledge/schemas/` checks no document against
    /// any contract, and passes. The counts are the only thing that tells
    /// that run apart from one where every document conformed — which is the
    /// state every managed repo is in today, because the pack ships the
    /// templates and not the schemas.
    #[test]
    fn a_repository_with_no_schemas_checks_no_document_and_says_so() {
        let dir = tempfile::tempdir().unwrap();
        let knowledge = dir.path().join("knowledge");
        std::fs::create_dir_all(&knowledge).unwrap();
        std::fs::write(
            knowledge.join("manifest.sokf.yaml"),
            "sokf: \"0.3\"\nname: t\n",
        )
        .unwrap();
        std::fs::write(
            knowledge.join("a.md"),
            "---\ntype: Architecture\nid: a\n---\n\n# Whatever\n",
        )
        .unwrap();
        let run = validate_repo(dir.path(), &knowledge, &[], &live()).unwrap();
        assert_eq!(run.schemas, 0, "no schemas to read");
        assert_eq!(run.documents, 0, "so no document is checked");
        assert!(run.report.passed(), "{:#?}", run.report.findings);
    }

    /// A glob is the fallback for documents that carry no frontmatter, so a
    /// schema declaring a `type` const has no use for one. Dispatch takes the
    /// type and ignores the glob, which would leave the glob reading as a
    /// second, live way in; this is what keeps the two from both being true of
    /// one schema.
    #[test]
    fn a_schema_dispatching_by_type_declares_no_glob() {
        let dir = repo().join("knowledge/schemas");
        let mut both = Vec::new();
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "md") || path.ends_with("index.md") {
                continue;
            }
            let name = relative(&repo(), &path);
            let text = std::fs::read_to_string(&path).unwrap();
            let Ok(Some(schema)) = schema::document::DocSchema::parse(&name, &text) else {
                continue;
            };
            if schema.type_const().is_some() && schema.declares_glob() {
                both.push(name);
            }
        }
        assert!(both.is_empty(), "these declare both: {both:#?}");
    }

    /// This repository does carry schemas, so the same counts are non-zero —
    /// the assertion that the check is reachable at all.
    #[test]
    fn this_repository_checks_its_documents_against_its_schemas() {
        let root = repo();
        let run = validate_repo(&root, &root.join("knowledge"), &[], &live()).unwrap();
        assert!(run.schemas >= 40, "schemas read: {}", run.schemas);
        assert!(run.documents >= 80, "documents checked: {}", run.documents);
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

    /// A format finding fails the run without touching what SOKF reported.
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

    /// The repository the named-run parity tests share: a manifest, one
    /// schema, a concept violating it, and a concept whose type names no
    /// schema.
    fn parity_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let knowledge = dir.path().join("knowledge");
        std::fs::create_dir_all(knowledge.join("schemas")).unwrap();
        std::fs::write(
            knowledge.join("manifest.sokf.yaml"),
            "sokf: \"0.3\"\nname: t\n",
        )
        .unwrap();
        std::fs::write(
            knowledge.join("schemas/thing.md"),
            r#"---
type: Schema
id: schema-thing
title: Thing Schema
description: A governed thing, for the test.
---

# Thing Schema

Structural rules for a thing.

````yaml
description: A governed thing.

frontmatter:
  type:
    const: Thing

sections:
  - heading: "First"
    level: 2
    required: true
    content: prose
  - heading: "Second"
    level: 2
    required: true
    content: prose

example: |
  ---
  type: Thing
  ---

  # A thing

  ## First

  Prose.

  ## Second

  More prose.
````
"#,
        )
        .unwrap();
        std::fs::write(
            knowledge.join("a.md"),
            "---\ntype: Thing\nid: a\n---\n\n# A\n\n## First\n\nx\n",
        )
        .unwrap();
        std::fs::write(
            knowledge.join("b.md"),
            "---\ntype: Invented\nid: b\n---\n\n# B\n\nx\n",
        )
        .unwrap();
        dir
    }

    /// Covers I040: a checkout carrying CRLF reports exactly what an LF one
    /// does — the same schemas registered, the same documents governed, the
    /// same findings. A schema whose contract block was read by a byte
    /// comparison registered no type, and a generated block compared by
    /// bytes differed on every line, which is what the Windows job saw.
    #[test]
    fn a_crlf_checkout_reports_what_an_lf_one_does() {
        let g = live();
        let findings = |run: &RepoReport| -> Vec<(String, String, bool)> {
            run.report
                .findings
                .iter()
                .map(|f| (f.path.clone(), f.message.clone(), f.fatal))
                .collect()
        };
        // Cites `a` and carries the generated block for it, so the block
        // comparison is exercised and not only the schema parse.
        let citing = "---\ntype: Thing\nid: citing\n---\n\n# Citing\n\n## First\n\n\
                      See [a][sokf:a].\n\n## Second\n\nx\n\n\
                      <!-- sokf:links -->\n[sokf:a]: /knowledge/a.md\n";
        let names = [
            "schemas/thing.md",
            "a.md",
            "b.md",
            "citing.md",
            "manifest.sokf.yaml",
        ];

        let lf_dir = parity_repo();
        std::fs::write(lf_dir.path().join("knowledge/citing.md"), citing).unwrap();
        let lf = validate_repo(lf_dir.path(), &lf_dir.path().join("knowledge"), &[], &g).unwrap();

        let crlf_dir = parity_repo();
        let knowledge = crlf_dir.path().join("knowledge");
        std::fs::write(knowledge.join("citing.md"), citing).unwrap();
        for name in names {
            let path = knowledge.join(name);
            let text = std::fs::read_to_string(&path).unwrap();
            std::fs::write(&path, text.replace('\n', "\r\n")).unwrap();
        }
        let crlf = validate_repo(crlf_dir.path(), &knowledge, &[], &g).unwrap();

        assert!(
            !findings(&lf)
                .iter()
                .any(|(p, m, _)| p == "citing.md" && m.contains("generated form")),
            "the fixture's block is in generated form to begin with: {:#?}",
            findings(&lf)
        );

        assert!(
            lf.schemas > 0 && lf.documents > 0,
            "the fixture is governed at all: {} schemas, {} documents",
            lf.schemas,
            lf.documents
        );
        assert_eq!(
            crlf.schemas, lf.schemas,
            "the CRLF run read a different schema set"
        );
        assert_eq!(
            crlf.documents, lf.documents,
            "the CRLF run checked a different number of documents"
        );
        assert_eq!(
            findings(&crlf),
            findings(&lf),
            "the CRLF run reports different findings"
        );
    }

    /// A named concept gets every finding the bare run reports for that
    /// file, and the run reports nothing about any other file (I019
    /// criterion 1, ADR-026).
    #[test]
    fn a_named_concept_gets_the_bare_runs_findings_for_that_file() {
        let dir = parity_repo();
        let knowledge = dir.path().join("knowledge");
        let g = live();
        let bare = validate_repo(dir.path(), &knowledge, &[], &g).unwrap();
        let one = vec![PathBuf::from("knowledge/a.md")];
        let named = validate_repo(dir.path(), &knowledge, &one, &g).unwrap();

        assert!(named.schemas > 0, "the schema set was loaded");
        assert!(named.documents > 0, "the named file was checked");
        let bare_for_file: Vec<(&str, &str, bool)> = bare
            .report
            .findings
            .iter()
            .filter(|f| f.path == "knowledge/a.md")
            .map(|f| (f.path.as_str(), f.message.as_str(), f.fatal))
            .collect();
        assert!(!bare_for_file.is_empty(), "the fixture violates its schema");
        let named_all: Vec<(&str, &str, bool)> = named
            .report
            .findings
            .iter()
            .map(|f| (f.path.as_str(), f.message.as_str(), f.fatal))
            .collect();
        assert_eq!(
            named_all, bare_for_file,
            "the named run's findings are exactly the bare run's for the file"
        );
    }

    /// A named README.md is dispatched by `schema-readme`'s glob, and
    /// CHANGELOG.md likewise (I019 criterion 2): the schema set loads and
    /// the named file is the one document checked.
    #[test]
    fn a_named_readme_or_changelog_is_dispatched_by_glob() {
        let root = repo();
        let g = live();
        for name in ["README.md", "CHANGELOG.md"] {
            let one = vec![PathBuf::from(name)];
            let run = validate_repo(&root, &root.join("knowledge"), &one, &g).unwrap();
            assert!(run.schemas > 0, "{name}: the schema set was loaded");
            assert_eq!(run.documents, 1, "{name} is governed by a glob schema");
            assert!(
                run.report.findings.iter().all(|f| f.path == name),
                "{name}: {:#?}",
                run.report.findings
            );
        }
    }

    /// A named file whose type names no schema gets the bare run's
    /// unknown-type finding (I019 criterion 3).
    #[test]
    fn a_named_file_with_an_unknown_type_gets_the_unknown_type_finding() {
        let dir = parity_repo();
        let one = vec![PathBuf::from("knowledge/b.md")];
        let named =
            validate_repo(dir.path(), &dir.path().join("knowledge"), &one, &live()).unwrap();
        assert!(
            named
                .report
                .findings
                .iter()
                .any(|f| f.path == "knowledge/b.md"
                    && f.message.contains("type `Invented` names no schema")),
            "{:#?}",
            named.report.findings
        );
        assert!(
            named
                .report
                .findings
                .iter()
                .all(|f| f.path == "knowledge/b.md"),
            "{:#?}",
            named.report.findings
        );
    }

    /// A fault reading the knowledge fails only a run whose paths touch it:
    /// a run scoped elsewhere skips the context the fault sits in, and a run
    /// asking about the knowledge fails exactly where the bare run does.
    #[test]
    fn broken_knowledge_fails_only_a_run_that_touches_it() {
        let dir = parity_repo();
        let knowledge = dir.path().join("knowledge");
        std::fs::write(knowledge.join("binary.md"), [0xFF, 0xFE, 0x00]).unwrap();
        std::fs::write(dir.path().join("notes.md"), "no frontmatter here\n").unwrap();
        let g = live();

        let outside = vec![PathBuf::from("notes.md")];
        let run = validate_repo(dir.path(), &knowledge, &outside, &g).unwrap();
        assert!(
            run.report.findings.iter().all(|f| f.path == "notes.md"),
            "{:#?}",
            run.report.findings
        );

        let bare = validate_repo(dir.path(), &knowledge, &[], &g);
        let one = vec![PathBuf::from("knowledge/a.md")];
        let named = validate_repo(dir.path(), &knowledge, &one, &g);
        assert_eq!(bare.is_err(), named.is_err(), "the two runs agree");
    }

    /// A named knowledge file whose frontmatter does not parse gets the bare
    /// run's findings — the SOKF half's — and never a schema check the bare
    /// run would not give it.
    #[test]
    fn a_named_broken_knowledge_file_gets_the_bare_runs_findings() {
        let dir = parity_repo();
        let knowledge = dir.path().join("knowledge");
        std::fs::write(
            knowledge.join("c.md"),
            "---\ntype: Thing\ntype: Thing\nid: c\n---\n\n# C\n\n## First\n\nx\n",
        )
        .unwrap();
        let g = live();
        let bare = validate_repo(dir.path(), &knowledge, &[], &g).unwrap();
        let one = vec![PathBuf::from("knowledge/c.md")];
        let named = validate_repo(dir.path(), &knowledge, &one, &g).unwrap();

        let for_file = |run: &RepoReport| -> Vec<(String, String, bool)> {
            run.report
                .findings
                .iter()
                .filter(|f| f.path == "knowledge/c.md")
                .map(|f| (f.path.clone(), f.message.clone(), f.fatal))
                .collect()
        };
        assert_eq!(for_file(&named), for_file(&bare));
        assert!(
            named
                .report
                .findings
                .iter()
                .all(|f| f.path == "knowledge/c.md"),
            "{:#?}",
            named.report.findings
        );
    }

    /// A concept whose name a grammar kind would claim — a `.prompt.md`
    /// suffix — is still the knowledge's: named, it gets the bare run's
    /// findings, never the grammar's reading (I019 criterion 1).
    #[test]
    fn a_named_concept_with_a_unit_suffix_is_not_misread() {
        let dir = parity_repo();
        let knowledge = dir.path().join("knowledge");
        std::fs::write(
            knowledge.join("p.prompt.md"),
            "---\ntype: Thing\nid: p\n---\n\n# P\n\n## First\n\nx\n\n## Second\n\ny\n",
        )
        .unwrap();
        let one = vec![PathBuf::from("knowledge/p.prompt.md")];
        let run = validate_repo(dir.path(), &knowledge, &one, &live()).unwrap();
        assert!(run.report.passed(), "{:#?}", run.report.findings);
        assert_eq!(run.files, 0, "the grammar never saw the concept");
    }

    /// The schema set has one source: a custom knowledge directory serves a
    /// bare and a named run alike, so the two cannot disagree about which
    /// schemas govern.
    #[test]
    fn a_custom_knowledge_directory_serves_both_runs_one_schema_set() {
        let dir = tempfile::tempdir().unwrap();
        let kb = dir.path().join("kb");
        std::fs::create_dir_all(kb.join("schemas")).unwrap();
        std::fs::write(kb.join("manifest.sokf.yaml"), "sokf: \"0.3\"\nname: t\n").unwrap();
        std::fs::copy(
            parity_repo().path().join("knowledge/schemas/thing.md"),
            kb.join("schemas/thing.md"),
        )
        .unwrap();
        std::fs::write(
            kb.join("a.md"),
            "---\ntype: Thing\nid: a\n---\n\n# A\n\n## First\n\nx\n",
        )
        .unwrap();
        let g = live();
        let bare = validate_repo(dir.path(), &kb, &[], &g).unwrap();
        assert!(bare.schemas > 0, "the bare run reads kb/schemas");
        let one = vec![PathBuf::from("kb/a.md")];
        let named = validate_repo(dir.path(), &kb, &one, &g).unwrap();
        assert_eq!(named.schemas, bare.schemas);
        let for_file = |run: &RepoReport| -> Vec<(String, bool)> {
            run.report
                .findings
                .iter()
                .filter(|f| f.path == "kb/a.md")
                .map(|f| (f.message.clone(), f.fatal))
                .collect()
        };
        assert!(
            !for_file(&bare).is_empty(),
            "the fixture violates its schema"
        );
        assert_eq!(for_file(&named), for_file(&bare));
    }

    /// With no schema set to resolve against, dispatch means nothing: a typed
    /// file outside the knowledge takes the fallback kind rather than passing
    /// unchecked, while a typed concept keeps the knowledge's treatment.
    #[test]
    fn a_typed_file_without_any_schema_set_takes_the_fallback_kind() {
        let dir = tempfile::tempdir().unwrap();
        let knowledge = dir.path().join("knowledge");
        std::fs::create_dir_all(&knowledge).unwrap();
        std::fs::write(
            knowledge.join("manifest.sokf.yaml"),
            "sokf: \"0.3\"\nname: t\n",
        )
        .unwrap();
        std::fs::write(
            knowledge.join("a.md"),
            "---\ntype: Architecture\nid: a\n---\n\n# Whatever\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("outside.md"),
            "---\ntype: Architecture\n---\n\n# Whatever\n",
        )
        .unwrap();
        let g = live();

        let outside = vec![PathBuf::from("outside.md")];
        let run = validate_repo(dir.path(), &knowledge, &outside, &g).unwrap();
        assert_eq!(run.files, 1, "the fallback kind claimed the file");
        assert!(!run.report.passed(), "the grammar read it as a unit");

        let concept = vec![PathBuf::from("knowledge/a.md")];
        let run = validate_repo(dir.path(), &knowledge, &concept, &g).unwrap();
        assert_eq!(run.files, 0, "the concept stays the knowledge's");
        assert!(run.report.passed(), "{:#?}", run.report.findings);
    }

    /// A `..` spelling names the same file: the run resolves it away, so
    /// coverage compares one spelling (I019 criterion 1).
    #[test]
    fn a_dot_dot_spelling_names_the_same_file() {
        let dir = parity_repo();
        let knowledge = dir.path().join("knowledge");
        let g = live();
        let plain = vec![PathBuf::from("knowledge/a.md")];
        let dotted = vec![PathBuf::from("knowledge/../knowledge/a.md")];
        let a = validate_repo(dir.path(), &knowledge, &plain, &g).unwrap();
        let b = validate_repo(dir.path(), &knowledge, &dotted, &g).unwrap();
        let flat = |run: &RepoReport| -> Vec<(String, String, bool)> {
            run.report
                .findings
                .iter()
                .map(|f| (f.path.clone(), f.message.clone(), f.fatal))
                .collect()
        };
        assert!(!flat(&a).is_empty(), "the fixture violates its schema");
        assert_eq!(flat(&a), flat(&b));
    }

    /// Naming a directory and a file inside it checks the file once: the
    /// findings and the count match the directory run's.
    #[test]
    fn naming_a_directory_and_a_file_inside_it_checks_the_file_once() {
        let root = repo();
        let g = live();
        let dir_only = vec![PathBuf::from(".claude/skills")];
        let both = vec![
            PathBuf::from(".claude/skills"),
            PathBuf::from(".claude/skills/handoff/SKILL.md"),
        ];
        let a = validate_repo(&root, &root.join("knowledge"), &dir_only, &g).unwrap();
        let b = validate_repo(&root, &root.join("knowledge"), &both, &g).unwrap();
        assert_eq!(a.files, b.files);
        assert_eq!(a.report.findings.len(), b.report.findings.len());
    }

    /// A named run reports no finding about a file the paths do not cover
    /// (I019 criterion 1), where the bare run does report that file.
    #[test]
    fn a_named_run_reports_nothing_outside_its_paths() {
        let dir = parity_repo();
        let knowledge = dir.path().join("knowledge");
        let g = live();
        let bare = validate_repo(dir.path(), &knowledge, &[], &g).unwrap();
        assert!(
            bare.report
                .findings
                .iter()
                .any(|f| f.path == "knowledge/b.md"),
            "the bare run reports the uncovered file: {:#?}",
            bare.report.findings
        );
        let one = vec![PathBuf::from("knowledge/a.md")];
        let named = validate_repo(dir.path(), &knowledge, &one, &g).unwrap();
        assert!(
            named
                .report
                .findings
                .iter()
                .all(|f| f.path != "knowledge/b.md"),
            "{:#?}",
            named.report.findings
        );
    }
}

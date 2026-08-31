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
    /// Files the grammar governs, read this run, which the caller emits as
    /// `files`. Zero and clean is a run that found nothing to check,
    /// indistinguishable from a pass unless the number is shown.
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
/// for what is reported: findings name only files the paths cover. The run
/// still reads the knowledge and the schema set, and a named file joins the
/// document candidates, so a named document gets exactly the findings a bare
/// run gives it (ADR-026). The bundle is validated as a whole only when one
/// of the given paths is the bundle or contains it. Findings are grouped by
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
    // `covered`; the coverage filter below drops the rest.
    let named = paths.iter().any(|p| bundle.starts_with(p));
    let covered = named || paths.is_empty();
    if named || bundle.is_dir() {
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
            subjects.push(lifecycle::Subject {
                path: path.clone(),
                doc_type: concept.kind.clone(),
                lifecycle: concept.lifecycle.clone(),
            });
            documents.push((
                path,
                read(&bundle.join(&concept.path))?,
                Some(concept.kind.clone()),
            ));
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
                continue;
            }
            let name = relative(&repo_root, path);
            let text = read(path)?;
            if schema::detect_kind(path, g, true).is_some() {
                // A file named on the command line is checked whatever it is
                // called, as the reference does: the fallback kind applies
                // where the walk would have passed it over.
                files.push((name.clone(), text.clone()));
            }
            // A named file joins the document candidates, so the schema half
            // reaches it (ADR-026). Schemas and indexes stay out, as they do
            // above; a knowledge concept is already a candidate.
            if !name.contains("/schemas/")
                && !name.ends_with("/index.md")
                && name != "index.md"
                && !documents.iter().any(|(p, ..)| *p == name)
            {
                let doc_type = frontmatter_type(&text);
                documents.push((name, text, doc_type));
            }
        }
    }

    findings.extend(schema::check_files(&files, g).into_iter().map(|f| Finding {
        path: f.file,
        message: f.message,
        fatal: f.fatal,
    }));

    // Documents against the schemas that govern them. In a bare run the
    // schemas come from the files just walked — `knowledge/schemas` is one of
    // the grammar's roots — so a repository whose schemas were not read simply
    // checks no documents, rather than reporting every one of them as
    // ungoverned. A named run does not walk the roots, so its schema set
    // comes from the resolved knowledge's own schemas directory: the same
    // files, read the same way (ADR-026).
    let is_schema = |name: &str| name.contains("/schemas/") && !name.ends_with("/index.md");
    let schema_files: Vec<(String, String)> = if paths.is_empty() {
        files
            .iter()
            .filter(|(name, _)| is_schema(name))
            .cloned()
            .collect()
    } else {
        let dir = bundle.join("schemas");
        let mut walked = Vec::new();
        if dir.is_dir() {
            collect(&repo_root, &dir, g, &mut walked)?;
        }
        walked.into_iter().filter(|(name, _)| is_schema(name)).collect()
    };
    let mut schemas = 0;
    let mut checked = 0;
    if !schema_files.is_empty() {
        // Two documents carry no frontmatter and are named by glob instead.
        // Skipped when already a candidate, as a named README.md is.
        for name in ["README.md", "CHANGELOG.md"] {
            let path = repo_root.join(name);
            if path.is_file() && !documents.iter().any(|(p, ..)| p == name) {
                documents.push((name.to_string(), read(&path)?, None));
            }
        }
        schemas = schema_files.len();
        let (set, mut schema_findings) = schema::document::SchemaSet::load(&schema_files);
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
        files: files.len(),
        schemas,
        documents: checked,
    })
}

/// Repair the SOKF knowledge's links in place, ahead of validating it.
///
/// The repairs are [`fix`]'s; this is where the run resolves the same paths
/// [`validate_repo`] resolves, so `--fix` and the check that follows it read
/// one knowledge directory — and cover it on the same condition, so naming a
/// skill on the command line checks that skill and repairs nothing.
///
/// # Errors
/// The knowledge is unreadable, or a document cannot be written.
pub fn fix_repo(repo_root: &Path, bundle: &Path, paths: &[PathBuf]) -> Result<Repair> {
    let repo_root = normalise(&std::env::current_dir().unwrap_or_default(), repo_root);
    let bundle = normalise(&repo_root, bundle);
    let paths: Vec<PathBuf> = paths.iter().map(|p| normalise(&repo_root, p)).collect();
    let covered = paths.is_empty() || paths.iter().any(|p| bundle.starts_with(p));
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
    schema::read::parse_frontmatter(&split.fm)
        .into_iter()
        .find(|e| e.key == "type")
        .and_then(|e| e.scalar)
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
        let one = vec![PathBuf::from("knowledge/schemas/adr.md")];
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
        let bare_for_file: Vec<&Finding> = bare
            .report
            .findings
            .iter()
            .filter(|f| f.path == "knowledge/a.md")
            .collect();
        assert!(!bare_for_file.is_empty(), "the fixture violates its schema");
        for finding in &bare_for_file {
            assert!(
                named
                    .report
                    .findings
                    .iter()
                    .any(|f| f.path == finding.path && f.message == finding.message),
                "bare finding missing from the named run: {finding:#?}\nnamed: {:#?}",
                named.report.findings
            );
        }
        assert!(
            named.report.findings.iter().all(|f| f.path == "knowledge/a.md"),
            "{:#?}",
            named.report.findings
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
            named.report.findings.iter().all(|f| f.path == "knowledge/b.md"),
            "{:#?}",
            named.report.findings
        );
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
            bare.report.findings.iter().any(|f| f.path == "knowledge/b.md"),
            "the bare run reports the uncovered file: {:#?}",
            bare.report.findings
        );
        let one = vec![PathBuf::from("knowledge/a.md")];
        let named = validate_repo(dir.path(), &knowledge, &one, &g).unwrap();
        assert!(
            named.report.findings.iter().all(|f| f.path != "knowledge/b.md"),
            "{:#?}",
            named.report.findings
        );
    }
}

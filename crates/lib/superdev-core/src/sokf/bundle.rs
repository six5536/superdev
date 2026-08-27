//! bundle.rs — loading a directory of AOKF concept files.

use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml_ng::Value;

use super::concept::{Concept, ParseError, parse_concept};
use crate::error::{Error, Result};

/// A loaded AOKF bundle: its manifest, the concepts that parsed, and the
/// files that did not.
#[derive(Debug, Clone)]
pub struct Bundle {
    /// Absolute path of the bundle directory.
    pub root: PathBuf,
    /// The root `manifest.sokf.yaml`, when it exists and parses.
    pub manifest: Option<BundleManifest>,
    /// Why the manifest did not parse; `None` when it parsed or is absent.
    pub manifest_error: Option<String>,
    /// Concepts that parsed, sorted by bundle-relative path.
    pub concepts: Vec<Concept>,
    /// The reserved `index.md` files — bundle-relative path and full text —
    /// sorted by path. Not concepts, but the validator checks their entries.
    pub indexes: Vec<(String, String)>,
    /// Files that did not parse, sorted by bundle-relative path.
    pub broken: Vec<ParseError>,
}

/// The bundle-level `manifest.sokf.yaml`.
#[derive(Debug, Clone)]
pub struct BundleManifest {
    /// Spec version the bundle targets.
    pub sokf: Option<String>,
    /// Bundle name.
    pub name: Option<String>,
    /// The whole document, for checks the typed fields drop — stamped keys,
    /// and a manifest that is not a mapping at all.
    pub raw: Value,
}

/// Load every concept under `dir`, recursively.
///
/// `manifest.sokf.yaml` (bundle root only) and `index.md` (any directory) are
/// reserved by the spec and are not concepts; hidden directories are skipped.
/// A file that fails to parse lands in [`Bundle::broken`] rather than failing
/// the load — a bundle with one bad file is still worth serving.
///
/// # Errors
///
/// Returns [`Error::Io`] when the directory cannot be walked or a file cannot
/// be read.
pub fn load_bundle(dir: &Path) -> Result<Bundle> {
    let root = std::path::absolute(dir).map_err(|e| Error::Io {
        path: dir.to_path_buf(),
        source: e,
    })?;

    let (concept_paths, index_paths) = markdown_files(&root)?;
    let mut concepts = Vec::new();
    let mut broken = Vec::new();
    for path in concept_paths {
        let relative = relative_path(&root, &path);
        match parse_concept(&relative, &read(&path)?) {
            Ok(concept) => concepts.push(concept),
            Err(e) => broken.push(e),
        }
    }
    let mut indexes = Vec::new();
    for path in index_paths {
        indexes.push((relative_path(&root, &path), read(&path)?));
    }
    concepts.sort_by(|a, b| a.path.cmp(&b.path));
    broken.sort_by(|a, b| a.path.cmp(&b.path));
    indexes.sort_by(|(a, _), (b, _)| a.cmp(b));

    let (manifest, manifest_error) = load_manifest(&root)?;
    Ok(Bundle {
        root,
        manifest,
        manifest_error,
        concepts,
        indexes,
        broken,
    })
}

/// Read a file, naming it in the error.
fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Every `.md` file under `root`, as (concepts, `index.md` files), in
/// directory-walk order.
fn markdown_files(root: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut files = Vec::new();
    let mut indexes = Vec::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        let entries = fs::read_dir(&dir).map_err(|e| Error::Io {
            path: dir.clone(),
            source: e,
        })?;
        for entry in entries {
            let entry = entry.map_err(|e| Error::Io {
                path: dir.clone(),
                source: e,
            })?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let file_type = entry.file_type().map_err(|e| Error::Io {
                path: path.clone(),
                source: e,
            })?;
            if file_type.is_dir() {
                dirs.push(path);
            } else if name == "index.md" {
                indexes.push(path);
            } else if name.ends_with(".md") {
                files.push(path);
            }
        }
    }
    Ok((files, indexes))
}

/// Read the root manifest, returning it and the reason it did not parse. A
/// manifest that does not parse reads as absent, so a consumer sees no
/// half-read manifest; the validator reports the reason.
fn load_manifest(root: &Path) -> Result<(Option<BundleManifest>, Option<String>)> {
    let path = root.join("manifest.sokf.yaml");
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((None, None)),
        Err(e) => return Err(Error::Io { path, source: e }),
    };
    match serde_yaml_ng::from_str::<Value>(&text) {
        Ok(raw) => Ok((
            Some(BundleManifest {
                sokf: raw["sokf"].as_str().map(str::to_string),
                name: raw["name"].as_str().map(str::to_string),
                raw,
            }),
            None,
        )),
        Err(e) => Ok((None, Some(e.to_string()))),
    }
}

/// `path` relative to the bundle root, with forward slashes on every
/// platform.
fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_concepts_and_skips_reserved_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("manifest.sokf.yaml"),
            "sokf: \"0.1\"\nname: t\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("index.md"), "# Index\n").unwrap();
        std::fs::write(dir.path().join("a.md"), "---\ntype: T\nid: a\n---\nbody\n").unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("sub/index.md"), "# Sub\n").unwrap();
        std::fs::write(
            dir.path().join("sub/b.md"),
            "---\ntype: T\nid: b\n---\nbody\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("sub/broken.md"), "no frontmatter\n").unwrap();
        let b = load_bundle(dir.path()).unwrap();
        assert_eq!(b.manifest.as_ref().unwrap().name.as_deref(), Some("t"));
        let paths: Vec<&str> = b.concepts.iter().map(|c| c.path.as_str()).collect();
        assert_eq!(paths, vec!["a.md", "sub/b.md"]);
        assert_eq!(b.broken.len(), 1);
    }

    #[test]
    fn hidden_directories_and_non_markdown_are_skipped() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join(".git/a.md"), "---\ntype: T\n---\nb\n").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "---\ntype: T\n---\nb\n").unwrap();
        let b = load_bundle(dir.path()).unwrap();
        assert!(b.concepts.is_empty());
        assert!(b.manifest.is_none());
        assert!(b.root.is_absolute());
    }

    #[test]
    fn an_unparseable_manifest_reads_as_absent_but_reports_why() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("manifest.sokf.yaml"), "sokf: [unclosed\n").unwrap();
        let b = load_bundle(dir.path()).unwrap();
        assert!(b.manifest.is_none());
        assert!(b.manifest_error.is_some());
    }

    #[test]
    fn index_files_are_kept_with_their_text() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("index.md"), "# Index\n\n* [a](a.md)\n").unwrap();
        std::fs::write(dir.path().join("sub/index.md"), "# Sub\n").unwrap();
        let b = load_bundle(dir.path()).unwrap();
        let paths: Vec<&str> = b.indexes.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(paths, vec!["index.md", "sub/index.md"]);
        assert!(b.indexes[0].1.contains("[a](a.md)"));
    }

    #[test]
    fn a_manifest_that_is_not_a_mapping_keeps_its_raw_value() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("manifest.sokf.yaml"), "- one\n").unwrap();
        let b = load_bundle(dir.path()).unwrap();
        assert!(b.manifest_error.is_none());
        assert!(b.manifest.unwrap().raw.is_sequence());
    }

    #[test]
    fn a_missing_directory_is_an_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let e = load_bundle(&dir.path().join("nope")).unwrap_err();
        assert!(matches!(e, Error::Io { .. }));
    }
}

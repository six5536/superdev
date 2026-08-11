//! bundle.rs — loading a directory of AOKF concept files.

use std::fs;
use std::path::{Path, PathBuf};

use super::concept::{Concept, ParseError, parse_concept};
use crate::error::{Error, Result};

/// A loaded AOKF bundle: its manifest, the concepts that parsed, and the
/// files that did not.
#[derive(Debug, Clone)]
pub struct Bundle {
    /// Absolute path of the bundle directory.
    pub root: PathBuf,
    /// The root `manifest.aokf.yaml`, when it exists and parses.
    pub manifest: Option<BundleManifest>,
    /// Concepts that parsed, sorted by bundle-relative path.
    pub concepts: Vec<Concept>,
    /// Files that did not parse, sorted by bundle-relative path.
    pub broken: Vec<ParseError>,
}

/// The bundle-level `manifest.aokf.yaml`.
#[derive(Debug, Clone)]
pub struct BundleManifest {
    /// Spec version the bundle targets.
    pub aokf: Option<String>,
    /// Bundle name.
    pub name: Option<String>,
}

/// Load every concept under `dir`, recursively.
///
/// `manifest.aokf.yaml` (bundle root only) and `index.md` (any directory) are
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

    let mut concepts = Vec::new();
    let mut broken = Vec::new();
    for path in concept_files(&root)? {
        let relative = relative_path(&root, &path);
        let text = fs::read_to_string(&path).map_err(|e| Error::Io {
            path: path.clone(),
            source: e,
        })?;
        match parse_concept(&relative, &text) {
            Ok(concept) => concepts.push(concept),
            Err(e) => broken.push(e),
        }
    }
    concepts.sort_by(|a, b| a.path.cmp(&b.path));
    broken.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(Bundle {
        manifest: load_manifest(&root)?,
        root,
        concepts,
        broken,
    })
}

/// Every non-reserved `.md` file under `root`, in directory-walk order.
fn concept_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
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
            } else if name.ends_with(".md") && name != "index.md" {
                files.push(path);
            }
        }
    }
    Ok(files)
}

/// Read the root manifest. A manifest that does not parse reads as absent;
/// the validator reports it from the file itself.
fn load_manifest(root: &Path) -> Result<Option<BundleManifest>> {
    let path = root.join("manifest.aokf.yaml");
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::Io { path, source: e }),
    };
    let Ok(value) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&text) else {
        return Ok(None);
    };
    Ok(Some(BundleManifest {
        aokf: value["aokf"].as_str().map(str::to_string),
        name: value["name"].as_str().map(str::to_string),
    }))
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
            dir.path().join("manifest.aokf.yaml"),
            "aokf: \"0.1\"\nname: t\n",
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
    fn an_unparseable_manifest_reads_as_absent() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("manifest.aokf.yaml"), "aokf: [unclosed\n").unwrap();
        assert!(load_bundle(dir.path()).unwrap().manifest.is_none());
    }

    #[test]
    fn a_missing_directory_is_an_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let e = load_bundle(&dir.path().join("nope")).unwrap_err();
        assert!(matches!(e, Error::Io { .. }));
    }
}

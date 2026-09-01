//! contract_files.rs — the config and format contracts bound to the types
//! that read those files.
//!
//! [ADR-036] obliges a project to bind its implemented interface to the
//! contract's declared surface. These interfaces are hand-written, so tests
//! bind them: a contract's declared file must parse with the reader that
//! reads it in production, and every key that reader writes must appear in
//! the contract. An element on one side and not the other fails.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn repo(path: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../../..", path]
        .iter()
        .collect()
}

/// The first fenced block carrying `tag`, without its markers.
///
/// Scanned a line at a time, so a CRLF checkout reads as its LF twin: a
/// fence is a fence whatever ends the line (I040). The closing marker is the
/// one that opened the block, so a ```` ```` ```` block may contain ``` ``` ````.
fn fenced_block(text: &str, tag: &str) -> Option<String> {
    let mut lines = text.lines();
    let marker = loop {
        let trimmed = lines.next()?.trim_start();
        let ticks = trimmed.len() - trimmed.trim_start_matches('`').len();
        if ticks >= 3 && trimmed[ticks..].trim() == tag {
            break trimmed[..ticks].to_string();
        }
    };
    let mut body = Vec::new();
    for line in lines {
        if line.trim_start().starts_with(&marker) {
            return Some(body.join("\n"));
        }
        body.push(line);
    }
    None
}

/// The first fenced block tagged `tag` in the contract at `path`.
fn block(path: &str, tag: &str) -> String {
    let text = std::fs::read_to_string(repo(path)).expect("the contract is on file");
    fenced_block(&text, tag).unwrap_or_else(|| panic!("{path} carries no {tag} block"))
}

/// Covers I035 criteria 1 and 4: the manifest the config contract declares
/// parses with the reader that reads `.superdev/config.toml` in production,
/// so every key it declares is a key superdev accepts.
#[test]
fn the_declared_manifest_parses_with_the_real_reader() {
    let declared = block(
        "knowledge/contracts/public/active/contract-004-config-superdev.md",
        "toml",
    );
    let manifest = superdev_core::manifest::Manifest::parse(&declared)
        .expect("the contract's declared manifest is a manifest superdev reads");
    assert_eq!(manifest.blueprint, "0.2.0");
    assert_eq!(manifest.packs.len(), 2, "both declared packs are read");
    assert!(
        manifest.capabilities.contains_key("code-index"),
        "a declared capability table is read"
    );
}

/// Covers I035 criterion 4: every key the manifest writer emits appears in
/// the contract's declared file, so superdev cannot write a key its contract
/// never named.
#[test]
fn every_key_the_manifest_writes_is_declared() {
    let declared = block(
        "knowledge/contracts/public/active/contract-004-config-superdev.md",
        "toml",
    );
    // A manifest carrying every optional part, so nothing is missed for
    // being absent from a default value.
    let full = superdev_core::manifest::Manifest::parse(
        "blueprint = \"0.2.0\"\n\
         [template]\n\
         name = \"rust-npm\"\n\
         project-name = \"Widget\"\n\
         project-slug = \"widget\"\n\
         version = \"0.2.0\"\n\
         [[packs]]\n\
         source = \"github:six5536/superdev\"\n\
         rev = \"assets-v1.4.0\"\n\
         [knowledge]\n\
         custom = [\"humanise\"]\n\
         [knowledge.embeddings]\n\
         provider = \"openai\"\n\
         model = \"text-embedding-3-small\"\n\
         [code-index]\n\
         provider = \"codegraph\"\n\
         version = \"1.5.0\"\n\
         [skills]\n\
         provider = \"superdev-skills\"\n\
         version = \"0.2.0\"\n",
    )
    .expect("the fully-populated manifest parses");

    let written = full.to_toml();
    // Compare key paths, not bare words: `version` under one table must not
    // satisfy `version` under another.
    let paths = |toml: &str| -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let mut table = String::new();
        for line in toml.lines() {
            // A trailing comment is not part of a key or a table name.
            let trimmed = line.split_once(" #").map_or(line, |(before, _)| before);
            let trimmed = trimmed.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with('[') {
                table = trimmed.trim_matches(['[', ']']).to_string();
                out.insert(table.clone());
            } else if let Some((name, _)) = trimmed.split_once('=') {
                let name = name.trim();
                out.insert(if table.is_empty() {
                    name.to_string()
                } else {
                    format!("{table}.{name}")
                });
            }
        }
        out
    };
    let written_paths = paths(&written);
    let declared_paths = paths(&declared);
    let missing: Vec<&String> = written_paths.difference(&declared_paths).collect();
    assert!(
        missing.is_empty(),
        "DEFECT — superdev writes keys the config contract does not declare: {missing:?}\n\
         written:\n{written}"
    );
}

/// Covers I035 criteria 1 and 4: the pack manifest the format contract
/// declares parses with the reader that reads `pack.toml`.
#[test]
fn the_declared_pack_manifest_parses_with_the_real_reader() {
    let declared = block(
        "knowledge/contracts/public/active/contract-005-text-format-pack.md",
        "toml",
    );
    let parsed = superdev_core::pack::PackManifest::parse("the contract", &declared)
        .expect("the contract's declared pack.toml is one superdev reads");
    assert_eq!(parsed.format, 1, "the declared format is the one read");
}

/// Covers I035 criteria 1 and 4: the lock the format contract declares is
/// read by the type that reads `.superdev/lock.toml`, and every key that
/// type writes appears in the contract.
#[test]
fn the_declared_lock_round_trips_through_the_real_type() {
    let declared = block(
        "knowledge/contracts/public/active/contract-006-text-format-lock.md",
        "toml",
    );
    // The contract's block is an illustration with elided hashes, so it is
    // read as the shape it declares rather than as a file to load.
    let readable = declared.replace("sha256:…", "sha256:0000");
    let dir = tempfile::tempdir().expect("a temporary directory");
    let superdev = dir.path().join(".superdev");
    std::fs::create_dir_all(&superdev).expect("the cache directory is made");
    std::fs::write(superdev.join("lock.toml"), &readable).expect("the lock is written");
    let lock = superdev_core::lock::Lock::load(dir.path())
        .expect("the contract's declared lock is one superdev reads");
    assert_eq!(lock.packs.len(), 1, "the declared pack entry is read");
    assert!(
        !lock.files.is_empty(),
        "the declared file hashes are read: {lock:?}"
    );

    // And nothing superdev writes is missing from the contract. The value
    // is built here rather than from the contract, so a key the contract
    // drops is still demanded.
    let full: superdev_core::lock::Lock = toml_edit::de::from_str(
        "[[packs]]\n\
         source = \"github:six5536/superdev\"\n\
         identity = \"github.com/six5536/superdev\"\n\
         rev = \"assets-v1.4.0\"\n\
         digest = \"sha256:0000\"\n\
         format = 1\n\
         [components.code-index]\n\
         provider = \"codegraph\"\n\
         version = \"1.5.0\"\n\
         [files]\n\
         \".agents/superdev.md\" = \"a99e4f86\"\n",
    )
    .expect("the fully-populated lock parses");
    let written = toml_edit::ser::to_string_pretty(&full).expect("the lock serialises");
    let mut missing = Vec::new();
    for line in written.lines() {
        let trimmed = line
            .split_once(" #")
            .map_or(line, |(before, _)| before)
            .trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(table) = trimmed
            .strip_prefix('[')
            .map(|r| r.trim_end_matches(']').trim_start_matches('['))
        {
            let head = table.split('.').next().unwrap_or(table);
            if !declared.contains(head) {
                missing.push(head.to_string());
            }
        } else if let Some((name, _)) = trimmed.split_once('=') {
            let name = name.trim().trim_matches('"');
            if !declared.contains(name) {
                missing.push(name.to_string());
            }
        }
    }
    assert!(
        missing.is_empty(),
        "DEFECT — superdev writes keys the lock contract does not declare: {missing:?}\n\
         written:\n{written}"
    );
}

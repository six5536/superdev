//! contract_files.rs — the config and format contracts bound to the types
//! that read those files.
//!
//! [ADR-036] obliges a project to bind its implemented interface to the
//! contract's declared surface. These interfaces are hand-written, so tests
//! bind them: a contract's declared file must parse with the reader that
//! reads it in production, and every key that reader writes must appear in
//! the contract. An element on one side and not the other fails.

use std::path::PathBuf;

fn repo(path: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../../..", path]
        .iter()
        .collect()
}

/// The first fenced block tagged `tag` in the contract at `path`.
fn block(path: &str, tag: &str) -> String {
    let text = std::fs::read_to_string(repo(path)).expect("the contract is on file");
    text.split(&format!("```{tag}\n"))
        .nth(1)
        .and_then(|rest| rest.split("\n```").next())
        .unwrap_or_else(|| panic!("{path} carries no {tag} block"))
        .to_string()
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
    let mut missing = Vec::new();
    for line in written.lines() {
        let trimmed = line.trim();
        let key = if trimmed.starts_with('[') {
            trimmed.trim_matches(['[', ']']).to_string()
        } else if let Some((name, _)) = trimmed.split_once('=') {
            name.trim().to_string()
        } else {
            continue;
        };
        if key.is_empty() {
            continue;
        }
        // A capability table is named by the capability, and the contract
        // declares the shape once per capability it ships.
        if !declared.contains(&key) {
            missing.push(key);
        }
    }
    assert!(
        missing.is_empty(),
        "superdev writes keys the config contract does not declare: {missing:?}\n\
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

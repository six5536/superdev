//! Enumerates the embedded pack.
//!
//! The stock content lives at `/pack`, reached from this crate as `assets/`
//! (ADR-006). Its layout is what names the items in it (ADR-003), so the
//! binary needs the whole file list, not a hand-written table that a new
//! file would have to be added to twice.
//!
//! Emits `pack_files.rs` into `OUT_DIR`: an array of (pack-relative path,
//! `include_str!` of the file). The contents stay literals in the binary,
//! exactly as the hand-written `asset!()` tables put them there; only the
//! list of them is generated.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let assets =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets it")).join("assets");
    let mut files = Vec::new();
    walk(&assets, &assets, &mut files);
    // Sorted so the generated file — and every item order derived from it —
    // does not depend on the order the filesystem happens to hand back.
    files.sort();

    let mut generated = String::from("&[\n");
    for rel in &files {
        generated.push_str(&format!(
            "    ({rel:?}, include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/\", {rel:?}))),\n"
        ));
    }
    generated.push_str("]\n");

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets it")).join("pack_files.rs");
    fs::write(&out, generated).expect("OUT_DIR is writable");
}

/// Collect every file under `dir` as a path relative to `root`, forward-slashed
/// so the generated paths are the same on every platform.
///
/// Prints a `rerun-if-changed` line for each directory as well as each file:
/// cargo compares directory mtimes, and only the immediate parent's changes
/// when a file is added, so naming just the top of the tree would miss a new
/// concept three levels down.
fn walk(root: &Path, dir: &Path, files: &mut Vec<String>) {
    println!("cargo:rerun-if-changed={}", dir.display());
    let entries = fs::read_dir(dir).unwrap_or_else(|e| {
        panic!(
            "the embedded pack must be readable at {} — on Windows a checkout \
             without `core.symlinks=true` leaves assets/ as a text file ({e})",
            dir.display()
        )
    });
    for entry in entries {
        let path = entry.expect("readable dir entry").path();
        if path.is_dir() {
            walk(root, &path, files);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
            let rel = path
                .strip_prefix(root)
                .expect("walked from root")
                .to_str()
                .expect("pack paths are UTF-8")
                .replace('\\', "/");
            files.push(rel);
        }
    }
}

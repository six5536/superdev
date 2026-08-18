//! templates/rust_npm.rs — the rust-npm template: a Rust CLI workspace
//! deployed as prebuilt binaries through npm, derived from superdev's own
//! repo shape. On disk the assets follow the seeded layout with leading dots
//! stripped and tokenised segments written `_slug_` (see the module docs in
//! [`super`]); this table restores both in the target paths.

use super::Template;

macro_rules! tpl {
    ($rel:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/templates/rust-npm/",
            $rel
        ))
    };
}

/// (tokenised target path, embedded content), in write order.
const FILES: [(&str, &str); 41] = [
    ("CHANGELOG.md", tpl!("CHANGELOG.md")),
    ("CODE_OF_CONDUCT.md", tpl!("CODE_OF_CONDUCT.md")),
    ("CONTRIBUTING.md", tpl!("CONTRIBUTING.md")),
    ("Cargo.toml", tpl!("Cargo.toml")),
    ("LICENSE", tpl!("LICENSE")),
    ("README.md", tpl!("README.md")),
    ("SECURITY.md", tpl!("SECURITY.md")),
    (
        "crates/app/{{superdev:project-slug}}/Cargo.toml",
        tpl!("crates/app/_slug_/Cargo.toml"),
    ),
    (
        "crates/app/{{superdev:project-slug}}/src/main.rs",
        tpl!("crates/app/_slug_/src/main.rs"),
    ),
    (
        "crates/lib/{{superdev:project-slug}}-core/Cargo.toml",
        tpl!("crates/lib/_slug_-core/Cargo.toml"),
    ),
    (
        "crates/lib/{{superdev:project-slug}}-core/src/lib.rs",
        tpl!("crates/lib/_slug_-core/src/lib.rs"),
    ),
    ("deny.toml", tpl!("deny.toml")),
    (".gitattributes", tpl!("gitattributes")),
    (
        ".github/workflows/audit.yml",
        tpl!("github/workflows/audit.yml"),
    ),
    (
        ".github/workflows/checks.yml",
        tpl!("github/workflows/checks.yml"),
    ),
    (".github/workflows/ci.yml", tpl!("github/workflows/ci.yml")),
    (
        ".github/workflows/release.yml",
        tpl!("github/workflows/release.yml"),
    ),
    (".gitignore", tpl!("gitignore")),
    ("package.json", tpl!("package.json")),
    (
        "packages/{{superdev:project-slug}}-darwin-arm64/bin/.gitkeep",
        tpl!("packages/_slug_-darwin-arm64/bin/gitkeep"),
    ),
    (
        "packages/{{superdev:project-slug}}-darwin-arm64/package.json",
        tpl!("packages/_slug_-darwin-arm64/package.json"),
    ),
    (
        "packages/{{superdev:project-slug}}-darwin-x64/bin/.gitkeep",
        tpl!("packages/_slug_-darwin-x64/bin/gitkeep"),
    ),
    (
        "packages/{{superdev:project-slug}}-darwin-x64/package.json",
        tpl!("packages/_slug_-darwin-x64/package.json"),
    ),
    (
        "packages/{{superdev:project-slug}}-linux-arm64/bin/.gitkeep",
        tpl!("packages/_slug_-linux-arm64/bin/gitkeep"),
    ),
    (
        "packages/{{superdev:project-slug}}-linux-arm64/package.json",
        tpl!("packages/_slug_-linux-arm64/package.json"),
    ),
    (
        "packages/{{superdev:project-slug}}-linux-x64/bin/.gitkeep",
        tpl!("packages/_slug_-linux-x64/bin/gitkeep"),
    ),
    (
        "packages/{{superdev:project-slug}}-linux-x64/package.json",
        tpl!("packages/_slug_-linux-x64/package.json"),
    ),
    (
        "packages/{{superdev:project-slug}}-win32-x64/bin/.gitkeep",
        tpl!("packages/_slug_-win32-x64/bin/gitkeep"),
    ),
    (
        "packages/{{superdev:project-slug}}-win32-x64/package.json",
        tpl!("packages/_slug_-win32-x64/package.json"),
    ),
    (
        "packages/{{superdev:project-slug}}/bin/{{superdev:project-slug}}.js",
        tpl!("packages/_slug_/bin/_slug_.js"),
    ),
    (
        "packages/{{superdev:project-slug}}/lib/binary.js",
        tpl!("packages/_slug_/lib/binary.js"),
    ),
    (
        "packages/{{superdev:project-slug}}/package.json",
        tpl!("packages/_slug_/package.json"),
    ),
    (
        "packages/{{superdev:project-slug}}/test/binary.test.js",
        tpl!("packages/_slug_/test/binary.test.js"),
    ),
    (".prettierignore", tpl!("prettierignore")),
    ("rust-toolchain.toml", tpl!("rust-toolchain.toml")),
    ("rustfmt.toml", tpl!("rustfmt.toml")),
    (
        "scripts/launcher-smoke.mjs",
        tpl!("scripts/launcher-smoke.mjs"),
    ),
    (
        "scripts/release-smoke.mjs",
        tpl!("scripts/release-smoke.mjs"),
    ),
    ("scripts/release.mjs", tpl!("scripts/release.mjs")),
    ("scripts/set-version.mjs", tpl!("scripts/set-version.mjs")),
    (
        "scripts/verify-version.mjs",
        tpl!("scripts/verify-version.mjs"),
    ),
];

pub(super) const TEMPLATES: [Template; 1] = [Template {
    name: "rust-npm",
    description: "Rust CLI workspace deployed as prebuilt binaries through npm",
    files: &FILES,
}];

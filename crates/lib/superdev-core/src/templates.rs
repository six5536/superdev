//! templates.rs — project templates: embedded write-once repo scaffolds
//! `init` seeds a repo from. A template is purely files — capabilities and
//! providers stay with the init flags — and every file has scaffold
//! semantics: written only where absent, the user's from the moment it
//! exists, never hashed, never revisited by `sync`.
//!
//! Five tokens substitute in file contents and in target paths, exact-match
//! only; everything else — including GitHub Actions' `${{ … }}` — passes
//! through untouched. On disk the assets mirror the seeded repo, except for
//! three manglings the target paths restore: leading dots are stripped (a
//! real `.gitignore` would hide sibling assets from git), a tokenised path
//! segment is written `_slug_` or `_pascal_` because `:` cannot appear in
//! Windows file names, and a `Cargo.toml` is stored as `Cargo.toml.tpl`
//! because cargo omits any directory holding a real manifest from the
//! published crate, which would ship a Rust template with no files in it.

use std::path::Path;

use crate::action::{Action, Ownership};
use crate::engine::Planned;

/// Replaced by the project name as given (e.g. "My Tool").
pub const TOKEN_NAME: &str = "{{superdev:project-name}}";
/// Replaced by the kebab-case slug (e.g. "my-tool") — crate and package names.
pub const TOKEN_SLUG: &str = "{{superdev:project-slug}}";
/// The slug as a Rust identifier (e.g. "my_tool") — a hyphenated crate name
/// is referenced with underscores in source, which the slug cannot express.
pub const TOKEN_IDENT: &str = "{{superdev:project-ident}}";
/// The slug with the hyphens dropped (e.g. "mytool") — reverse-domain app
/// ids, where Android forbids `-` and iOS forbids `_`, so only alphanumeric
/// segments work on both.
pub const TOKEN_COMPACT: &str = "{{superdev:project-compact}}";
/// The slug in PascalCase (e.g. "MyTool") — Swift and Kotlin type names,
/// Xcode project/scheme names and Gradle root projects, none of which admit
/// a separator.
pub const TOKEN_PASCAL: &str = "{{superdev:project-pascal}}";

/// The values the tokens substitute to, recorded in the manifest afterwards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tokens {
    /// The project name as given.
    pub name: String,
    /// The kebab-case slug derived from it.
    pub slug: String,
}

impl Tokens {
    /// Tokens for a project name; the slug is derived.
    pub fn for_name(name: &str) -> Tokens {
        Tokens {
            name: name.to_string(),
            slug: slug_of(name),
        }
    }

    /// The slug as a Rust identifier: "my-tool" → "my_tool".
    pub fn ident(&self) -> String {
        self.slug.replace('-', "_")
    }

    /// The slug with the hyphens dropped: "my-tool" → "mytool".
    pub fn compact(&self) -> String {
        self.slug.replace('-', "")
    }

    /// The slug's segments capitalised and joined: "my-tool" → "MyTool".
    pub fn pascal(&self) -> String {
        self.slug
            .split('-')
            .map(|seg| {
                let mut chars = seg.chars();
                match chars.next() {
                    Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect()
    }
}

/// Lowercase kebab-case: alphanumerics kept, every other run becomes one
/// `-`, never leading or trailing. An unusable name falls back to "project".
pub fn slug_of(name: &str) -> String {
    let mut slug = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_end_matches('-');
    if slug.is_empty() {
        "project".to_string()
    } else {
        slug.to_string()
    }
}

/// One shipped project template.
#[derive(Debug)]
pub struct Template {
    /// The name `--template` and the prompt select by.
    pub name: &'static str,
    /// One line for the selection prompt.
    pub description: &'static str,
    /// (tokenised target path, embedded content) pairs.
    pub(crate) files: &'static [(&'static str, &'static str)],
}

/// Every template this binary ships, in prompt order.
static SHIPPED: [Template; 2] = [rust_npm::TEMPLATE, web_react_android_ios_native::TEMPLATE];

/// Every template this binary ships.
pub fn shipped() -> &'static [Template] {
    &SHIPPED
}

/// The shipped template with this name, if any.
pub fn find(name: &str) -> Option<&'static Template> {
    shipped().iter().find(|t| t.name == name)
}

/// The template rendered for these tokens: (target path, content) pairs,
/// substituted in both. The pure half of `template render` — callers decide
/// where (or whether) the tree lands on disk.
pub fn render(template: &Template, tokens: &Tokens) -> Vec<(String, String)> {
    template
        .files
        .iter()
        .map(|(target, content)| (substitute(target, tokens), substitute(content, tokens)))
        .collect()
}

/// Substitute the tokens, exact-match only.
pub fn substitute(text: &str, tokens: &Tokens) -> String {
    text.replace(TOKEN_NAME, &tokens.name)
        .replace(TOKEN_SLUG, &tokens.slug)
        .replace(TOKEN_IDENT, &tokens.ident())
        .replace(TOKEN_COMPACT, &tokens.compact())
        .replace(TOKEN_PASCAL, &tokens.pascal())
}

/// The template's diff against the repo: one scaffold write per absent
/// target, and one report line per target that already exists and is kept.
/// Init-only by construction — no verb ever re-plans a template.
pub fn plan(root: &Path, template: &Template, tokens: &Tokens) -> (Planned, Vec<String>) {
    let mut actions = Vec::new();
    let mut kept = Vec::new();
    for (target, content) in template.files {
        let path = substitute(target, tokens);
        if root.join(&path).exists() {
            kept.push(format!(
                "template {}: kept {path} — already exists",
                template.name
            ));
        } else {
            actions.push(Action::WriteFile {
                path,
                content: substitute(content, tokens),
                ownership: Ownership::Scaffold,
                reason: format!("{} template", template.name),
            });
        }
    }
    (
        Planned {
            capability: None,
            provider: format!("template:{}", template.name),
            actions,
        },
        kept,
    )
}

mod rust_npm;
mod web_react_android_ios_native;

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens() -> Tokens {
        Tokens::for_name("My Tool")
    }

    #[test]
    fn slugs_are_kebab_case_with_a_fallback() {
        assert_eq!(slug_of("My Tool"), "my-tool");
        assert_eq!(slug_of("  weird__Name 2.0! "), "weird-name-2-0");
        assert_eq!(slug_of("already-a-slug"), "already-a-slug");
        assert_eq!(slug_of("???"), "project");
    }

    #[test]
    fn substitution_is_exact_match_only() {
        let tokens = tokens();
        assert_eq!(
            substitute(
                "# {{superdev:project-name}}\nbin: {{superdev:project-slug}}\nuse {{superdev:project-ident}}_core;\nid: com.{{superdev:project-compact}}.app\nscheme: {{superdev:project-pascal}}",
                &tokens
            ),
            "# My Tool\nbin: my-tool\nuse my_tool_core;\nid: com.mytool.app\nscheme: MyTool"
        );
        // GitHub Actions syntax and near-miss tokens pass through untouched.
        let untouched = "${{ secrets.TOKEN }} {{superdev:project}} {{name}}";
        assert_eq!(substitute(untouched, &tokens), untouched);
    }

    /// Cargo omits any directory holding a `Cargo.toml` from the packaged
    /// crate, treating it as a nested package. A template manifest stored
    /// under its real name therefore drops that whole template from the
    /// tarball, and the published crate fails to compile on `include_str!` —
    /// invisibly, since building from the repo still works. Assets keep
    /// manifests as `Cargo.toml.tpl`; this pins the rule.
    #[test]
    fn no_template_asset_is_named_cargo_toml() {
        fn walk(dir: &Path, found: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(dir).expect("assets dir is readable") {
                let path = entry.expect("readable dir entry").path();
                if path.is_dir() {
                    walk(&path, found);
                } else if path.file_name().is_some_and(|name| name == "Cargo.toml") {
                    found.push(path);
                }
            }
        }

        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/projects");
        let mut found = Vec::new();
        walk(&assets, &mut found);
        assert!(
            found.is_empty(),
            "template assets must store manifests as `Cargo.toml.tpl`, \
             or cargo strips them from the published crate; found: {found:?}"
        );
    }

    const TEST_TEMPLATE: Template = Template {
        name: "test",
        description: "test template",
        files: &[
            ("README.md", "# {{superdev:project-name}}\n"),
            (
                "crates/app/{{superdev:project-slug}}/Cargo.toml",
                "name = \"{{superdev:project-slug}}\"\n",
            ),
        ],
    };

    #[test]
    fn render_substitutes_paths_and_contents_without_touching_disk() {
        let rendered = render(&TEST_TEMPLATE, &tokens());
        assert_eq!(
            rendered,
            vec![
                ("README.md".to_string(), "# My Tool\n".to_string()),
                (
                    "crates/app/my-tool/Cargo.toml".to_string(),
                    "name = \"my-tool\"\n".to_string()
                ),
            ]
        );
    }

    #[test]
    fn plans_absent_files_with_tokenised_paths_and_keeps_existing_ones() {
        let dir = tempfile::tempdir().unwrap();
        let (planned, kept) = plan(dir.path(), &TEST_TEMPLATE, &tokens());
        assert!(kept.is_empty());
        assert_eq!(planned.provider, "template:test");
        assert!(planned.capability.is_none());
        let descs: Vec<String> = planned.actions.iter().map(Action::describe).collect();
        assert_eq!(descs.len(), 2, "{descs:?}");
        assert!(
            descs[1].contains("crates/app/my-tool/Cargo.toml"),
            "{descs:?}"
        );

        // An existing target drops out of the plan and is reported kept.
        std::fs::write(dir.path().join("README.md"), "mine").unwrap();
        let (planned, kept) = plan(dir.path(), &TEST_TEMPLATE, &tokens());
        assert_eq!(planned.actions.len(), 1);
        assert_eq!(kept, vec!["template test: kept README.md — already exists"]);
    }

    #[test]
    fn shipped_templates_resolve_by_name() {
        assert!(find("rust-npm").is_some());
        assert!(find("flying").is_none());
        assert!(shipped().iter().all(|t| !t.description.is_empty()));
    }

    /// The disjointness rule from the spec: no template target may overlap a
    /// path any capability claims or scaffolds. Collisions are impossible
    /// rather than resolved, and this test is what enforces it — in both
    /// directions: the static components' writes and claims must all fall
    /// under the reserved prefixes, and no template target may. A component
    /// that grows a file outside the prefixes fails here, forcing the list
    /// (and the rule) to be updated together. `EnsureLine` targets are
    /// exempt: shared-line files merge, so a template may ship the file the
    /// lines land in. The dynamic providers (mattskills, codegraph) write
    /// only under `.claude/` and `.mise.toml` by construction.
    #[test]
    fn shipped_template_paths_are_disjoint_from_capability_files() {
        const RESERVED: &[&str] = &[
            "AGENTS.md",
            "CLAUDE.md",
            ".agents/",
            ".claude/",
            ".superdev/",
            ".mise.toml",
            ".mcp.json",
            "knowledge/",
        ];
        let covered = |path: &str| RESERVED.iter().any(|r| path == *r || path.starts_with(r));

        let dir = tempfile::tempdir().unwrap();
        let manifest = crate::manifest::Manifest::default_for("0.1.0", &[]);
        let lock = crate::lock::Lock::default();
        let fake = crate::runner::FakeRunner::new();
        let ctx = crate::component::Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
            content: crate::content::test_snapshot(),
        };
        use crate::component::{Claim, Component};
        let statics: [Box<dyn Component>; 2] = [
            Box::new(crate::components::aokf::Aokf),
            Box::new(crate::components::skillpack::SkillPack),
        ];
        for component in &statics {
            for action in component.plan(&ctx).unwrap() {
                if let Action::WriteFile { path, .. } = action {
                    assert!(covered(&path), "unreserved component write: {path}");
                }
            }
            for claim in component.owned(&ctx) {
                let path = match claim {
                    Claim::File(path) => path,
                    Claim::JsonKey { path, .. } => path,
                    Claim::MisePin(_) => continue,
                };
                assert!(covered(&path), "unreserved component claim: {path}");
            }
        }

        for template in shipped() {
            for (target, _) in template.files {
                assert!(
                    !covered(target),
                    "{}: {target} overlaps a capability path",
                    template.name
                );
            }
        }
    }

    #[test]
    fn shipped_template_contents_substitute_cleanly() {
        // Every token in every shipped file is one of the real tokens: a
        // typo'd token would survive substitution and leak into a seeded repo.
        let tokens = Tokens::for_name("demo");
        for template in shipped() {
            for (target, content) in template.files {
                for text in [*target, &substitute(content, &tokens) as &str] {
                    let text = substitute(text, &tokens);
                    assert!(
                        !text.contains("{{superdev:"),
                        "{}: {target} carries an unknown superdev token",
                        template.name
                    );
                }
            }
        }
    }
}

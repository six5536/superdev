---
type: Plan
id: plan-blueprint-migrations
title: Blueprint Migrations Implementation Plan
description: Task-by-task plan for the orphan pass, the removal actions, the CLAUDE.md import line, and blueprint-version stamping.
status: draft
links:
  - rel: implements
    to: spec-blueprint-migrations
    note: Edges declared plan-side only, so deleting this plan leaves no dangling references.
---

# Blueprint Migrations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the [blueprint migrations spec](/knowledge/specs/2026-08-12-blueprint-migrations-design.md): components declare what they own, the lock's leftovers are pruned or released, `CLAUDE.md` imports `AGENTS.md`, and `blueprint` becomes the version last applied.

**Architecture:** A new `Claim` type and `Component::owned()` let a repo-level orphan pass subtract live claims from the lock. Three new `Action` variants carry removals through the existing engine (backup, journal, unwind). `manage.rs` plans the orphan entry last, drops released/gone keys from the lock, and stamps the blueprint version on a successful sync.

**Tech Stack:** Rust (existing workspace only — `toml_edit`, `serde_json`, `sha2`, `tempfile`, `assert_cmd`). No new dependencies.

## Global Constraints

- No new dependencies. If a task seems to need one, stop and ask.
- Lock shape decision (spec left it open): keep the flat `files` map with shape-classified keys. `.mise.toml:<tool>` is a pin, any other key containing `:` is a JSON key, everything else is a file path. Zero migration; a 0.1.0 lock reads unchanged.
- Prose rules in `.agents/PROSE.md` apply to every comment, doc line, and commit message. Comments say why, never what.
- Exact user-facing strings (used across tasks and tests):
  - skip reasons: `already gone` and `changed since superdev wrote it — left in place`
  - released report line: `orphan: <key> changed since superdev wrote it — left in place, released from the lock`
  - blueprint report line: `blueprint <manifest>, binary <binary> — sync will update it`
  - removal reason: `no longer in the blueprint`
  - CLAUDE.md line: `@AGENTS.md`, reason `make Claude Code read AGENTS.md`
- Verify with `cargo nextest run --workspace` (the repo's runner; plain `cargo test -p <crate>` is fine mid-task). Before each commit: `npm run fmt` and `npm run lint` must be clean.
- After any change under `knowledge/`: `npm run check:aokf` must PASS at level 2 (a PostToolUse hook enforces this on Edit/Write).
- Never stage or revert `.claude/settings.json`, `.devcontainer/devcontainer.json`, `.mise.toml`, `.entire/`, `.DS_Store` — they carry the user's uncommitted local state.
- Commits: conventional prefix (`feat:`/`fix:`/`test:`/`docs:`), no Claude signature. Sign with `-S`; if the SSH signing agent fails, fall back to `--no-gpg-sign` (re-signed in batch later).
- Windows-safe paths: repo-relative, forward slashes, never a bare `:` in a file path superdev writes.

---

### Task 1: `Claim` and `Component::owned()`

Every component declares what it owns in the lock, whether or not it needs changing. This cannot be derived from `plan()`: a converged repo plans nothing, so an in-sync repo would look entirely orphaned.

**Files:**
- Modify: `crates/lib/superdev-core/src/component.rs`
- Modify: `crates/lib/superdev-core/src/components/plugin.rs`
- Modify: `crates/lib/superdev-core/src/components/codegraph.rs`
- Modify: `crates/lib/superdev-core/src/components/skillpack.rs`
- Modify: `crates/lib/superdev-core/src/components/aokf.rs`
- Test: `crates/lib/superdev-core/src/components/mod.rs` (new cross-component test)

**Interfaces:**
- Produces: `pub enum Claim { File(String), MisePin(String), JsonKey { path: String, pointer: String } }` with `pub fn lock_key(&self) -> String`, in `superdev_core::component`. A `JsonKey` pointer is dotted, with an optional trailing `[marker]` naming an array element (the lock-key encoding the engine already writes).
- Produces: `fn owned(&self, ctx: &Ctx<'_>) -> Vec<Claim>` required on the `Component` trait.
- Consumes: existing constants — `SKILLS`, `SETTINGS_PATH`, `HOOK_POINTER`, `HOOK_MARKER` (skillpack); `FILES`, `MCP_PATH`, `MCP_POINTER` (aokf); `CODEGRAPH_MISE_TOOL`; `Marketplace` (plugin).

- [ ] **Step 1: Write the failing cross-component test**

In `components/mod.rs` tests. It is the strongest possible statement of the contract: for every component, applying a fresh plan produces exactly the lock keys `owned()` claims.

```rust
#[test]
fn owned_matches_what_apply_locks() {
    use std::collections::BTreeSet;
    use crate::component::{Claim, Ctx};
    use crate::engine::{self, Planned};
    use crate::lock::Lock;
    use crate::runner::FakeRunner;

    let manifest = Manifest::default_for(env!("CARGO_PKG_VERSION"), &[]);
    for component in enabled(&manifest) {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        let empty = Lock::default();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &empty,
        };
        let planned = vec![Planned {
            capability: Some(component.capability()),
            provider: component.provider().to_string(),
            actions: component.plan(&ctx).unwrap(),
        }];
        let mut lock = Lock::default();
        let result = engine::apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{}: apply failed", component.provider());
        let claimed: BTreeSet<String> =
            component.owned(&ctx).iter().map(Claim::lock_key).collect();
        let locked: BTreeSet<String> = lock.files.keys().cloned().collect();
        assert_eq!(claimed, locked, "{}", component.provider());
    }
}
```

Also add to `skillpack.rs` tests (custom skills leave `owned()`):

```rust
#[test]
fn owned_omits_custom_skills_but_keeps_the_hook() {
    use crate::component::Claim;
    let dir = tempfile::tempdir().unwrap();
    let (mut manifest, lock) = ctx_parts();
    manifest.capabilities.get_mut("skills").unwrap().custom = vec!["humanise".into()];
    let fake = FakeRunner::new();
    let ctx = Ctx {
        root: dir.path(),
        runner: &fake,
        manifest: &manifest,
        lock: &lock,
    };
    let keys: Vec<String> = SkillPack.owned(&ctx).iter().map(Claim::lock_key).collect();
    assert!(!keys.iter().any(|k| k.contains("humanise")), "{keys:?}");
    assert!(keys.contains(&".claude/skills/grill-me/SKILL.md".to_string()));
    assert!(keys.contains(
        &".claude/settings.json:hooks.PostToolUse[superdev aokf hook validate]".to_string()
    ));
}
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev-core owned_` — expected: compile error, `owned` and `Claim` do not exist.

- [ ] **Step 3: Implement `Claim` and the trait method**

In `component.rs`:

```rust
/// One thing a component owns in a managed repo: the typed form of a lock
/// `files` key. The orphan pass subtracts these from the lock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Claim {
    /// A whole superdev-owned file, by repo-relative path.
    File(String),
    /// A managed `[tools]` key in `.mise.toml`.
    MisePin(String),
    /// A managed key in a shared JSON file. `pointer` is dotted, with an
    /// optional trailing `[marker]` naming an array element.
    JsonKey {
        /// Repo-relative file path.
        path: String,
        /// Dotted key path, e.g. `mcpServers.superdev-aokf`.
        pointer: String,
    },
}

impl Claim {
    /// The lock `files` key this claim covers.
    pub fn lock_key(&self) -> String {
        match self {
            Claim::File(path) => path.clone(),
            Claim::MisePin(tool) => crate::components::mise::pin_lock_key(tool),
            Claim::JsonKey { path, pointer } => format!("{path}:{pointer}"),
        }
    }
}
```

On the trait (and an empty-vec impl on the `Nop` test component in this file):

```rust
    /// Everything this component owns in a managed repo, whether or not it
    /// needs changing. Derived from the same constants `plan` writes from —
    /// a converged repo plans nothing, so `plan` output cannot answer this.
    fn owned(&self, ctx: &Ctx<'_>) -> Vec<Claim>;
```

- [ ] **Step 4: Implement `owned()` on the four components**

`plugin.rs` — only a mise-pinned marketplace owns lock state; a plugin install is machine state, never locked:

```rust
    fn owned(&self, _ctx: &Ctx<'_>) -> Vec<Claim> {
        match &self.marketplace {
            Marketplace::MiseTool { tool, .. } => vec![Claim::MisePin((*tool).to_string())],
            Marketplace::GitHub { .. } => Vec::new(),
        }
    }
```

`codegraph.rs` — the pin only; `.codegraph/` is gitignored machine state:

```rust
    fn owned(&self, _ctx: &Ctx<'_>) -> Vec<Claim> {
        vec![Claim::MisePin(CODEGRAPH_MISE_TOOL.to_string())]
    }
```

`skillpack.rs` — non-custom skills plus the settings hook entry:

```rust
    fn owned(&self, ctx: &Ctx<'_>) -> Vec<Claim> {
        let custom = ctx
            .config(Capability::Skills)
            .map(|c| c.custom.as_slice())
            .unwrap_or_default();
        let mut claims: Vec<Claim> = SKILLS
            .iter()
            .filter(|(name, _)| !custom.iter().any(|c| c == name))
            .map(|(name, _)| Claim::File(format!(".claude/skills/{name}/SKILL.md")))
            .collect();
        claims.push(Claim::JsonKey {
            path: SETTINGS_PATH.into(),
            pointer: format!("{HOOK_POINTER}[{HOOK_MARKER}]"),
        });
        claims
    }
```

`aokf.rs` — the `Owned` files only (scaffolds are never hashed into the lock) plus the MCP registration key:

```rust
    fn owned(&self, _ctx: &Ctx<'_>) -> Vec<Claim> {
        let mut claims: Vec<Claim> = FILES
            .iter()
            .filter(|(_, _, ownership, _)| *ownership == Ownership::Owned)
            .map(|(path, ..)| Claim::File((*path).to_string()))
            .collect();
        claims.push(Claim::JsonKey {
            path: MCP_PATH.into(),
            pointer: MCP_POINTER.into(),
        });
        claims
    }
```

Add `use crate::component::Claim;` (and `Capability` where missing) to each file's imports.

- [ ] **Step 5: Run the workspace tests** — `cargo test -p superdev-core` — expected: PASS (every existing `Component` impl now compiles with `owned`; the two new tests pass).

- [ ] **Step 6: Commit** — `git add crates/lib/superdev-core/src/component.rs crates/lib/superdev-core/src/components/ && git commit -S -m "feat(core): components declare the lock entries they own"`

---

### Task 2: Removal primitives — actions, `mise::remove_pin`, JSON pointer helpers

Pure data and pure functions only; the engine wires them up in Task 3.

**Files:**
- Modify: `crates/lib/superdev-core/src/action.rs`
- Modify: `crates/lib/superdev-core/src/components/mise.rs`
- Modify: `crates/lib/superdev-core/src/engine.rs` (pure helper fns + tests only)

**Interfaces:**
- Produces: `Action::RemoveFile { path: String, reason: String }`, `Action::RemoveMisePin { tool: String }`, `Action::RemoveJsonKey { path: String, pointer: String }`; `describe()` renders `remove <path> (<reason>)`, `unpin <tool> in .mise.toml`, `remove <pointer> from <path>`.
- Produces: `pub fn remove_pin(mise_toml: &str, tool: &str) -> Result<Option<String>>` in `components::mise` — `Ok(None)` when the tool is not pinned; an emptied `[tools]` table stays.
- Produces, in `engine.rs`, `pub(crate)`:
  - `fn parse_pointer(pointer: &str) -> (Vec<&str>, Option<&str>)` — `a.b` → `(["a","b"], None)`; `a.b[m]` → `(["a","b"], Some("m"))`.
  - `fn json_value_at(path: &str, json: &str, pointer: &str) -> Result<Option<String>>` — the canonical value text at the pointer (object key, or the array element whose serialised form contains the marker — same rule as `edit_json_array_element`); `Ok(None)` when absent; `Err` on malformed JSON.
  - `fn remove_json_pointer(path: &str, json: &str, pointer: &str) -> Result<Option<(String, String)>>` — `(new file content, removed canonical value)`; `Ok(None)` when absent; empty parents stay.
  - Also make the existing `fn read_text` `pub(crate)` (Task 4 needs it).

- [ ] **Step 1: Write the failing tests**

`action.rs` — extend `describe_names_the_target`:

```rust
        let a = Action::RemoveFile {
            path: ".claude/skills/humanise/SKILL.md".into(),
            reason: "no longer in the blueprint".into(),
        };
        assert_eq!(
            a.describe(),
            "remove .claude/skills/humanise/SKILL.md (no longer in the blueprint)"
        );
        let a = Action::RemoveMisePin { tool: "http:codegraph".into() };
        assert_eq!(a.describe(), "unpin http:codegraph in .mise.toml");
        let a = Action::RemoveJsonKey {
            path: ".mcp.json".into(),
            pointer: "mcpServers.superdev-aokf".into(),
        };
        assert_eq!(a.describe(), "remove mcpServers.superdev-aokf from .mcp.json");
```

`mise.rs`:

```rust
    #[test]
    fn remove_pin_takes_one_key_and_leaves_the_rest() {
        let with = set_pin(SAMPLE, "http:codegraph", "\"1.5.0\"").unwrap();
        let out = remove_pin(&with, "http:codegraph").unwrap().unwrap();
        assert_eq!(current_pin(&out, "http:codegraph").unwrap(), None);
        assert!(out.contains("# my tools"));
        assert!(out.contains("node = \"24\" # keep"));
        // Not pinned: nothing to write.
        assert!(remove_pin(SAMPLE, "http:codegraph").unwrap().is_none());
        // The emptied [tools] table stays.
        let only = set_pin("", "http:codegraph", "\"1.5.0\"").unwrap();
        let out = remove_pin(&only, "http:codegraph").unwrap().unwrap();
        assert!(out.contains("[tools]"), "{out}");
        // Malformed file: an error, never a guess.
        assert!(remove_pin("[tools\n", "http:codegraph").is_err());
    }
```

`engine.rs` tests:

```rust
    #[test]
    fn pointers_parse_navigate_and_remove() {
        assert_eq!(parse_pointer("a.b"), (vec!["a", "b"], None));
        assert_eq!(
            parse_pointer("hooks.PostToolUse[superdev aokf hook validate]"),
            (vec!["hooks", "PostToolUse"], Some("superdev aokf hook validate"))
        );

        let json = r#"{"mcpServers":{"superdev-aokf":{"command":"superdev"},"mine":{}}}"#;
        let value = json_value_at("f", json, "mcpServers.superdev-aokf").unwrap().unwrap();
        assert!(value.contains("superdev"));
        assert_eq!(json_value_at("f", json, "mcpServers.gone").unwrap(), None);
        assert!(json_value_at("f", "not json", "a").is_err());

        let (content, removed) =
            remove_json_pointer("f", json, "mcpServers.superdev-aokf").unwrap().unwrap();
        assert!(removed.contains("superdev"));
        let root: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(root["mcpServers"].get("superdev-aokf").is_none());
        // The user's key and the (possibly emptied) parent survive.
        assert!(root["mcpServers"].get("mine").is_some());
        assert_eq!(remove_json_pointer("f", json, "mcpServers.gone").unwrap(), None);

        let hooks = r#"{"hooks":{"PostToolUse":[{"matcher":"Agent","hooks":[]},{"matcher":"Edit|Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}]}}"#;
        let pointer = "hooks.PostToolUse[superdev aokf hook validate]";
        assert!(json_value_at("f", hooks, pointer).unwrap().unwrap().contains("Edit|Write"));
        let (content, _) = remove_json_pointer("f", hooks, pointer).unwrap().unwrap();
        let root: serde_json::Value = serde_json::from_str(&content).unwrap();
        let items = root["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(items.len(), 1, "only superdev's element goes");
        assert_eq!(items[0]["matcher"], "Agent");
    }
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev-core remove_pin_takes pointers_parse describe_names` — expected: compile errors on the new names.

- [ ] **Step 3: Implement**

`action.rs` variants (with doc comments in the file's style) and `describe()` arms as specified in Interfaces.

`mise.rs`:

```rust
/// Remove one `[tools]` key, preserving everything else. `None` when the tool
/// is not pinned. An emptied `[tools]` table stays: guessing which empty
/// containers a user wants gone is worse than the residue.
pub fn remove_pin(mise_toml: &str, tool: &str) -> Result<Option<String>> {
    let mut doc = parse(mise_toml)?;
    let removed = doc
        .get_mut("tools")
        .and_then(Item::as_table_like_mut)
        .and_then(|tools| tools.remove(tool));
    Ok(removed.map(|_| doc.to_string()))
}
```

`engine.rs`:

```rust
/// Split a lock-style pointer into dotted segments and the optional trailing
/// `[marker]` naming an array element.
pub(crate) fn parse_pointer(pointer: &str) -> (Vec<&str>, Option<&str>) {
    match pointer.split_once('[') {
        Some((dotted, rest)) => (
            dotted.split('.').collect(),
            Some(rest.strip_suffix(']').unwrap_or(rest)),
        ),
        None => (pointer.split('.').collect(), None),
    }
}

/// The canonical value text at `pointer`: the object key's value, or the
/// array element whose serialised form contains the marker — the same rule
/// `edit_json_array_element` matches by. `Ok(None)` when absent.
pub(crate) fn json_value_at(path: &str, json: &str, pointer: &str) -> Result<Option<String>> {
    let root: serde_json::Value = serde_json::from_str(json).map_err(|e| Error::Toml {
        path: path.into(),
        message: e.to_string(),
    })?;
    let (segments, marker) = parse_pointer(pointer);
    let mut cursor = &root;
    for segment in segments {
        match cursor.get(segment) {
            Some(next) => cursor = next,
            None => return Ok(None),
        }
    }
    let value = match marker {
        None => Some(cursor),
        Some(marker) => cursor
            .as_array()
            .and_then(|items| items.iter().find(|item| item.to_string().contains(marker))),
    };
    Ok(value.map(ToString::to_string))
}

/// Remove the entry `pointer` names. Returns the new file content and the
/// removed canonical value; `Ok(None)` when absent. Empty parents stay.
pub(crate) fn remove_json_pointer(
    path: &str,
    json: &str,
    pointer: &str,
) -> Result<Option<(String, String)>> {
    let bad = |message: String| Error::Toml {
        path: path.into(),
        message,
    };
    let mut root: serde_json::Value = serde_json::from_str(json).map_err(|e| bad(e.to_string()))?;
    let (mut segments, marker) = parse_pointer(pointer);
    let last = if marker.is_none() { segments.pop() } else { None };
    let mut cursor = &mut root;
    for segment in segments {
        match cursor.get_mut(segment) {
            Some(next) => cursor = next,
            None => return Ok(None),
        }
    }
    let removed = match (last, marker) {
        (Some(key), None) => cursor.as_object_mut().and_then(|map| map.remove(key)),
        (_, Some(marker)) => cursor.as_array_mut().and_then(|items| {
            let index = items
                .iter()
                .position(|item| item.to_string().contains(marker))?;
            Some(items.remove(index))
        }),
        (None, None) => None,
    };
    Ok(removed.map(|value| {
        let mut content =
            serde_json::to_string_pretty(&root).expect("a parsed value re-serialises");
        content.push('\n');
        (content, value.to_string())
    }))
}
```

Change `fn read_text` to `pub(crate) fn read_text`. In `mise.rs`, ensure `Item` is imported (it already is for `set_pin`).

- [ ] **Step 4: Run the crate tests** — `cargo test -p superdev-core` — expected: PASS. New match arms in `apply_entry` are NOT added yet; the compiler will demand them — add temporary arms that `unreachable!("wired in the next commit")`? No: non-exhaustive matches are hard errors. Add the three arms to `apply_entry`'s match now, each returning `ActionOutcome::Failed("removal actions are wired in the next commit".into())` — Task 3 replaces them. (`report.rs` and everything else match on `Action` only via `describe()`.)

- [ ] **Step 5: Commit** — `git add crates/lib/superdev-core/src/action.rs crates/lib/superdev-core/src/components/mise.rs crates/lib/superdev-core/src/engine.rs && git commit -S -m "feat(core): removal actions and the pin/JSON removal primitives"`

---

### Task 3: Engine executes removals

Backup, journal, unwind — and the lock key is released on skip as well as on success: gone or user-changed, the target is no longer superdev's.

**Files:**
- Modify: `crates/lib/superdev-core/src/engine.rs`

**Interfaces:**
- Consumes: Task 2's action variants and helpers; `Session::prior_hashes` (the lock's hashes at run start); `BACKUP_DIR`; `Undo::RestoreFile`.
- Produces: `apply()` executes the three removal actions. Outcomes: `Applied` (backed up + journalled + removed), `Skipped("already gone")`, `Skipped("changed since superdev wrote it — left in place")` — the re-check against `prior_hashes` at apply time, so a file edited between plan and apply is never deleted. On `Applied` and both `Skipped`s the entry's completion removes the lock key; a `Failed` outcome aborts the entry before any lock change, as everywhere else.

- [ ] **Step 1: Write the failing engine tests**

```rust
    #[test]
    fn remove_file_backs_up_journals_and_releases_the_lock_key() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("old.txt"), "superdev content").unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        lock.files
            .insert("old.txt".into(), sha256_hex(b"superdev content"));
        let planned = vec![Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![Action::RemoveFile {
                path: "old.txt".into(),
                reason: "no longer in the blueprint".into(),
            }],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        assert!(!dir.path().join("old.txt").exists());
        assert!(!lock.files.contains_key("old.txt"));
        let backups: Vec<_> = std::fs::read_dir(dir.path().join(BACKUP_DIR))
            .unwrap()
            .map(|e| e.unwrap().path().join("old.txt"))
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(
            std::fs::read_to_string(&backups[0]).unwrap(),
            "superdev content"
        );
    }

    #[test]
    fn remove_file_skips_the_gone_and_the_user_changed() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let remove = |path: &str| Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![Action::RemoveFile {
                path: path.into(),
                reason: "no longer in the blueprint".into(),
            }],
        };
        // Already gone: skipped, key released.
        let mut lock = Lock::default();
        lock.files.insert("gone.txt".into(), sha256_hex(b"x"));
        let result = apply(dir.path(), &fake, &manifest, &[remove("gone.txt")], &mut lock);
        assert!(result.ok);
        assert_eq!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Skipped("already gone".into())
        );
        assert!(!lock.files.contains_key("gone.txt"));
        // Changed since planning: left in place, key released.
        std::fs::write(dir.path().join("mine.txt"), "edited by hand").unwrap();
        let mut lock = Lock::default();
        lock.files.insert("mine.txt".into(), sha256_hex(b"superdev content"));
        let result = apply(dir.path(), &fake, &manifest, &[remove("mine.txt")], &mut lock);
        assert!(result.ok);
        assert_eq!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Skipped("changed since superdev wrote it — left in place".into())
        );
        assert!(dir.path().join("mine.txt").exists());
        assert!(!lock.files.contains_key("mine.txt"));
    }

    #[test]
    fn remove_mise_pin_and_json_key_rewrite_only_their_entry() {
        let dir = tempfile::tempdir().unwrap();
        let mise_toml =
            mise::set_pin("[tools]\nnode = \"24\"\n", "http:codegraph", "\"1.5.0\"").unwrap();
        std::fs::write(dir.path().join(".mise.toml"), &mise_toml).unwrap();
        std::fs::write(
            dir.path().join(".mcp.json"),
            r#"{"mcpServers":{"superdev-aokf":{"command":"superdev","args":["mcp","aokf"]},"mine":{"command":"me"}}}"#,
        )
        .unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let pin_value = mise::current_pin(&mise_toml, "http:codegraph").unwrap().unwrap();
        lock.files.insert(
            mise::pin_lock_key("http:codegraph"),
            sha256_hex(pin_value.as_bytes()),
        );
        let mcp_value: serde_json::Value =
            serde_json::from_str(r#"{"command":"superdev","args":["mcp","aokf"]}"#).unwrap();
        lock.files.insert(
            ".mcp.json:mcpServers.superdev-aokf".into(),
            sha256_hex(mcp_value.to_string().as_bytes()),
        );
        let planned = vec![Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![
                Action::RemoveMisePin { tool: "http:codegraph".into() },
                Action::RemoveJsonKey {
                    path: ".mcp.json".into(),
                    pointer: "mcpServers.superdev-aokf".into(),
                },
            ],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{:?}", result.reports);
        let mise_after = std::fs::read_to_string(dir.path().join(".mise.toml")).unwrap();
        assert_eq!(mise::current_pin(&mise_after, "http:codegraph").unwrap(), None);
        assert!(mise_after.contains("node = \"24\""));
        let mcp: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap(),
        )
        .unwrap();
        assert!(mcp["mcpServers"].get("superdev-aokf").is_none());
        assert_eq!(mcp["mcpServers"]["mine"]["command"], "me");
        assert!(lock.files.is_empty());
        // No installs follow a removal.
        assert!(fake.calls().is_empty());
    }

    #[test]
    fn a_later_failure_restores_a_removed_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("old.txt"), "superdev content").unwrap();
        let fake = FakeRunner::new();
        fake.script(
            "codegraph init",
            Output { status: 1, stdout: String::new(), stderr: "boom".into() },
        );
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        lock.files
            .insert("old.txt".into(), sha256_hex(b"superdev content"));
        let planned = vec![
            Planned {
                capability: None,
                provider: "superdev".into(),
                actions: vec![Action::RemoveFile {
                    path: "old.txt".into(),
                    reason: "no longer in the blueprint".into(),
                }],
            },
            Planned {
                capability: Some(crate::capability::Capability::CodeIndex),
                provider: "codegraph".into(),
                actions: vec![Action::Run {
                    program: "codegraph".into(),
                    args: vec!["init".into()],
                    purpose: "index".into(),
                    undo: None,
                    optional: false,
                }],
            },
        ];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("old.txt")).unwrap(),
            "superdev content"
        );
        assert!(result.reverted.iter().any(|r| r.contains("old.txt")));
    }
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev-core remove_ a_later_failure` — expected: FAIL through the placeholder `Failed` arms from Task 2.

- [ ] **Step 3: Implement the three `Session` methods and wire `apply_entry`**

In `apply_entry`, add `let mut removed: Vec<String> = Vec::new();` beside `written`, replace the placeholder arms:

```rust
                Action::RemoveFile { path, .. } => self.remove_file(path, &mut removed),
                Action::RemoveMisePin { tool } => self.remove_mise_pin(tool, &mut removed),
                Action::RemoveJsonKey { path, pointer } => {
                    self.remove_json_key(path, pointer, &mut removed)
                }
```

and after the existing lock-insert loop:

```rust
        for key in removed {
            lock.files.remove(&key);
        }
```

The methods (each pushes its lock key first — a `Failed` outcome aborts the entry before the removal loop runs, so the push is inert on failure):

```rust
    /// Remove an owned file the blueprint dropped. The lock key is released
    /// even when the removal is skipped: gone or user-changed, the file is no
    /// longer superdev's.
    fn remove_file(&mut self, path: &str, removed: &mut Vec<String>) -> ActionOutcome {
        removed.push(path.to_string());
        let full = self.root.join(path);
        let existing = match read_text(&full) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let Some(content) = existing else {
            return ActionOutcome::Skipped("already gone".into());
        };
        // Re-check at apply time: an edit between plan and apply is the
        // user's, and superdev takes back only what it wrote.
        if self.prior_hashes.get(path) != Some(&sha256_hex(content.as_bytes())) {
            return ActionOutcome::Skipped(
                "changed since superdev wrote it — left in place".into(),
            );
        }
        let backup = self
            .root
            .join(BACKUP_DIR)
            .join(self.stamp.to_string())
            .join(path);
        if let Err(e) = write_file(&backup, &content) {
            return ActionOutcome::Failed(e.to_string());
        }
        self.journal.push(Undo::RestoreFile {
            path: path.to_string(),
            prior: Some(content),
        });
        match fs::remove_file(&full) {
            Ok(()) => ActionOutcome::Applied { note: None },
            Err(e) => ActionOutcome::Failed(e.to_string()),
        }
    }

    /// Remove a managed pin. Journals the whole file: removing a pin is a
    /// file rewrite.
    fn remove_mise_pin(&mut self, tool: &str, removed: &mut Vec<String>) -> ActionOutcome {
        let key = mise::pin_lock_key(tool);
        removed.push(key.clone());
        let path = self.root.join(".mise.toml");
        let existing = match read_text(&path) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let Some(content) = existing else {
            return ActionOutcome::Skipped("already gone".into());
        };
        let value = match mise::current_pin(&content, tool) {
            Ok(Some(value)) => value,
            Ok(None) => return ActionOutcome::Skipped("already gone".into()),
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        if self.prior_hashes.get(&key) != Some(&sha256_hex(value.as_bytes())) {
            return ActionOutcome::Skipped(
                "changed since superdev wrote it — left in place".into(),
            );
        }
        let next = match mise::remove_pin(&content, tool) {
            Ok(next) => next.expect("the pin is present"),
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        self.journal.push(Undo::RestoreFile {
            path: ".mise.toml".into(),
            prior: Some(content),
        });
        match write_file(&path, &next) {
            Ok(()) => ActionOutcome::Applied { note: None },
            Err(e) => ActionOutcome::Failed(e.to_string()),
        }
    }

    /// Remove a managed JSON key or array element. Journals the whole file.
    fn remove_json_key(
        &mut self,
        path: &str,
        pointer: &str,
        removed: &mut Vec<String>,
    ) -> ActionOutcome {
        let key = format!("{path}:{pointer}");
        removed.push(key.clone());
        let full = self.root.join(path);
        let existing = match read_text(&full) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let Some(content) = existing else {
            return ActionOutcome::Skipped("already gone".into());
        };
        let value = match json_value_at(path, &content, pointer) {
            Ok(Some(value)) => value,
            Ok(None) => return ActionOutcome::Skipped("already gone".into()),
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        if self.prior_hashes.get(&key) != Some(&sha256_hex(value.as_bytes())) {
            return ActionOutcome::Skipped(
                "changed since superdev wrote it — left in place".into(),
            );
        }
        let (next, _) = match remove_json_pointer(path, &content, pointer) {
            Ok(edited) => edited.expect("the entry is present"),
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        self.journal.push(Undo::RestoreFile {
            path: path.to_string(),
            prior: Some(content),
        });
        match write_file(&full, &next) {
            Ok(()) => ActionOutcome::Applied { note: None },
            Err(e) => ActionOutcome::Failed(e.to_string()),
        }
    }
```

- [ ] **Step 4: Run the crate tests** — `cargo test -p superdev-core` — expected: PASS.

- [ ] **Step 5: Commit** — `git add crates/lib/superdev-core/src/engine.rs && git commit -S -m "feat(engine): execute removals with backup, journal and lock release"`

---

### Task 4: The orphan pass — `superdev_core::orphan`

Subtract live claims from the lock and classify what remains. No filesystem writes; this is planning.

**Files:**
- Create: `crates/lib/superdev-core/src/orphan.rs`
- Modify: `crates/lib/superdev-core/src/lib.rs` (add `pub mod orphan;` to the module list)
- Modify: `crates/lib/superdev-core/src/lock.rs` (back-compat test only)

**Interfaces:**
- Consumes: `Claim` (Task 1), removal `Action`s (Task 2), `engine::{read_text, json_value_at}` (`pub(crate)`), `mise::current_pin`, `lock::sha256_hex`.
- Produces:

```rust
pub struct OrphanPlan {
    pub actions: Vec<Action>,   // removals of superdev's own residue, in lock order
    pub released: Vec<String>,  // lock keys the user changed: left in place
    pub gone: Vec<String>,      // lock keys whose target is already gone
}
impl OrphanPlan {
    pub fn released_lines(&self) -> Vec<String>;  // one report line per released key
}
pub fn plan(root: &Path, lock: &Lock, claims: &[Claim]) -> Result<OrphanPlan>;
```

`plan` fails on an orphan it cannot read (IO error other than NotFound, malformed `.mise.toml` or JSON, a directory at a file path).

- [ ] **Step 1: Write the failing tests** (in `orphan.rs`'s own tests mod)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::component::Claim;
    use crate::components::mise;
    use crate::lock::{Lock, sha256_hex};

    fn lock_with(entries: &[(&str, &str)]) -> Lock {
        let mut lock = Lock::default();
        for (key, content) in entries {
            lock.files.insert((*key).into(), sha256_hex(content.as_bytes()));
        }
        lock
    }

    #[test]
    fn claimed_entries_are_never_orphans() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("kept.txt"), "content").unwrap();
        let lock = lock_with(&[("kept.txt", "content")]);
        let claims = vec![Claim::File("kept.txt".into())];
        let plan = plan(dir.path(), &lock, &claims).unwrap();
        assert!(plan.actions.is_empty());
        assert!(plan.released.is_empty());
        assert!(plan.gone.is_empty());
    }

    #[test]
    fn each_shape_classifies_by_disk_state() {
        let dir = tempfile::tempdir().unwrap();
        // Unmodified file → removal. Modified file → released. Missing → gone.
        std::fs::write(dir.path().join("stale.txt"), "superdev's").unwrap();
        std::fs::write(dir.path().join("theirs.txt"), "edited").unwrap();
        // Unmodified pin → removal.
        let mise_toml = mise::set_pin("", "http:codegraph", "\"1.5.0\"").unwrap();
        std::fs::write(dir.path().join(".mise.toml"), &mise_toml).unwrap();
        let pin_value = mise::current_pin(&mise_toml, "http:codegraph").unwrap().unwrap();
        // Unmodified JSON key → removal.
        std::fs::write(
            dir.path().join(".mcp.json"),
            r#"{"mcpServers":{"superdev-aokf":{"command":"superdev"}}}"#,
        )
        .unwrap();
        let mcp_value: serde_json::Value =
            serde_json::from_str(r#"{"command":"superdev"}"#).unwrap();

        let mut lock = lock_with(&[
            ("stale.txt", "superdev's"),
            ("theirs.txt", "superdev's"),
            ("vanished.txt", "whatever"),
        ]);
        lock.files.insert(
            mise::pin_lock_key("http:codegraph"),
            sha256_hex(pin_value.as_bytes()),
        );
        lock.files.insert(
            ".mcp.json:mcpServers.superdev-aokf".into(),
            sha256_hex(mcp_value.to_string().as_bytes()),
        );

        let plan = plan(dir.path(), &lock, &[]).unwrap();
        assert_eq!(plan.released, vec!["theirs.txt".to_string()]);
        assert_eq!(plan.gone, vec!["vanished.txt".to_string()]);
        let descs: Vec<String> = plan.actions.iter().map(Action::describe).collect();
        assert!(descs.contains(&"remove stale.txt (no longer in the blueprint)".into()), "{descs:?}");
        assert!(descs.contains(&"unpin http:codegraph in .mise.toml".into()), "{descs:?}");
        assert!(
            descs.contains(&"remove mcpServers.superdev-aokf from .mcp.json".into()),
            "{descs:?}"
        );
        assert_eq!(
            plan.released_lines(),
            vec![
                "orphan: theirs.txt changed since superdev wrote it — left in place, released from the lock"
                    .to_string()
            ]
        );
    }

    #[test]
    fn an_absent_pin_or_key_in_a_present_file_is_gone() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".mise.toml"), "[tools]\nnode = \"24\"\n").unwrap();
        std::fs::write(dir.path().join(".mcp.json"), r#"{"mcpServers":{}}"#).unwrap();
        let lock = lock_with(&[
            (".mise.toml:http:codegraph", "x"),
            (".mcp.json:mcpServers.superdev-aokf", "x"),
        ]);
        let plan = plan(dir.path(), &lock, &[]).unwrap();
        assert!(plan.actions.is_empty());
        assert_eq!(plan.gone.len(), 2);
    }

    #[test]
    fn an_unreadable_orphan_fails_loudly() {
        let dir = tempfile::tempdir().unwrap();
        // A directory where the locked file should be.
        std::fs::create_dir(dir.path().join("was-a-file.txt")).unwrap();
        let lock = lock_with(&[("was-a-file.txt", "content")]);
        assert!(plan(dir.path(), &lock, &[]).is_err());
        // Malformed shared files are errors too, never guesses.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".mise.toml"), "[tools\n").unwrap();
        let lock = lock_with(&[(".mise.toml:http:codegraph", "x")]);
        assert!(plan(dir.path(), &lock, &[]).is_err());
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".mcp.json"), "not json").unwrap();
        let lock = lock_with(&[(".mcp.json:mcpServers.superdev-aokf", "x")]);
        assert!(plan(dir.path(), &lock, &[]).is_err());
    }
}
```

And the back-compat test in `lock.rs` — a verbatim 0.1.0-shape lock with all three key shapes:

```rust
    #[test]
    fn a_0_1_0_lock_reads_unchanged() {
        let toml = r#"[components.skills]
provider = "superdev-skills"
version = "0.1.0"

[files]
".agents/aokf/SPEC.md" = "aaaa"
".mise.toml:http:superpowers" = "bbbb"
".claude/settings.json:hooks.PostToolUse[superdev aokf hook validate]" = "cccc"
"#;
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".superdev")).unwrap();
        std::fs::write(dir.path().join(LOCK_PATH), toml).unwrap();
        let lock = Lock::load(dir.path()).unwrap();
        assert_eq!(lock.components["skills"].provider, "superdev-skills");
        assert_eq!(lock.files[".mise.toml:http:superpowers"], "bbbb");
        assert_eq!(lock.files.len(), 3);
    }
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev-core orphan a_0_1_0` — expected: compile error, no `orphan` module.

- [ ] **Step 3: Implement `orphan.rs`**

```rust
//! orphan.rs — what the lock records that no live claim covers.
//!
//! There are no migration scripts: the lock is what superdev applied, the
//! components' claims are what the blueprint wants now, and the difference
//! is the migration.

use std::collections::BTreeSet;
use std::path::Path;

use crate::action::Action;
use crate::component::Claim;
use crate::components::mise;
use crate::engine::{json_value_at, read_text};
use crate::error::Result;
use crate::lock::{Lock, sha256_hex};

/// The orphan pass, computed against the lock. Planning only; the engine
/// runs the actions and the caller drops the released and gone keys.
#[derive(Debug, Default)]
pub struct OrphanPlan {
    /// Removals of superdev's own residue, in lock order.
    pub actions: Vec<Action>,
    /// Lock keys whose content the user changed: left in place, released.
    pub released: Vec<String>,
    /// Lock keys whose target is already gone: dropped silently.
    pub gone: Vec<String>,
}

impl OrphanPlan {
    /// One report line per released orphan.
    pub fn released_lines(&self) -> Vec<String> {
        self.released
            .iter()
            .map(|key| {
                format!(
                    "orphan: {key} changed since superdev wrote it — left in place, released from the lock"
                )
            })
            .collect()
    }
}

/// Every lock `files` entry no claim covers, classified by what is on disk.
/// Fails on an orphan it cannot read — the rule everywhere the engine
/// refuses to guess about content.
pub fn plan(root: &Path, lock: &Lock, claims: &[Claim]) -> Result<OrphanPlan> {
    let claimed: BTreeSet<String> = claims.iter().map(Claim::lock_key).collect();
    let mut plan = OrphanPlan::default();
    for (key, locked_hash) in &lock.files {
        if claimed.contains(key) {
            continue;
        }
        match current_value(root, key)? {
            None => plan.gone.push(key.clone()),
            Some(value) if sha256_hex(value.as_bytes()) == *locked_hash => {
                plan.actions.push(removal(key));
            }
            Some(_) => plan.released.push(key.clone()),
        }
    }
    Ok(plan)
}

/// A lock key, parsed back into the claim shape its format encodes.
/// superdev never writes a file path containing `:`, so a colon means a
/// managed entry in a shared file.
fn classify(key: &str) -> Claim {
    if let Some(tool) = key.strip_prefix(".mise.toml:") {
        return Claim::MisePin(tool.to_string());
    }
    match key.split_once(':') {
        Some((path, pointer)) => Claim::JsonKey {
            path: path.to_string(),
            pointer: pointer.to_string(),
        },
        None => Claim::File(key.to_string()),
    }
}

/// What the repo currently holds for a lock key — the text the key's hash
/// was taken over. `None` when the target is gone.
fn current_value(root: &Path, key: &str) -> Result<Option<String>> {
    match classify(key) {
        Claim::File(path) => read_text(&root.join(path)),
        Claim::MisePin(tool) => match read_text(&root.join(".mise.toml"))? {
            None => Ok(None),
            Some(content) => mise::current_pin(&content, &tool),
        },
        Claim::JsonKey { path, pointer } => match read_text(&root.join(&path))? {
            None => Ok(None),
            Some(content) => json_value_at(&path, &content, &pointer),
        },
    }
}

fn removal(key: &str) -> Action {
    match classify(key) {
        Claim::File(path) => Action::RemoveFile {
            path,
            reason: "no longer in the blueprint".into(),
        },
        Claim::MisePin(tool) => Action::RemoveMisePin { tool },
        Claim::JsonKey { path, pointer } => Action::RemoveJsonKey { path, pointer },
    }
}
```

Add `pub mod orphan;` to `lib.rs` (alphabetical: between `manifest` and `registry`).

- [ ] **Step 4: Run the crate tests** — `cargo test -p superdev-core` — expected: PASS.

- [ ] **Step 5: Commit** — `git add crates/lib/superdev-core/src/orphan.rs crates/lib/superdev-core/src/lib.rs crates/lib/superdev-core/src/lock.rs && git commit -S -m "feat(core): plan the orphan pass from the lock and the live claims"`

---

### Task 5: The entry point — `CLAUDE.md` imports `AGENTS.md`

Claude Code reads `CLAUDE.md`, not `AGENTS.md`. The knowledge component ensures the import line, behaving exactly like the `.gitignore` lines: planned only when missing, never rewritten, never hashed into the lock, therefore never an orphan.

**Files:**
- Modify: `crates/lib/superdev-core/src/components/aokf.rs`

**Interfaces:**
- Consumes: `Action::EnsureLine` (exact whole-line match — the plan-time check must mirror the engine's `lines().any(|l| l == line)` rule, or a converged repo would plan work and `status` would exit 1 forever).
- Produces: constants `CLAUDE_ENTRY_PATH: &str = "CLAUDE.md"` and `CLAUDE_ENTRY_LINE: &str = "@AGENTS.md"` (private), and the `EnsureLine` in `Aokf::plan`. `owned()` is untouched — an `EnsureLine` is never locked.

- [ ] **Step 1: Write the failing tests** (in `aokf.rs` tests, using the existing `plan_in` helper)

```rust
    #[test]
    fn claude_md_gets_the_agents_import() {
        // No CLAUDE.md at all: plan the line (the engine creates the file).
        let dir = tempfile::tempdir().unwrap();
        let ensure = plan_in(dir.path()).into_iter().find_map(|a| match a {
            Action::EnsureLine { path, line, .. } => Some((path, line)),
            _ => None,
        });
        assert_eq!(ensure, Some(("CLAUDE.md".to_string(), "@AGENTS.md".to_string())));

        // A CLAUDE.md of the user's own: plan the append, touch nothing else.
        std::fs::write(dir.path().join("CLAUDE.md"), "# My rules\n").unwrap();
        assert!(plan_in(dir.path()).iter().any(|a| matches!(a, Action::EnsureLine { .. })));

        // The line present (anywhere, exact whole-line): nothing to plan.
        std::fs::write(dir.path().join("CLAUDE.md"), "# My rules\n@AGENTS.md\n").unwrap();
        assert!(!plan_in(dir.path()).iter().any(|a| matches!(a, Action::EnsureLine { .. })));

        // A substring is not the line: `see @AGENTS.md` does not satisfy it.
        std::fs::write(dir.path().join("CLAUDE.md"), "see @AGENTS.md inline\n").unwrap();
        assert!(plan_in(dir.path()).iter().any(|a| matches!(a, Action::EnsureLine { .. })));
    }
```

Also update the existing `scaffolds_are_not_replanned_but_owned_drift_is` test: its converge loop replays planned actions and will now meet an `EnsureLine` — add a match arm that appends the line:

```rust
                Action::EnsureLine { path, line, .. } => {
                    let p = dir.path().join(path);
                    let mut content = std::fs::read_to_string(&p).unwrap_or_default();
                    content.push_str(&line);
                    content.push('\n');
                    std::fs::write(p, content).unwrap();
                }
```

(The same replay loop exists in `fresh_repo_plans_every_file_with_name_substituted` — its `filter_map` must tolerate the new variant: map `EnsureLine` to `None` beside `SetJsonKey`.)

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev-core claude_md_gets` — expected: FAIL, no `EnsureLine` planned.

- [ ] **Step 3: Implement**

Constants beside `MCP_PATH`:

```rust
/// Claude Code reads CLAUDE.md, not AGENTS.md: without this import, every
/// rule superdev writes into AGENTS.md is invisible to it. Behaves like the
/// .gitignore lines — added when missing, never rewritten, never locked.
const CLAUDE_ENTRY_PATH: &str = "CLAUDE.md";
const CLAUDE_ENTRY_LINE: &str = "@AGENTS.md";
```

In `Aokf::plan`, after the `FILES` loop and before the MCP check:

```rust
        let claude = std::fs::read_to_string(ctx.root.join(CLAUDE_ENTRY_PATH)).unwrap_or_default();
        // Exact whole-line match, the same rule the engine applies.
        if !claude.lines().any(|l| l == CLAUDE_ENTRY_LINE) {
            actions.push(Action::EnsureLine {
                path: CLAUDE_ENTRY_PATH.into(),
                line: CLAUDE_ENTRY_LINE.into(),
                reason: "make Claude Code read AGENTS.md".into(),
            });
        }
```

- [ ] **Step 4: Run the crate tests** — `cargo test -p superdev-core` — expected: PASS.

- [ ] **Step 5: Commit** — `git add crates/lib/superdev-core/src/components/aokf.rs && git commit -S -m "feat(knowledge): ensure CLAUDE.md imports AGENTS.md"`

---

### Task 6: Wire the orphan pass into `manage.rs`

Plan the orphan entry last, print released reports, drop released/gone keys (and disabled capabilities' applied records) from the lock on sync.

**Files:**
- Modify: `crates/app/superdev/src/manage.rs`

**Interfaces:**
- Consumes: `superdev_core::orphan::{self, OrphanPlan}`, `superdev_core::component::Claim`, `Component::owned`.
- Produces: `fn plan_all(...) -> Result<(Vec<Planned>, orphan::OrphanPlan)>` — unchanged inputs, now also returns the orphan plan; the orphan entry (`capability: None, provider: REPO_PROVIDER`) is pushed **last**, so removals run after every component write and a failed write rolls back before anything is deleted.
- Ordering invariant (state it in a comment): `prune_custom_skills` runs **before** `plan_all` in both `status` and `sync`. A skill just marked custom still has its lock entry; unpruned, an unmodified one would read as an orphan and be deleted — the opposite of what marking it custom asked for.
- Exit codes: removal actions are planned actions (`status` exits 1); released orphans are reports and never touch the exit code.

- [ ] **Step 1: Write the failing unit test** (in `manage.rs` tests — the full e2e comes in Task 8)

```rust
    #[test]
    fn plan_all_puts_the_orphan_entry_last_and_reports_released() {
        use superdev_core::runner::FakeRunner;
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let manifest = Manifest::default_for(superdev_core::version(), &[]);
        let mut lock = Lock::default();
        // An unmodified leftover and a user-edited one, under no live claim.
        std::fs::write(dir.path().join("stale.txt"), "superdev's").unwrap();
        lock.files.insert(
            "stale.txt".into(),
            superdev_core::lock::sha256_hex(b"superdev's"),
        );
        std::fs::write(dir.path().join("theirs.txt"), "edited").unwrap();
        lock.files.insert(
            "theirs.txt".into(),
            superdev_core::lock::sha256_hex(b"superdev's"),
        );
        let fake = FakeRunner::new();
        let (planned, orphans) = plan_all(dir.path(), &fake, &manifest, &lock).unwrap();
        let last = planned.last().unwrap();
        assert!(last.capability.is_none());
        assert!(
            last.actions
                .iter()
                .any(|a| a.describe().contains("remove stale.txt")),
            "{:?}",
            last.actions
        );
        assert_eq!(orphans.released, vec!["theirs.txt".to_string()]);
    }
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev plan_all_puts` — expected: compile error (`plan_all` returns a single `Vec`).

- [ ] **Step 3: Implement**

`plan_all` (add `use superdev_core::component::Claim;` and `use superdev_core::orphan;`):

```rust
fn plan_all(
    root: &Path,
    runner: &dyn CommandRunner,
    manifest: &Manifest,
    lock: &Lock,
) -> Result<(Vec<Planned>, orphan::OrphanPlan)> {
    let components = components::enabled(manifest);
    let ctx = Ctx {
        root,
        runner,
        manifest,
        lock,
    };
    let mut planned = Vec::new();
    planned.extend(repo_entry(root, manifest)?);
    planned.extend(engine::plan(&components, &ctx)?);
    let claims: Vec<Claim> = components.iter().flat_map(|c| c.owned(&ctx)).collect();
    let orphans = orphan::plan(root, lock, &claims)?;
    // Last, so removals run after every component write: a rename whose
    // write fails rolls back before anything is deleted.
    if !orphans.actions.is_empty() {
        planned.push(Planned {
            capability: None,
            provider: REPO_PROVIDER.into(),
            actions: orphans.actions.clone(),
        });
    }
    Ok((planned, orphans))
}
```

`init`: destructure `let (planned, _) = plan_all(...)?;` — the lock is empty on init, so the orphan plan is too.

`status`:

```rust
pub fn status(root: &Path) -> Result<u8> {
    let manifest = load_manifest(root)?;
    let behind = behind_pins(&manifest);
    let mut lock = Lock::load(root)?;
    // In memory only — status never writes. Unpruned, a skill just marked
    // custom would read as an orphan and plan its own deletion.
    prune_custom_skills(&manifest, &mut lock);
    let runner = SystemRunner;
    let (planned, orphans) = plan_all(root, &runner, &plannable(&manifest), &lock)?;
    print_plan(&planned)?;
    for line in &behind {
        out(line)?;
    }
    for line in &custom_lines(&manifest) {
        out(line)?;
    }
    for line in &orphans.released_lines() {
        out(line)?;
    }
    Ok(u8::from(has_actions(&planned) || !behind.is_empty()))
}
```

`sync` — released and gone orphans leave the lock without an action, and a disabled capability's applied record goes with its files; both save even on an otherwise idle run:

```rust
pub fn sync(root: &Path, dry_run: bool) -> Result<u8> {
    let manifest = load_manifest(root)?;
    // Unlike `status`, sync would have to act on the pin. Substituting the
    // default silently is worse than stopping.
    if let Some((capability, pinned, default)) = checksum_pin_mismatch(&manifest) {
        return Err(Error::Manifest {
            message: format!(
                "{} is pinned {pinned} but this superdev only supports {default} — run `superdev update`",
                capability.as_str()
            ),
        });
    }
    let mut lock = Lock::load(root)?;
    let mut lock_changed = prune_custom_skills(&manifest, &mut lock);
    let runner = SystemRunner;
    let (planned, orphans) = plan_all(root, &runner, &manifest, &lock)?;
    print_plan(&planned)?;
    for line in &behind_pins(&manifest) {
        out(line)?;
    }
    for line in &orphans.released_lines() {
        out(line)?;
    }
    if dry_run {
        return Ok(0);
    }
    for key in orphans.released.iter().chain(orphans.gone.iter()) {
        lock_changed |= lock.files.remove(key).is_some();
    }
    let disabled: Vec<String> = lock
        .components
        .keys()
        .filter(|name| !manifest.capabilities.contains_key(*name))
        .cloned()
        .collect();
    for name in disabled {
        lock.components.remove(&name);
        lock_changed = true;
    }
    if !has_actions(&planned) {
        if lock_changed {
            lock.save(root)?;
        }
        return Ok(0);
    }
    apply_and_report(root, &runner, &manifest, &planned, &mut lock)
}
```

- [ ] **Step 4: Run the workspace tests** — `cargo nextest run --workspace` — expected: PASS (existing e2e tests still converge: their repos have no unclaimed lock entries).

- [ ] **Step 5: Commit** — `git add crates/app/superdev/src/manage.rs && git commit -S -m "feat(manage): plan the orphan pass last and release what the user changed"`

---

### Task 7: `blueprint` becomes the version last applied

**Files:**
- Modify: `crates/app/superdev/src/manage.rs`

**Interfaces:**
- Produces:
  - `fn blueprint_line(manifest: &Manifest) -> Option<String>` — `Some("blueprint <a>, binary <b> — sync will update it")` when they differ, printed by `status` after the custom/orphan lines; never affects the exit code.
  - `fn stamp_blueprint(root: &Path, manifest: &Manifest) -> Result<u8>` — rewrites `config.toml` only when the value changes (the same whole-file `Manifest::save` that `update` already uses); returns `Ok(0)`. Called at the end of `sync` on **both** success paths (idle and applied), never on `--dry-run`, and not by `status`.
- Note: `update` saves the manifest with its loaded `blueprint` untouched, then funnels into `sync`, which stamps — no change needed there. `init` writes the current version at creation and needs no stamp. The version scripts (`set-version.mjs`/`verify-version.mjs`) deliberately do not track `blueprint`: after a release bump it is one `sync` behind, which is an informational line, not drift, so the CI `check:blueprint` gate stays green.

- [ ] **Step 1: Write the failing unit tests**

```rust
    #[test]
    fn the_blueprint_line_reports_only_a_difference() {
        let mut manifest = Manifest::default_for(superdev_core::version(), &[]);
        assert_eq!(blueprint_line(&manifest), None);
        manifest.blueprint = "0.0.1".into();
        assert_eq!(
            blueprint_line(&manifest),
            Some(format!(
                "blueprint 0.0.1, binary {} — sync will update it",
                superdev_core::version()
            ))
        );
    }

    #[test]
    fn stamping_rewrites_only_a_stale_blueprint() {
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.0.1", &[]);
        manifest.save(dir.path()).unwrap();
        assert_eq!(stamp_blueprint(dir.path(), &manifest).unwrap(), 0);
        let stamped = Manifest::load(dir.path()).unwrap();
        assert_eq!(stamped.blueprint, superdev_core::version());
        // Already current: the file is left untouched.
        let before = std::fs::metadata(dir.path().join(CONFIG_PATH)).unwrap().modified().unwrap();
        assert_eq!(stamp_blueprint(dir.path(), &stamped).unwrap(), 0);
        let after = std::fs::metadata(dir.path().join(CONFIG_PATH)).unwrap().modified().unwrap();
        assert_eq!(before, after, "no rewrite when the value is current");
    }
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev blueprint_line stamping_rewrites` — expected: compile error.

- [ ] **Step 3: Implement**

```rust
/// The blueprint-version report: informational, never the exit code. A
/// settled repo under a newer binary is not drift.
fn blueprint_line(manifest: &Manifest) -> Option<String> {
    (manifest.blueprint != superdev_core::version()).then(|| {
        format!(
            "blueprint {}, binary {} — sync will update it",
            manifest.blueprint,
            superdev_core::version()
        )
    })
}

/// Record this binary's version as the blueprint last applied. Rewrites
/// config.toml only when the value changes.
fn stamp_blueprint(root: &Path, manifest: &Manifest) -> Result<u8> {
    if manifest.blueprint != superdev_core::version() {
        let mut manifest = manifest.clone();
        manifest.blueprint = superdev_core::version().to_string();
        manifest.save(root)?;
    }
    Ok(0)
}
```

In `status`, after the released-orphan lines:

```rust
    if let Some(line) = blueprint_line(&manifest) {
        out(&line)?;
    }
```

In `sync`, replace the two success returns:

```rust
    if !has_actions(&planned) {
        if lock_changed {
            lock.save(root)?;
        }
        return stamp_blueprint(root, &manifest);
    }
    apply_and_report(root, &runner, &manifest, &planned, &mut lock)?;
    stamp_blueprint(root, &manifest)
```

- [ ] **Step 4: Run the workspace tests** — `cargo nextest run --workspace` — expected: PASS.

- [ ] **Step 5: Commit** — `git add crates/app/superdev/src/manage.rs && git commit -S -m "feat(manage): stamp the blueprint version on a successful sync"`

---

### Task 8: End-to-end tests

The spec's e2e list, through the real binary. The skills/knowledge scenarios run in `tests/cli.rs` (no external tools: init with `--no-workflows --no-code-index --no-frontend` plans no `Run` actions). The mise-pin sweep runs in `tests/manage.rs` against the fake `mise`. Follow each file's existing helpers (`cli.rs` has its own repo/init conventions — read them first; `manage.rs` has `Sandbox`).

**Files:**
- Test: `crates/app/superdev/tests/cli.rs`
- Test: `crates/app/superdev/tests/manage.rs`

**Interfaces:**
- Consumes: the shipped behaviour of Tasks 1–7; `skillpack::SKILLS` content for byte-identical fixtures.

- [ ] **Step 1: Write the disabled-capability sweep test** (`cli.rs`)

Scenario, asserting the exact strings from Global Constraints:

```rust
#[test]
fn disabling_skills_sweeps_them_and_releases_the_users_edit() {
    // 1. git repo + `init --no-workflows --no-code-index --no-frontend` → exit 0.
    // 2. Overwrite .claude/skills/humanise/SKILL.md with "mine now\n".
    // 3. Rewrite .superdev/config.toml without the [skills] table (keep the
    //    other tables and the blueprint line verbatim).
    // 4. `status` → exit 1; stdout contains
    //    "remove .claude/skills/grill-me/SKILL.md (no longer in the blueprint)"
    //    and "orphan: .claude/skills/humanise/SKILL.md changed since superdev
    //    wrote it — left in place, released from the lock".
    // 5. `sync` → exit 0. Then assert:
    //    - grill-me's SKILL.md is gone; a backup copy with the shipped bytes
    //      exists under .superdev/cache/backup/<stamp>/.claude/skills/grill-me/;
    //    - humanise's SKILL.md still reads "mine now\n";
    //    - .claude/settings.json parses, and hooks.PostToolUse no longer has
    //      an element containing "superdev aokf hook validate";
    //    - .superdev/lock.toml contains no ".claude/skills/" key, no
    //      "hooks.PostToolUse" key, and no [components.skills] table.
    // 6. `status` → exit 0: a settled state is not drift.
}
```

- [ ] **Step 2: Write the CLAUDE.md tests** (`cli.rs`)

```rust
#[test]
fn claude_md_import_is_created_appended_and_restored() {
    // A: repo with no CLAUDE.md → init → CLAUDE.md exists, content "@AGENTS.md\n".
    // B: repo with CLAUDE.md "# House rules\n" → init → content
    //    "# House rules\n@AGENTS.md\n" (the user's text intact, line appended).
    // C: delete the line (restore "# House rules\n") → `status` exits 1,
    //    `sync` exits 0 and the line is back — the .gitignore bargain.
    // D: `status` after C exits 0 and CLAUDE.md was not rewritten further.
}
```

- [ ] **Step 3: Write the stale-blueprint test** (`cli.rs`)

```rust
#[test]
fn a_stale_blueprint_reports_on_status_and_sync_stamps_it() {
    // init (skills+knowledge only), then edit .superdev/config.toml:
    // blueprint = "0.0.1". `status` → exit 0 (converged!), stdout contains
    // "blueprint 0.0.1, binary <CARGO_PKG_VERSION> — sync will update it".
    // `sync` → exit 0; config.toml's blueprint equals CARGO_PKG_VERSION;
    // `status` no longer prints the line.
}
```

Use `env!("CARGO_PKG_VERSION")` for the binary version in assertions.

- [ ] **Step 4: Write the pin sweep test** (`manage.rs`, unix-only like the rest of the file)

```rust
#[test]
fn disabling_code_index_unpins_codegraph_and_keeps_user_pins() {
    // Full-blueprint init in the Sandbox (fakes provide mise/claude/codegraph).
    // Append a user pin: `node = "24"` under [tools] via fs edit of .mise.toml.
    // Remove the [code-index] table from .superdev/config.toml.
    // `status` → exit 1, stdout contains "unpin http:codegraph in .mise.toml".
    // `sync` → exit 0; .mise.toml has no http:codegraph key, still has the
    // node pin and the http:superpowers pin; lock has no
    // ".mise.toml:http:codegraph" key and no [components.code-index] table.
}
```

- [ ] **Step 5: Run each new test, then the whole suite** — `cargo nextest run --workspace` — expected: PASS. If a scenario surfaces a real defect in Tasks 1–7, fix the defect (not the test) and note it in the commit body.

- [ ] **Step 6: Commit** — `git add crates/app/superdev/tests/ && git commit -S -m "test: end-to-end orphan sweep, CLAUDE.md import and blueprint stamping"`

---

### Task 9: Knowledge bundle and changelog

Code is canonical; the bundle must describe the new behaviour. Every edit under `knowledge/` triggers the validation hook; finish with a clean `npm run check:aokf` (PASS at level 2) and `npm run check:blueprint` (exit 0 — this repo must stay converged).

**Files:**
- Modify: `knowledge/configuration.md`
- Modify: `knowledge/api-contracts.md`
- Modify: `knowledge/glossary.md`
- Modify: `knowledge/architecture.md`
- Modify: `knowledge/specs/2026-08-12-blueprint-migrations-design.md` (frontmatter only: `status: draft` → `status: stable`)
- Modify: `CHANGELOG.md`

**Interfaces:**
- Consumes: the shipped behaviour of Tasks 1–8. Read each file before editing; write in its existing register (PROSE.md).

- [ ] **Step 1: `configuration.md`** — two edits. In the manifest section, replace the `blueprint` comment/description: it is now *the superdev version last applied* — `sync` stamps it on success and rewrites the file only when the value changes; `status` reports a difference without failing on it. In the `lock.toml` section, add a short paragraph: a lock entry no component claims any more is an orphan — `sync` removes it when its content still hashes to the locked value (backed up like any overwrite) and otherwise leaves the file and drops the entry, reporting it once; entries whose target is already gone leave the lock silently.

- [ ] **Step 2: `api-contracts.md`** — update the verb bullets: `init` also ensures `CLAUDE.md` contains `@AGENTS.md` (Claude Code reads only `CLAUDE.md`; the line makes it load the canonical entry point — appended to an existing file, or created as a one-line file); `status` exits 1 on planned removals, and prints released orphans and the blueprint-version line as reports that never affect the exit code; `sync` removes orphans after all writes, releases user-modified ones, and stamps the blueprint version on success.

- [ ] **Step 3: `glossary.md`** — add two terms in the file's style: **claim** (a typed lock entry a component declares it owns — file, mise pin, or JSON key; the orphan pass subtracts claims from the lock) and **orphan** (a lock entry no live claim covers: removed when unmodified, released to the user when edited).

- [ ] **Step 4: `architecture.md`** — in the managed-files description, add the `CLAUDE.md` import line beside the `.gitignore` lines (same never-locked `EnsureLine` mechanics), and one sentence that migrations are derived from lock-minus-claims, not scripted.

- [ ] **Step 5: `CHANGELOG.md`** — under `## [Unreleased]` → `### Added`:

```markdown
- Blueprint migrations: `sync` now removes what the blueprint no longer
  ships — dropped files, renamed paths' old copies, a disabled capability's
  pins and registrations. Unmodified leftovers are removed with a backup;
  user-edited ones are left in place, released from the lock, and reported
- `sync` ensures `CLAUDE.md` imports `AGENTS.md` (`@AGENTS.md`), so Claude
  Code actually loads the managed entry point
- `blueprint` in `.superdev/config.toml` now records the version last
  applied: `sync` stamps it, `status` reports a difference without failing
```

- [ ] **Step 6: Flip the spec** — in `knowledge/specs/2026-08-12-blueprint-migrations-design.md`, change `status: draft` to `status: stable`. Nothing else in the file changes.

- [ ] **Step 7: Validate and verify** — `npm run check:aokf` (PASS at level 2; the `implements` warning on this plan file is expected), `npm run check:blueprint` (exit 0), `cargo nextest run --workspace` (PASS).

- [ ] **Step 8: Commit** — `git add knowledge/ CHANGELOG.md && git commit -S -m "docs: record the migration, entry-point and blueprint-version behaviour"`

---

## Self-Review

- **Spec coverage:** derived migration model + `owned()` → Tasks 1, 4; removal actions with backup/journal/unwind and the unmodified/modified/gone/unreadable rules → Tasks 2–4; orphan pass planned last → Task 6; entry point → Task 5; blueprint version → Task 7; exit codes and reports → Tasks 6–7; every test named in the spec's Testing section → Tasks 1, 3, 4, 8 (lock back-compat in Task 4); docs → Task 9. The spec's "fresh goodbye-tinnitus adoption" proof is a post-merge reality trial, not a plan task.
- **Placeholder scan:** Task 8's test bodies are commented scenarios with exact commands, paths, and expected strings rather than compiling code — deliberate, because `cli.rs` helpers must be read first; every assertion value is stated. No TBDs remain.
- **Type consistency:** `Claim::lock_key`, `owned(&self, ctx: &Ctx<'_>) -> Vec<Claim>`, `plan_all -> Result<(Vec<Planned>, orphan::OrphanPlan)>`, `orphan::plan(root, lock, claims)`, `released_lines`, `stamp_blueprint(root, &manifest)`, and the three action variants are named identically in every task that uses them.

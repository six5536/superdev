---
type: Plan
id: plan-workflows-provider-default
title: Workflows Provider Default Implementation Plan
description: Task-by-task plan for registry-backed provider selection, the materialised mattpocock-skills provider, the default flip, and the dogfood switch.
status: draft
links:
  - rel: implements
    to: spec-workflows-provider-default
    note: Edges declared plan-side only, so deleting this plan leaves no dangling references.
---

# Workflows Provider Default Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the [workflows provider default spec](/knowledge/specs/2026-08-17-workflows-provider-default-design.md): real provider selection in the registry, mattpocock-skills delivered as materialised repo files, superpowers kept as the plugin-based secondary, and the default flipped — including switching this repo.

**Architecture:** The registry becomes per-(capability, provider) with one default flag each; `components::enabled` resolves the manifest's provider string and errors on unknown ids. A new `MaterialiseSkills` engine action copies the mise-pinned checkout into `.claude/skills/` with per-file backup/journal/lock plus a new lock `owners` attribution table, so `owned()` and the orphan pass stay correct without hardcoding the upstream skill list. The default flips in a late task to localise test churn.

**Tech Stack:** Rust (existing workspace only). No new dependencies.

## Global Constraints

- No new dependencies. If a task seems to need one, stop and ask.
- Pinned provider data (exact values, verified against the real v1.2.3 tarball):
  - mise tool: `http:mattpocock-skills`
  - URL: `https://github.com/mattpocock/skills/archive/refs/tags/v1.2.3.tar.gz`
  - checksum: `sha256:238fac54d0f53d3e2d0501c1b38c9c0e4e9bc26f6b057b53a7328ea15d43b66f`
  - version: `1.2.3`, `strip_components = 1`
  - source dirs inside the checkout: `skills/engineering`, `skills/productivity`
  - the 25 skill names (engineering then productivity, each set sorted): `ask-matt`, `code-review`, `codebase-design`, `diagnosing-bugs`, `domain-modeling`, `grill-with-docs`, `implement`, `improve-codebase-architecture`, `prototype`, `research`, `resolving-merge-conflicts`, `setup-matt-pocock-skills`, `tdd`, `to-spec`, `to-tickets`, `triage`, `wayfinder`, `wizard`, `grill-me`, `grilling`, `handoff`, `teach`, `to-questionnaire`, `wait-what`, `writing-for-agents`
- Exact user-facing strings:
  - unknown provider: `workflows provider must be one of: mattpocock-skills, superpowers` (list from the registry, registry order)
  - materialise action description: `materialise http:mattpocock-skills skills into .claude/skills/`
  - unknown custom name: `skills: custom names unknown skill '<name>' — no effect` and `workflows: custom names unknown skill '<name>' — no effect`
  - setup hint: `workflows: run /setup-matt-pocock-skills in Claude Code to finish configuring`
  - plugin leftover: `workflows: superpowers plugin left installed — \`claude plugin uninstall superpowers\` removes it`
  - import hint: `workflows: update the .agents import in AGENTS.md for the new provider`
  - adoption: `workflows: kept your <name> — marked custom in .superdev/config.toml`
  - custom report: `workflows: <name> custom, unmanaged`
- Reports never move the exit code; planned actions (pin, materialise, sweeps) do.
- Prose rules in `.agents/PROSE.md` bind every comment, doc line, and commit message.
- Verify with `cargo nextest run --workspace`; before each commit `npm run fmt` and `cargo clippy --workspace --all-targets` must be clean (do NOT use `npm run lint` — its wrapper is broken in this environment). From Task 5 on, also `npm run check:blueprint` (exit 0): this repo is superdev-managed and must stay converged on every commit.
- Never stage or revert `.claude/settings.json`, `.devcontainer/devcontainer.json`, `.mise.toml`, `.entire/`, `.DS_Store` — the user's uncommitted local state. Exception: Task 12 needs a `.mise.toml` change and explicitly stops for the user first.
- Commits: conventional prefix, no Claude signature, sign with `-S` (fall back to `--no-gpg-sign` if the SSH agent fails).
- After any change under `knowledge/`: `npm run check:aokf` must PASS at level 2 (a PostToolUse hook enforces this on Edit/Write).

---

### Task 1: Per-provider registry and fallible `enabled()`

The registry gains one entry per (capability, provider) with a `default` flag; `components::enabled` resolves the manifest's provider string instead of ignoring it. The mattpocock entry is NOT added here (Task 4); this task builds the machinery with the existing five providers, so the unknown-provider error already works.

**Files:**
- Modify: `crates/lib/superdev-core/src/registry.rs`
- Modify: `crates/lib/superdev-core/src/components/mod.rs`
- Modify: `crates/lib/superdev-core/src/manifest.rs` (default_for filters on `default`)
- Modify: `crates/app/superdev/src/manage.rs` (provider-aware registry lookups; `enabled(...)?`)
- Modify: `crates/lib/superdev-core/src/engine.rs` (test `plan_runs_every_component` uses `enabled(...)?`)

**Interfaces:**
- Produces, in `registry`:
  - `RegistryEntry` gains `pub default: bool` (exactly one `default: true` per capability; the flag names what `init` picks when the user names nothing).
  - `pub fn entries() -> &'static [RegistryEntry]` (length no longer part of the signature).
  - `pub fn default_entry(capability: Capability) -> &'static RegistryEntry`
  - `pub fn entry_for(capability: Capability, provider: &str) -> Option<&'static RegistryEntry>`
  - `pub fn providers_for(capability: Capability) -> Vec<&'static str>` (available entries, registry order)
- Produces, in `components`: `pub fn enabled(manifest: &Manifest) -> Result<Vec<Box<dyn Component>>>` — resolves each enabled capability's `(capability, provider)` pair; an unknown pair errors with `<capability> provider must be one of: <providers_for list, comma-separated>`.
- Produces, in `manage` (private): `fn registry_version(manifest: &Manifest, capability: Capability) -> Option<String>` — the version of the manifest's chosen provider's entry (`entry_for`), used by `behind_pins`, `pin_mismatch`, `checksum_pin_mismatch`, `plannable`, and `update`'s no-target arm. `default_version(capability)` remains only where the default is genuinely meant: `parse_target`'s error text and `update <capability>` with no version for a capability whose provider the manifest already names — replace its body with `default_entry(capability).version.map(str::to_string)`.

- [ ] **Step 1: Write the failing tests**

`registry.rs` — replace `covers_every_capability_once` and add lookups:

```rust
    #[test]
    fn one_default_per_capability_and_lookups_resolve() {
        for c in Capability::ALL {
            assert_eq!(
                entries().iter().filter(|e| e.capability == c && e.default).count(),
                1,
                "{c:?}"
            );
        }
        assert_eq!(default_entry(Capability::Workflows).provider, "superpowers");
        assert_eq!(
            entry_for(Capability::Workflows, "superpowers").unwrap().version,
            Some("6.2.0")
        );
        assert!(entry_for(Capability::Workflows, "flying").is_none());
        assert_eq!(providers_for(Capability::Knowledge), vec!["aokf"]);
    }
```

`components/mod.rs`:

```rust
    #[test]
    fn enabled_rejects_an_unknown_provider() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("workflows").unwrap().provider = "flying".into();
        let err = enabled(&manifest).unwrap_err().to_string();
        assert!(
            err.contains("workflows provider must be one of: superpowers"),
            "{err}"
        );
        assert!(enabled(&Manifest::default_for("0.1.0", &[])).is_ok());
    }
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev-core -- registry enabled_rejects` — expected: compile errors (`default` field, helpers, `Result` return).

- [ ] **Step 3: Implement `registry.rs`**

Add `default: bool` to the struct (doc comment: `/// The provider init picks when the user names none. Exactly one per capability.`), set `default: true` on all five existing entries, change `ENTRIES` to `const ENTRIES: [RegistryEntry; 5]` unchanged in content, and relax `entries()` to `-> &'static [RegistryEntry]`. Helpers:

```rust
/// The entry `init` uses for `capability` when no provider is named.
pub fn default_entry(capability: Capability) -> &'static RegistryEntry {
    ENTRIES
        .iter()
        .find(|e| e.capability == capability && e.default)
        .expect("every capability has a default entry")
}

/// The entry for a (capability, provider) pair, when the registry has one.
pub fn entry_for(capability: Capability, provider: &str) -> Option<&'static RegistryEntry> {
    ENTRIES
        .iter()
        .find(|e| e.capability == capability && e.provider == provider)
}

/// Valid provider ids for `capability`, in registry order.
pub fn providers_for(capability: Capability) -> Vec<&'static str> {
    ENTRIES
        .iter()
        .filter(|e| e.capability == capability && e.available)
        .map(|e| e.provider)
        .collect()
}
```

- [ ] **Step 4: Implement `enabled()` resolution**

Replace `components::enabled`:

```rust
/// Every enabled component, in canonical apply order, resolved from the
/// manifest's provider choices. Until this resolution existed, the
/// manifest's `provider` field was recorded but never read.
pub fn enabled(manifest: &Manifest) -> Result<Vec<Box<dyn Component>>> {
    let mut components: Vec<Box<dyn Component>> = Vec::new();
    for entry in registry::entries() {
        let Some(config) = manifest.capabilities.get(entry.capability.as_str()) else {
            continue;
        };
        if config.provider != entry.provider {
            continue;
        }
        components.push(component_for(entry.capability, entry.provider));
    }
    // Anything enabled but unresolved names a provider the registry lacks.
    for (name, config) in &manifest.capabilities {
        let capability = Capability::parse(name).expect("manifest rejects unknown capabilities");
        if registry::entry_for(capability, &config.provider).is_none() {
            return Err(Error::Manifest {
                message: format!(
                    "{name} provider must be one of: {}",
                    registry::providers_for(capability).join(", ")
                ),
            });
        }
    }
    Ok(components)
}

/// The component implementing a known (capability, provider) pair.
fn component_for(capability: Capability, provider: &str) -> Box<dyn Component> {
    match (capability, provider) {
        (Capability::Workflows, "superpowers") => Box::new(plugin::superpowers()),
        (Capability::Frontend, _) => Box::new(plugin::frontend_design()),
        (Capability::Skills, _) => Box::new(skillpack::SkillPack),
        (Capability::CodeIndex, _) => Box::new(codegraph::Codegraph),
        (Capability::Knowledge, _) => Box::new(aokf::Aokf),
        _ => unreachable!("resolved from the registry"),
    }
}
```

Add `use crate::capability::Capability; use crate::error::{Error, Result}; use crate::registry;` as needed. Wait — the `unreachable!` IS reachable for `(Workflows, "x")` before the unknown check runs. Restructure so validation comes first: run the unknown-provider loop before building, then `component_for` is genuinely total. Order in the final code: validate every enabled capability's provider via `entry_for`, then build the component list. Both loops stay as written, validation first.

- [ ] **Step 5: Ripple through callers**

- `manifest.rs` `default_for`: filter `e.available && e.default && !disabled.contains(&e.capability)`.
- `manage.rs`: `let components = components::enabled(manifest)?;` in `plan_all`; add `registry_version` and route `behind_pins`, `pin_mismatch`, `checksum_pin_mismatch`, `plannable`, and `update`'s per-capability arms through it:

```rust
/// The registry version for the provider the manifest names, when both exist.
fn registry_version(manifest: &Manifest, capability: Capability) -> Option<String> {
    let config = manifest.capabilities.get(capability.as_str())?;
    registry::entry_for(capability, &config.provider)?
        .version
        .map(str::to_string)
}
```

In `plannable`, reset each `BINARY_PINNED` capability's version to `registry_version(&manifest, capability)` (leave it untouched when `None` — an unknown provider fails later with the good message). In `behind_pins`/`pin_mismatch`, the `default` compared against is `registry_version(manifest, capability)`. `default_version(capability)` shrinks to `registry::default_entry(capability).version.map(str::to_string)` and stays used by `parse_target`'s error text and `update`'s explicit-target fallback.
- `engine.rs` test `plan_runs_every_component`: `components::enabled(&manifest).unwrap()`, and the broken-manifest assertion becomes: a bad version still fails `plan`; additionally assert `components::enabled` errors for an unknown provider? No — that's covered in components tests; just make it compile with `.unwrap()`.
- `components/mod.rs` tests `enabled_skips_disabled_capabilities` and `owned_matches_what_apply_locks`: add `.unwrap()`.

- [ ] **Step 6: Run the workspace tests** — `cargo nextest run --workspace` — expected: PASS (behaviour identical: every manifest names default providers today).

- [ ] **Step 7: Commit** — `git add crates/lib/superdev-core/src/registry.rs crates/lib/superdev-core/src/components/mod.rs crates/lib/superdev-core/src/manifest.rs crates/lib/superdev-core/src/engine.rs crates/app/superdev/src/manage.rs && git commit -S -m "feat(core): resolve components from per-provider registry entries"`

---

### Task 2: Lock `owners` attribution

**Files:**
- Modify: `crates/lib/superdev-core/src/lock.rs`
- Modify: `crates/lib/superdev-core/src/engine.rs` (removal loop drops owners with files)
- Modify: `crates/app/superdev/src/manage.rs` (released/gone drops and `prune_custom_skills` drop owners too)

**Interfaces:**
- Produces: `Lock.owners: BTreeMap<String, String>` — `files` key → capability name, present only for entries written from a provider checkout rather than embedded content. `#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]`, so every existing lock reads unchanged and locks without materialised files serialise without the table.
- Invariant later tasks rely on: every `owners` key is also a `files` key; whoever removes a `files` key removes its `owners` key in the same breath.

- [ ] **Step 1: Write the failing tests**

`lock.rs`:

```rust
    #[test]
    fn owners_round_trip_and_stay_optional() {
        let dir = tempfile::tempdir().unwrap();
        let mut lock = Lock::default();
        lock.files
            .insert(".claude/skills/tdd/SKILL.md".into(), sha256_hex(b"x"));
        lock.owners
            .insert(".claude/skills/tdd/SKILL.md".into(), "workflows".into());
        lock.save(dir.path()).unwrap();
        assert_eq!(Lock::load(dir.path()).unwrap(), lock);
        // Without owners the table is absent entirely.
        let plain = Lock::default();
        plain.save(dir.path()).unwrap();
        let text = std::fs::read_to_string(dir.path().join(LOCK_PATH)).unwrap();
        assert!(!text.contains("owners"), "{text}");
    }
```

Extend `a_0_1_0_lock_reads_unchanged`: after the existing assertions, `assert!(lock.owners.is_empty());`.

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev-core -- owners_round_trip` — expected: compile error, no `owners` field.

- [ ] **Step 3: Implement**

Add to `Lock`:

```rust
    /// Which capability materialised each `files` entry, for entries copied
    /// from a provider checkout rather than embedded content. Everything
    /// else never appears here.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub owners: BTreeMap<String, String>,
```

In `engine.rs` `apply_entry`, the removal loop becomes:

```rust
        for key in removed {
            lock.files.remove(&key);
            lock.owners.remove(&key);
        }
```

In `manage.rs`, `sync`'s released/gone drop gains `lock.owners.remove(key);` beside the files removal, and `prune_custom_skills` removes the owners entry beside each files entry it prunes.

- [ ] **Step 4: Run the workspace tests** — `cargo nextest run --workspace` — expected: PASS.

- [ ] **Step 5: Commit** — `git add crates/lib/superdev-core/src/lock.rs crates/lib/superdev-core/src/engine.rs crates/app/superdev/src/manage.rs && git commit -S -m "feat(core): attribute materialised lock entries to their capability"`

---

### Task 3: The `MaterialiseSkills` engine action

The engine learns to copy a mise-pinned checkout's skill directories into `.claude/skills/` as owned, attributed files — and to reconcile: attributed entries the checkout no longer ships are removed (unmodified) or released (user-edited), the same semantics as `RemoveFile`.

**Files:**
- Modify: `crates/lib/superdev-core/src/action.rs`
- Modify: `crates/lib/superdev-core/src/engine.rs`

**Interfaces:**
- Produces:

```rust
    /// Copy the workflows provider's pinned checkout into `.claude/skills/`,
    /// one owned file at a time, attributed in the lock. The checkout is
    /// resolved through mise at apply time, so planning needs no checkout.
    MaterialiseSkills {
        /// mise tool key holding the checkout, e.g. `http:mattpocock-skills`.
        tool: String,
        /// Checkout-relative directories each holding skill directories.
        source_dirs: Vec<String>,
        /// Skill names released to the user; never written, never attributed.
        custom: Vec<String>,
    },
```

- `describe()`: `materialise <tool> skills into .claude/skills/`.
- Engine semantics: resolve the checkout via `mise where <tool>` (a failure is `Failed`, never optional); enumerate each `source_dir`'s subdirectories as skills (files directly in a source_dir, like the upstream `README.md`, are ignored); for each non-custom skill, walk its files recursively and write each to `.claude/skills/<name>/<relative path>` when content differs (backup + journal, note-counting user edits against `prior_hashes`), recording `(key, hash)` for the lock and the key for `owners` attribution either way — an unchanged file stays claimed. Then reconcile: every lock key whose `owners` value is this entry's capability and which the new set does not contain is removed via the existing `remove_file` semantics (gone → dropped; unmodified → backup + delete; user-edited → left, released). Non-UTF-8 or unreadable checkout content is `Failed` — the run unwinds. One aggregate outcome note: `wrote <W>, kept <K>, removed <R>, released <L>` with the user-edit count appended as `; overwrote <E> user-edited (backed up)` when non-zero.
- `install_committed_pins`'s trigger predicate now counts `MaterialiseSkills` as an action that needs the pinned tools present, exactly like `Run`.
- `apply_entry` gains an `attributed: Vec<String>` collected per entry; at entry completion each attributed key gets `lock.owners.insert(key, capability.as_str().to_string())` (repo-level entries never materialise, so `entry.capability` is present by construction — return `Failed` from the action if it is not, rather than panicking).

- [ ] **Step 1: Write the failing tests**

In `engine.rs` tests (a shared fixture helper first):

```rust
    /// A fake checkout: skills/engineering/{alpha,beta}, skills/productivity/gamma.
    /// alpha has a nested reference file; a stray README sits beside the dirs.
    fn fake_checkout() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let write = |rel: &str, content: &str| {
            let p = dir.path().join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, content).unwrap();
        };
        write("skills/engineering/alpha/SKILL.md", "alpha v1");
        write("skills/engineering/alpha/refs/DEEP.md", "alpha deep");
        write("skills/engineering/beta/SKILL.md", "beta v1");
        write("skills/engineering/README.md", "not a skill");
        write("skills/productivity/gamma/SKILL.md", "gamma v1");
        dir
    }

    fn materialise_action(custom: &[&str]) -> Action {
        Action::MaterialiseSkills {
            tool: "http:mattpocock-skills".into(),
            source_dirs: vec!["skills/engineering".into(), "skills/productivity".into()],
            custom: custom.iter().map(|c| (*c).to_string()).collect(),
        }
    }

    fn where_scripted(checkout: &std::path::Path) -> FakeRunner {
        let fake = FakeRunner::new();
        fake.script(
            "mise where http:mattpocock-skills",
            Output {
                status: 0,
                stdout: format!("{}\n", checkout.display()),
                stderr: String::new(),
            },
        );
        fake
    }

    #[test]
    fn materialise_writes_locks_and_attributes() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        let fake = where_scripted(checkout.path());
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&[])],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{:?}", result.reports);
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".claude/skills/alpha/refs/DEEP.md")).unwrap(),
            "alpha deep"
        );
        assert!(dir.path().join(".claude/skills/gamma/SKILL.md").exists());
        // The stray README is not a skill directory and is not copied.
        assert!(!dir.path().join(".claude/skills/README.md").exists());
        let key = ".claude/skills/alpha/SKILL.md";
        assert_eq!(lock.files[key], sha256_hex(b"alpha v1"));
        assert_eq!(lock.owners[key], "workflows");
        assert_eq!(lock.owners.len(), 4);
    }

    #[test]
    fn materialise_reconciles_dropped_skills_and_skips_custom() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        let fake = where_scripted(checkout.path());
        let manifest = Manifest::default_for("0.1.0", &[]);
        // A previously materialised skill the checkout no longer ships…
        std::fs::create_dir_all(dir.path().join(".claude/skills/old")).unwrap();
        std::fs::write(dir.path().join(".claude/skills/old/SKILL.md"), "old v1").unwrap();
        // …and one the user edited since.
        std::fs::create_dir_all(dir.path().join(".claude/skills/mine")).unwrap();
        std::fs::write(dir.path().join(".claude/skills/mine/SKILL.md"), "edited").unwrap();
        let mut lock = Lock::default();
        for (key, content) in [
            (".claude/skills/old/SKILL.md", "old v1"),
            (".claude/skills/mine/SKILL.md", "mine v1"),
        ] {
            lock.files.insert(key.into(), sha256_hex(content.as_bytes()));
            lock.owners.insert(key.into(), "workflows".into());
        }
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&["beta"])],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{:?}", result.reports);
        // Dropped and unmodified: deleted, with a backup.
        assert!(!dir.path().join(".claude/skills/old/SKILL.md").exists());
        // User-edited: left in place, released from the lock.
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".claude/skills/mine/SKILL.md")).unwrap(),
            "edited"
        );
        assert!(!lock.files.contains_key(".claude/skills/old/SKILL.md"));
        assert!(!lock.files.contains_key(".claude/skills/mine/SKILL.md"));
        assert!(lock.owners.keys().all(|k| !k.contains("/old/") && !k.contains("/mine/")));
        // Custom skill: never written, never attributed.
        assert!(!dir.path().join(".claude/skills/beta").exists());
        assert!(!lock.files.keys().any(|k| k.contains("/beta/")));
        // The pinned tool is installed before the checkout is read.
        let calls = fake.calls();
        let install = calls.iter().position(|c| c.starts_with("mise install")).unwrap();
        let where_ = calls.iter().position(|c| c.starts_with("mise where")).unwrap();
        assert!(install < where_, "calls: {calls:?}");
    }

    #[test]
    fn materialise_failures_unwind() {
        // No checkout: `mise where` fails.
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        fake.script(
            "mise where http:mattpocock-skills",
            Output { status: 1, stdout: String::new(), stderr: "not installed".into() },
        );
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&[])],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert!(lock.files.is_empty());

        // Non-UTF-8 checkout content: fails and unwinds the files written first.
        let checkout = fake_checkout();
        std::fs::write(checkout.path().join("skills/productivity/gamma/SKILL.md"), [0xff, 0xfe])
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let fake = where_scripted(checkout.path());
        let mut lock = Lock::default();
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert!(
            !dir.path().join(".claude/skills/alpha/SKILL.md").exists(),
            "earlier writes must unwind"
        );
    }

    #[test]
    fn a_converged_materialise_rewrites_nothing() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&[])],
        }];
        let mut lock = Lock::default();
        let fake = where_scripted(checkout.path());
        assert!(apply(dir.path(), &fake, &manifest, &planned, &mut lock).ok);
        let fake = where_scripted(checkout.path());
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        // Unchanged files leave no fresh backups behind.
        let backups = std::fs::read_dir(dir.path().join(BACKUP_DIR))
            .map(|d| d.count())
            .unwrap_or(0);
        assert_eq!(backups, 1, "only the first run's stamp dir may exist");
        assert_eq!(lock.owners.len(), 4);
    }
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev-core -- materialise a_converged_materialise` — expected: compile error on the new variant.

- [ ] **Step 3: Implement**

`action.rs`: the variant (doc comments per the Interfaces block) plus the `describe()` arm:

```rust
            Action::MaterialiseSkills { tool, .. } => {
                format!("materialise {tool} skills into .claude/skills/")
            }
```

`engine.rs`:
- `apply_entry` gains `let mut attributed: Vec<String> = Vec::new();`, an arm:

```rust
                Action::MaterialiseSkills { tool, source_dirs, custom } => self
                    .materialise_skills(
                        entry.capability,
                        tool,
                        source_dirs,
                        custom,
                        &mut written,
                        &mut attributed,
                        &mut removed,
                    ),
```

and, after the existing insertions at entry completion:

```rust
        for key in attributed {
            if let Some(capability) = entry.capability {
                lock.owners.insert(key, capability.as_str().to_string());
            }
        }
```

(place this before the `removed` loop so a reconciled removal wins over a stale attribution — order it: written-insert, attributed-insert, removed-drop).
- The method, beside the removal methods:

```rust
    /// Copy a pinned checkout's skill directories into the repo, then
    /// reconcile: attributed entries the checkout no longer ships leave by
    /// the same rules as RemoveFile. One aggregate outcome carries the counts.
    fn materialise_skills(
        &mut self,
        capability: Option<Capability>,
        tool: &str,
        source_dirs: &[String],
        custom: &[String],
        written: &mut Vec<(String, String)>,
        attributed: &mut Vec<String>,
        removed: &mut Vec<String>,
    ) -> ActionOutcome {
        let Some(capability) = capability else {
            return ActionOutcome::Failed("materialise needs an owning capability".into());
        };
        let args = vec!["where".to_string(), tool.to_string()];
        let checkout = match self.runner.run("mise", &args, self.root) {
            Ok(out) if out.status == 0 => std::path::PathBuf::from(out.stdout.trim()),
            Ok(out) => {
                return ActionOutcome::Failed(
                    Error::Command {
                        command: command_line("mise", &args),
                        status: Some(out.status),
                        stderr: out.stderr,
                    }
                    .to_string(),
                );
            }
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let mut wrote = 0usize;
        let mut kept = 0usize;
        let mut edited = 0usize;
        let mut fresh: Vec<String> = Vec::new();
        for source_dir in source_dirs {
            let dir = checkout.join(source_dir);
            let entries = match fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(e) => {
                    return ActionOutcome::Failed(
                        Error::Io { path: dir, source: e }.to_string(),
                    );
                }
            };
            for entry in entries {
                let Ok(entry) = entry else { continue };
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if custom.contains(&name) {
                    continue;
                }
                let mut files = Vec::new();
                if let Err(e) = collect_files(&path, &mut files) {
                    return ActionOutcome::Failed(e.to_string());
                }
                for file in files {
                    let rel = file
                        .strip_prefix(&path)
                        .expect("collected under this skill directory");
                    let target = format!(
                        ".claude/skills/{name}/{}",
                        rel.display().to_string().replace('\\', "/")
                    );
                    let content = match read_text(&file) {
                        Ok(Some(content)) => content,
                        Ok(None) => continue,
                        Err(e) => return ActionOutcome::Failed(e.to_string()),
                    };
                    fresh.push(target.clone());
                    let existing = match read_text(&self.root.join(&target)) {
                        Ok(existing) => existing,
                        Err(e) => return ActionOutcome::Failed(e.to_string()),
                    };
                    if existing.as_deref() == Some(content.as_str()) {
                        kept += 1;
                        written.push((target.clone(), sha256_hex(content.as_bytes())));
                        attributed.push(target);
                        continue;
                    }
                    let mut probe = Vec::new();
                    match self.write_action(&target, &content, Ownership::Owned, &mut probe) {
                        ActionOutcome::Applied { note } => {
                            wrote += 1;
                            edited += usize::from(note.is_some());
                        }
                        ActionOutcome::Failed(e) => return ActionOutcome::Failed(e),
                        ActionOutcome::Skipped(_) => unreachable!("owned writes never skip"),
                    }
                    written.append(&mut probe);
                    attributed.push(target);
                }
            }
        }
        // Reconcile: what this capability had materialised and the checkout
        // no longer ships leaves by the RemoveFile rules.
        let mut released = 0usize;
        let mut swept = 0usize;
        let stale: Vec<String> = self
            .prior_owners
            .iter()
            .filter(|(key, owner)| {
                owner.as_str() == capability.as_str() && !fresh.contains(key)
            })
            .map(|(key, _)| key.clone())
            .collect();
        for key in stale {
            match self.remove_file(&key, removed) {
                ActionOutcome::Applied { .. } => swept += 1,
                ActionOutcome::Skipped(reason) if reason == "already gone" => {}
                ActionOutcome::Skipped(_) => released += 1,
                ActionOutcome::Failed(e) => return ActionOutcome::Failed(e),
            }
        }
        let mut note = format!("wrote {wrote}, kept {kept}, removed {swept}, released {released}");
        if edited > 0 {
            note.push_str(&format!("; overwrote {edited} user-edited (backed up)"));
        }
        ActionOutcome::Applied { note: Some(note) }
    }
```

with a free helper beside `read_text`:

```rust
/// Every file under `dir`, recursively, in sorted order.
fn collect_files(dir: &Path, into: &mut Vec<std::path::PathBuf>) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| Error::Io { path: dir.into(), source: e })?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_files(&path, into)?;
        } else {
            into.push(path);
        }
    }
    Ok(())
}
```

- `Session` gains `prior_owners: BTreeMap<String, String>` cloned from `lock.owners` in `Session::new`, mirroring `prior_hashes` — reconciliation must see the run-start attribution, and `remove_file`'s hash re-check already reads `prior_hashes`.
- `install_committed_pins`: the predicate becomes

```rust
        let runs = |p: &&Planned| {
            p.actions.iter().any(|a| {
                matches!(a, Action::Run { .. } | Action::MaterialiseSkills { .. })
            })
        };
```

and `apply_pins`' trailing install already covers the pin-edit path.

- [ ] **Step 4: Run the crate tests** — `cargo test -p superdev-core` — expected: PASS.

- [ ] **Step 5: Commit** — `git add crates/lib/superdev-core/src/action.rs crates/lib/superdev-core/src/engine.rs && git commit -S -m "feat(engine): materialise a pinned checkout as owned, attributed skills"`

---

### Task 4: The mattpocock-skills component

**Files:**
- Create: `crates/lib/superdev-core/src/components/mattskills.rs`
- Modify: `crates/lib/superdev-core/src/registry.rs` (constants + the second workflows entry, `default: false`)
- Modify: `crates/lib/superdev-core/src/components/mod.rs` (module, `MANAGED_MISE_TOOLS`, `component_for` arm)

**Interfaces:**
- Produces, in `registry`:

```rust
/// Source tarball for the pinned mattpocock/skills release (mise `http` backend).
pub const MATTSKILLS_URL: &str =
    "https://github.com/mattpocock/skills/archive/refs/tags/v1.2.3.tar.gz";
/// sha256 of that tarball.
pub const MATTSKILLS_CHECKSUM: &str =
    "sha256:238fac54d0f53d3e2d0501c1b38c9c0e4e9bc26f6b057b53a7328ea15d43b66f";
```

plus the entry `RegistryEntry { capability: Capability::Workflows, provider: "mattpocock-skills", version: Some("1.2.3"), available: true, default: false }` placed immediately after the superpowers entry (`ENTRIES` becomes length 6).
- Produces, in `mattskills`:
  - `pub const MATTSKILLS_MISE_TOOL: &str = "http:mattpocock-skills";`
  - `pub const MATTSKILLS_SKILLS: [&str; 25]` — the names from Global Constraints, engineering set then productivity set, each sorted.
  - `pub struct MattSkills;` implementing `Component` — `capability()` = `Workflows`, `provider()` = `"mattpocock-skills"`.
- `plan()`: rejects a version off the registry entry (`workflows version must match the registry default 1.2.3 — the pinned checksum is the provenance`); plans the `SetMisePin` when the normalised current pin differs (same round-trip comparison as `plugin.rs` / `codegraph.rs`, value `{ version = "1.2.3", url = "<MATTSKILLS_URL>", checksum = "<MATTSKILLS_CHECKSUM>", strip_components = 1 }`); plans `MaterialiseSkills` when a refresh is due. Refresh predicate, all local: the lock's workflows record is absent, or its provider/version differ from the manifest, or no lock entry is attributed to workflows, or any workflows-attributed file is missing on disk or hashes differently from the lock.
- `owned()`: `Claim::MisePin(MATTSKILLS_MISE_TOOL)` plus `Claim::File(key)` for every `ctx.lock.owners` entry whose value is `"workflows"`.
- Consumes: `Action::MaterialiseSkills` (Task 3), `Claim` (SP4), `mise::{current_pin, set_pin}`.

- [ ] **Step 1: Write the failing tests** (`mattskills.rs` tests mod)

```rust
    fn ctx_parts() -> (Manifest, Lock) {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "mattpocock-skills".into();
        workflows.version = Some("1.2.3".into());
        (manifest, Lock::default())
    }

    #[test]
    fn a_fresh_repo_plans_pin_and_materialise() {
        let dir = tempfile::tempdir().unwrap();
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        let ctx = Ctx { root: dir.path(), runner: &fake, manifest: &manifest, lock: &lock };
        let descs: Vec<String> =
            MattSkills.plan(&ctx).unwrap().iter().map(|a| a.describe()).collect();
        assert!(descs.iter().any(|d| d.contains("pin http:mattpocock-skills")), "{descs:?}");
        assert!(
            descs.contains(&"materialise http:mattpocock-skills skills into .claude/skills/".to_string()),
            "{descs:?}"
        );
        assert!(fake.calls().is_empty(), "planning must run nothing");
    }

    #[test]
    fn a_converged_repo_plans_nothing_and_owns_its_files() {
        let dir = tempfile::tempdir().unwrap();
        let (manifest, mut lock) = ctx_parts();
        std::fs::write(
            dir.path().join(".mise.toml"),
            crate::components::mise::set_pin("", MATTSKILLS_MISE_TOOL, &pin_value()).unwrap(),
        )
        .unwrap();
        let skill = dir.path().join(".claude/skills/tdd/SKILL.md");
        std::fs::create_dir_all(skill.parent().unwrap()).unwrap();
        std::fs::write(&skill, "tdd content").unwrap();
        lock.files.insert(
            ".claude/skills/tdd/SKILL.md".into(),
            crate::lock::sha256_hex(b"tdd content"),
        );
        lock.owners
            .insert(".claude/skills/tdd/SKILL.md".into(), "workflows".into());
        lock.components.insert(
            "workflows".into(),
            crate::lock::LockedComponent {
                provider: "mattpocock-skills".into(),
                version: Some("1.2.3".into()),
            },
        );
        let fake = FakeRunner::new();
        let ctx = Ctx { root: dir.path(), runner: &fake, manifest: &manifest, lock: &lock };
        assert!(MattSkills.plan(&ctx).unwrap().is_empty());
        let keys: Vec<String> = MattSkills.owned(&ctx).iter().map(Claim::lock_key).collect();
        assert!(keys.contains(&".mise.toml:http:mattpocock-skills".to_string()));
        assert!(keys.contains(&".claude/skills/tdd/SKILL.md".to_string()));

        // Drift in one materialised file replans the refresh.
        std::fs::write(&skill, "edited").unwrap();
        let actions = MattSkills.plan(&ctx).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(actions[0].describe().contains("materialise"), "{actions:?}");
    }

    #[test]
    fn a_foreign_version_pin_is_rejected_and_custom_rides_the_action() {
        let dir = tempfile::tempdir().unwrap();
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("workflows").unwrap().version = Some("9.9.9".into());
        let fake = FakeRunner::new();
        let ctx = Ctx { root: dir.path(), runner: &fake, manifest: &manifest, lock: &lock };
        assert!(MattSkills.plan(&ctx).is_err());

        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("workflows").unwrap().custom = vec!["grill-me".into()];
        let ctx = Ctx { root: dir.path(), runner: &fake, manifest: &manifest, lock: &lock };
        let custom = MattSkills.plan(&ctx).unwrap().into_iter().find_map(|a| match a {
            Action::MaterialiseSkills { custom, .. } => Some(custom),
            _ => None,
        });
        assert_eq!(custom.unwrap(), vec!["grill-me".to_string()]);
    }
```

Also in `components/mod.rs` tests:

```rust
    #[test]
    fn the_workflows_provider_resolves_from_the_manifest() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "mattpocock-skills".into();
        workflows.version = Some("1.2.3".into());
        let components = enabled(&manifest).unwrap();
        assert!(components.iter().any(|c| c.provider() == "mattpocock-skills"));
        assert!(!components.iter().any(|c| c.provider() == "superpowers"));
    }
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev-core -- mattskills the_workflows_provider` — expected: compile errors.

- [ ] **Step 3: Implement**

Registry constants + entry per Interfaces. `mattskills.rs`:

```rust
//! components/mattskills.rs — the workflows capability via mattpocock/skills,
//! materialised into the repo as owned files. A collaborator gets working
//! skills from git alone; nothing is installed at user level.

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::error::{Error, Result};
use crate::registry::{self, MATTSKILLS_CHECKSUM, MATTSKILLS_URL};

/// mise `[tools]` key for the pinned checkout.
pub const MATTSKILLS_MISE_TOOL: &str = "http:mattpocock-skills";

/// Checkout directories holding one skill directory each.
const SOURCE_DIRS: [&str; 2] = ["skills/engineering", "skills/productivity"];

/// Upstream skill names at the pinned version, for init adoption and custom
/// reporting. Refresh together with the version, url and checksum.
pub const MATTSKILLS_SKILLS: [&str; 25] = [
    "ask-matt",
    "code-review",
    "codebase-design",
    "diagnosing-bugs",
    "domain-modeling",
    "grill-with-docs",
    "implement",
    "improve-codebase-architecture",
    "prototype",
    "research",
    "resolving-merge-conflicts",
    "setup-matt-pocock-skills",
    "tdd",
    "to-spec",
    "to-tickets",
    "triage",
    "wayfinder",
    "wizard",
    "grill-me",
    "grilling",
    "handoff",
    "teach",
    "to-questionnaire",
    "wait-what",
    "writing-for-agents",
];

/// The mattpocock-skills provider.
pub struct MattSkills;

/// The `.mise.toml` value for the pinned release.
fn pin_value() -> String {
    let version = registry::entry_for(Capability::Workflows, "mattpocock-skills")
        .and_then(|e| e.version)
        .expect("registry pins mattpocock-skills");
    format!(
        "{{ version = \"{version}\", url = \"{MATTSKILLS_URL}\", checksum = \"{MATTSKILLS_CHECKSUM}\", strip_components = 1 }}"
    )
}

impl Component for MattSkills {
    fn capability(&self) -> Capability {
        Capability::Workflows
    }

    fn provider(&self) -> &'static str {
        "mattpocock-skills"
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        let config = ctx
            .config(Capability::Workflows)
            .expect("planned only when enabled");
        let default = registry::entry_for(Capability::Workflows, "mattpocock-skills")
            .and_then(|e| e.version)
            .expect("registry pins mattpocock-skills");
        if config.version.as_deref() != Some(default) {
            return Err(Error::Manifest {
                message: format!(
                    "workflows version must match the registry default {default} — the pinned checksum is the provenance"
                ),
            });
        }
        let mut actions = Vec::new();
        let value = pin_value();
        let current = match std::fs::read_to_string(ctx.root.join(".mise.toml")) {
            Ok(s) => super::mise::current_pin(&s, MATTSKILLS_MISE_TOOL)?,
            Err(_) => None,
        };
        // Round-trip the desired value so layout differences never read as drift.
        let desired = super::mise::set_pin("", MATTSKILLS_MISE_TOOL, &value)
            .and_then(|s| super::mise::current_pin(&s, MATTSKILLS_MISE_TOOL))?
            .expect("pin just set");
        if current.as_deref() != Some(desired.as_str()) {
            actions.push(Action::SetMisePin {
                tool: MATTSKILLS_MISE_TOOL.into(),
                value_toml: value,
            });
        }
        if refresh_due(ctx, config) {
            actions.push(Action::MaterialiseSkills {
                tool: MATTSKILLS_MISE_TOOL.into(),
                source_dirs: SOURCE_DIRS.iter().map(|d| (*d).to_string()).collect(),
                custom: config.custom.clone(),
            });
        }
        Ok(actions)
    }

    fn owned(&self, ctx: &Ctx<'_>) -> Vec<Claim> {
        let mut claims = vec![Claim::MisePin(MATTSKILLS_MISE_TOOL.to_string())];
        claims.extend(
            ctx.lock
                .owners
                .iter()
                .filter(|(_, owner)| owner.as_str() == Capability::Workflows.as_str())
                .map(|(key, _)| Claim::File(key.clone())),
        );
        claims
    }
}

/// Whether the materialised set needs refreshing — all answered from the
/// lock and the working tree, so `status` needs neither network nor checkout.
fn refresh_due(ctx: &Ctx<'_>, config: &crate::manifest::CapabilityConfig) -> bool {
    let applied = ctx.lock.components.get(Capability::Workflows.as_str());
    let recorded = applied.is_some_and(|a| {
        a.provider == "mattpocock-skills" && a.version == config.version
    });
    let attributed: Vec<&String> = ctx
        .lock
        .owners
        .iter()
        .filter(|(_, owner)| owner.as_str() == Capability::Workflows.as_str())
        .map(|(key, _)| key)
        .collect();
    if !recorded || attributed.is_empty() {
        return true;
    }
    attributed.into_iter().any(|key| {
        let locked = &ctx.lock.files[key];
        match std::fs::read_to_string(ctx.root.join(key)) {
            Ok(content) => crate::lock::sha256_hex(content.as_bytes()) != *locked,
            Err(_) => true,
        }
    })
}
```

`components/mod.rs`: `pub mod mattskills;`, `MANAGED_MISE_TOOLS` becomes a 3-array adding `mattskills::MATTSKILLS_MISE_TOOL`, and `component_for` gains `(Capability::Workflows, "mattpocock-skills") => Box::new(mattskills::MattSkills),`.

Note on `refresh_due`'s `&ctx.lock.files[key]` index: the owners⊆files invariant (Task 2) guarantees presence; still, prefer `ctx.lock.files.get(key)` with a `return true` on `None`, so a hand-edited lock degrades to a refresh instead of a panic. Write it with `get`.

- [ ] **Step 4: Run the workspace tests** — `cargo nextest run --workspace` — expected: PASS (default is still superpowers; nothing else changes behaviour).

- [ ] **Step 5: Commit** — `git add crates/lib/superdev-core/src/components/mattskills.rs crates/lib/superdev-core/src/components/mod.rs crates/lib/superdev-core/src/registry.rs && git commit -S -m "feat(workflows): add the mattpocock-skills provider"`

---

### Task 5: Skill pack drops grill-me; unknown custom names soften

**Files:**
- Modify: `crates/lib/superdev-core/src/components/skillpack.rs`
- Delete: `crates/lib/superdev-core/assets/skills/grill-me/` (the whole directory, via `git rm -r`)
- Modify: `crates/app/superdev/tests/cli.rs` (tests that count five skills or use grill-me as a fixture)
- Modify: this repo's own `.claude/skills/` + `.superdev/lock.toml` via `scripts/superdev sync`

**Interfaces:**
- Produces: `SKILLS` becomes `[(&str, &str); 4]` — `aokf-maintain`, `double-check`, `humanise`, `self-improve`. `SkillPack::plan` no longer errors on a custom name outside the pack; it ignores it (the report line lands in Task 7). The existing behaviours for known custom names are untouched.
- The repo self-sync is part of this task: without it, `npm run check:blueprint` exits 1 from this commit on (the pack no longer claims this repo's materialised `grill-me`, so status plans its sweep).

- [ ] **Step 1: Adjust the tests**

In `skillpack.rs`: `a_fresh_repo_plans_every_skill_and_the_hook` asserts `actions.len() == 5` (4 skills + hook); replace `an_unknown_custom_name_is_rejected` with:

```rust
    #[test]
    fn an_unknown_custom_name_is_ignored_by_planning() {
        let dir = tempfile::tempdir().unwrap();
        converge(dir.path());
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("skills").unwrap().custom = vec!["grill-me".into()];
        let fake = FakeRunner::new();
        let ctx = Ctx { root: dir.path(), runner: &fake, manifest: &manifest, lock: &lock };
        assert!(SkillPack.plan(&ctx).unwrap().is_empty());
    }
```

(`converge` writes only the four remaining skills once `SKILLS` shrinks.) In `cli.rs`, update tests that enumerate the pack or use `grill-me` as a fixture — switch fixtures to `humanise` or `double-check`; `disabling_skills_sweeps_them_and_releases_the_users_edit` already uses grill-me as the swept skill: change to `double-check`.

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev-core -- skillpack` — expected: FAIL (pack still ships five; plan still errors on unknown names).

- [ ] **Step 3: Implement** — remove the `grill-me` line from `SKILLS`, `git rm -r crates/lib/superdev-core/assets/skills/grill-me`, and delete the unknown-name rejection block from `plan()` (the `for name in &config.custom { … Error … }` loop goes entirely).

- [ ] **Step 4: Self-sync this repo** — `scripts/superdev sync` (or `cargo run --quiet -- sync`). Expected output: the orphan pass removes `.claude/skills/grill-me/SKILL.md` (backed up under `.superdev/cache/backup/`). Then `npm run check:blueprint` exits 0. Stage exactly: `git add -A crates/lib/superdev-core/assets/skills .claude/skills/grill-me .superdev/lock.toml .superdev/config.toml` (config only if the sync stamped it; check `git status` first and never stage `.claude/settings.json` or `.mise.toml`).

- [ ] **Step 5: Run the workspace tests** — `cargo nextest run --workspace && npm run check:blueprint` — expected: PASS / exit 0.

- [ ] **Step 6: Commit** — `git add crates/lib/superdev-core/src/components/skillpack.rs crates/app/superdev/tests/cli.rs` plus the Step 4 paths, `git commit -S -m "feat(skills): drop grill-me from the pack and tolerate stale custom names"`

---

### Task 6: CLI — `init --workflows-provider` and `update --provider`

**Files:**
- Modify: `crates/app/superdev/src/main.rs` (the `Update` variant gains `--provider`; `Init` uses `InitArgs` already)
- Modify: `crates/app/superdev/src/manage.rs`

**Interfaces:**
- Produces:
  - `InitArgs` gains `#[arg(long, value_name = "ID", conflicts_with = "no_workflows")] pub workflows_provider: Option<String>` (help: `Workflows provider (default: the registry default)`).
  - `main.rs`: `Update { target: Option<String>, #[arg(long, value_name = "ID")] provider: Option<String> }`; call becomes `manage::update(&root()?, target.as_deref(), provider.as_deref())`.
  - `manage::update(root, target, provider)`: `--provider` without a target errors (`--provider needs a capability target`); with a target, the pair must exist in the registry (`entry_for`), else the same `<capability> provider must be one of: …` message; on success the config's `provider` and `version` are set from the entry, then sync. `--provider` combined with `<capability>@<version>` errors for binary-pinned capabilities exactly as today (parse_target already rejects the version).
  - `init`: when `workflows_provider` names a provider, validate via `entry_for(Capability::Workflows, id)` (same error message) and rewrite the manifest's workflows entry to that provider and its registry version before `adopt_existing_skills`/save.

- [ ] **Step 1: Write the failing tests** (`manage.rs` unit tests; e2e in Task 10)

```rust
    #[test]
    fn init_args_carry_a_validated_workflows_provider() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let args = InitArgs {
            no_workflows: false,
            no_frontend: true,
            no_skills: true,
            no_code_index: true,
            no_knowledge: true,
            workflows_provider: Some("flying".into()),
        };
        let err = init(dir.path(), &args).unwrap_err().to_string();
        assert!(err.contains("workflows provider must be one of"), "{err}");
        assert!(!dir.path().join(CONFIG_PATH).exists(), "nothing written on error");
    }

    #[test]
    fn update_provider_rules() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for(superdev_core::version(), &[]);
        manifest.save(dir.path()).unwrap();
        let err = update(dir.path(), None, Some("superpowers")).unwrap_err().to_string();
        assert!(err.contains("--provider needs a capability target"), "{err}");
        let err = update(dir.path(), Some("workflows"), Some("flying"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("workflows provider must be one of"), "{err}");
    }
```

(The success path of `update … --provider` runs a full sync and belongs to Task 10's sandbox e2e; these unit tests stop at validation.)

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev -- init_args_carry update_provider_rules` — expected: compile errors (field, signature).

- [ ] **Step 3: Implement**

In `init`, after the already-initialised guard and before `default_for`:

```rust
    if let Some(id) = &args.workflows_provider {
        if registry::entry_for(Capability::Workflows, id).is_none() {
            return Err(Error::Manifest {
                message: format!(
                    "workflows provider must be one of: {}",
                    registry::providers_for(Capability::Workflows).join(", ")
                ),
            });
        }
    }
```

and after `default_for`:

```rust
    if let (Some(id), Some(config)) = (
        &args.workflows_provider,
        manifest.capabilities.get_mut(Capability::Workflows.as_str()),
    ) {
        let entry = registry::entry_for(Capability::Workflows, id).expect("validated above");
        config.provider = entry.provider.to_string();
        config.version = entry.version.map(str::to_string);
    }
```

`update` becomes:

```rust
pub fn update(root: &Path, target: Option<&str>, provider: Option<&str>) -> Result<u8> {
    let mut manifest = load_manifest(root)?;
    match (target, provider) {
        (None, Some(_)) => {
            return Err(Error::Manifest {
                message: "--provider needs a capability target".into(),
            });
        }
        (Some(target), provider) => {
            let (capability, version) = parse_target(target)?;
            if let Some(id) = provider {
                if registry::entry_for(capability, id).is_none() {
                    return Err(Error::Manifest {
                        message: format!(
                            "{} provider must be one of: {}",
                            capability.as_str(),
                            registry::providers_for(capability).join(", ")
                        ),
                    });
                }
            }
            let config = manifest
                .capabilities
                .get_mut(capability.as_str())
                .ok_or_else(|| Error::Manifest {
                    message: format!("`{}` is not enabled", capability.as_str()),
                })?;
            if let Some(id) = provider {
                let entry = registry::entry_for(capability, id).expect("validated above");
                config.provider = entry.provider.to_string();
                config.version = entry.version.map(str::to_string);
            } else {
                config.version = version.or_else(|| registry_version(&manifest, capability));
            }
        }
        (None, None) => {
            for capability in Capability::ALL {
                if let Some(version) = registry_version(&manifest, capability) {
                    if let Some(config) = manifest.capabilities.get_mut(capability.as_str()) {
                        config.version = Some(version);
                    }
                }
            }
        }
    }
    manifest.save(root)?;
    sync(root, false)
}
```

Note the borrow order in the `Some(target)` arm: compute `version.or_else(|| registry_version(&manifest, capability))` needs `&manifest` while `config` borrows mutably — hoist the fallback before taking `get_mut` (`let fallback = registry_version(&manifest, capability);` then `config.version = version.or(fallback);`). Same hoisting for the `(None, None)` arm as written above. `main.rs`: add the `provider` arg and thread it through; the existing `update_refuses_hand_picked_checksum_pinned_versions` e2e keeps passing (no `--provider` there).

- [ ] **Step 4: Run the workspace tests** — `cargo nextest run --workspace` — expected: PASS.

- [ ] **Step 5: Commit** — `git add crates/app/superdev/src/main.rs crates/app/superdev/src/manage.rs && git commit -S -m "feat(cli): choose the workflows provider at init and update time"`

---

### Task 7: Adoption, pruning, and reports for workflows

**Files:**
- Modify: `crates/app/superdev/src/manage.rs`

**Interfaces:**
- Produces (all private to `manage`):
  - `adopt_existing_mattskills(root, &mut manifest) -> Vec<String>` — at `init`, when workflows is enabled with provider `mattpocock-skills`: any existing directory `.claude/skills/<name>` for a name in `mattskills::MATTSKILLS_SKILLS` goes into `[workflows] custom`, reported as `workflows: kept your <name> — marked custom in .superdev/config.toml`. (Content comparison is impossible before the checkout exists, so any existing directory counts — the report says what to do if the user wants it managed.) Called from `init` beside `adopt_existing_skills`.
  - `prune_custom_skills` extends to workflows: for each `[workflows] custom` name, remove every `lock.files`/`lock.owners` entry whose key starts with `.claude/skills/<name>/`. Rename the fn `prune_custom` (it now covers both capabilities); keep the return-bool contract.
  - `custom_lines` extends: for skills, names in `skillpack::SKILLS` print `skills: <name> custom, unmanaged`, others `skills: custom names unknown skill '<name>' — no effect`; for workflows (when provider is `mattpocock-skills`), names in `MATTSKILLS_SKILLS` print `workflows: <name> custom, unmanaged`, others `workflows: custom names unknown skill '<name>' — no effect`.
  - `switch_lines(manifest, lock) -> Vec<String>` — when the lock records a workflows provider different from the manifest's: always `workflows: update the .agents import in AGENTS.md for the new provider` (only when the knowledge capability is enabled), plus `` workflows: superpowers plugin left installed — `claude plugin uninstall superpowers` removes it `` when the lock's old provider was `superpowers`. Printed by `sync` before the dry-run gate and by `status` with the other report lines.
  - `sync` prints the setup hint after a successful apply whose plan contained a `MaterialiseSkills`: `workflows: run /setup-matt-pocock-skills in Claude Code to finish configuring`.
- Reports never affect exit codes (status's expression is untouched).

- [ ] **Step 1: Write the failing tests**

```rust
    #[test]
    fn adoption_marks_existing_upstream_skill_dirs_custom() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".claude/skills/tdd")).unwrap();
        std::fs::write(dir.path().join(".claude/skills/tdd/SKILL.md"), "mine").unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "mattpocock-skills".into();
        let lines = adopt_existing_mattskills(dir.path(), &mut manifest);
        assert_eq!(manifest.capabilities["workflows"].custom, ["tdd"]);
        assert_eq!(
            lines,
            vec!["workflows: kept your tdd — marked custom in .superdev/config.toml".to_string()]
        );
        // A superpowers manifest adopts nothing.
        let mut superpowers = Manifest::default_for("0.1.0", &[]);
        assert!(adopt_existing_mattskills(dir.path(), &mut superpowers).is_empty());
    }

    #[test]
    fn workflows_custom_entries_prune_files_and_owners() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "mattpocock-skills".into();
        workflows.custom = vec!["tdd".into()];
        let mut lock = Lock::default();
        for key in [".claude/skills/tdd/SKILL.md", ".claude/skills/tdd/refs/A.md"] {
            lock.files.insert(key.into(), "h".into());
            lock.owners.insert(key.into(), "workflows".into());
        }
        lock.files.insert(".claude/skills/wizard/SKILL.md".into(), "h".into());
        lock.owners.insert(".claude/skills/wizard/SKILL.md".into(), "workflows".into());
        assert!(prune_custom(&manifest, &mut lock));
        assert!(!lock.files.keys().any(|k| k.contains("/tdd/")));
        assert!(!lock.owners.keys().any(|k| k.contains("/tdd/")));
        assert!(lock.files.contains_key(".claude/skills/wizard/SKILL.md"));
    }

    #[test]
    fn custom_lines_and_switch_lines_report() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "mattpocock-skills".into();
        workflows.custom = vec!["tdd".into(), "flying".into()];
        manifest.capabilities.get_mut("skills").unwrap().custom =
            vec!["humanise".into(), "grill-me".into()];
        let lines = custom_lines(&manifest);
        assert!(lines.contains(&"workflows: tdd custom, unmanaged".to_string()));
        assert!(lines.contains(
            &"workflows: custom names unknown skill 'flying' — no effect".to_string()
        ));
        assert!(lines.contains(&"skills: humanise custom, unmanaged".to_string()));
        assert!(lines.contains(
            &"skills: custom names unknown skill 'grill-me' — no effect".to_string()
        ));

        let mut lock = Lock::default();
        lock.components.insert(
            "workflows".into(),
            superdev_core::lock::LockedComponent {
                provider: "superpowers".into(),
                version: Some("6.2.0".into()),
            },
        );
        let lines = switch_lines(&manifest, &lock);
        assert!(lines.iter().any(|l| l.contains("claude plugin uninstall superpowers")), "{lines:?}");
        assert!(lines.iter().any(|l| l.contains("update the .agents import")), "{lines:?}");
        // No switch, no lines.
        assert!(switch_lines(&manifest, &Lock::default()).is_empty() == false || true);
        let mut same = Lock::default();
        same.components.insert(
            "workflows".into(),
            superdev_core::lock::LockedComponent {
                provider: "mattpocock-skills".into(),
                version: Some("1.2.3".into()),
            },
        );
        assert!(switch_lines(&manifest, &same).is_empty());
    }
```

(Fix the deliberately odd line in the third test before committing: an empty lock has no workflows record, so `switch_lines(&manifest, &Lock::default())` must return empty — assert exactly that: `assert!(switch_lines(&manifest, &Lock::default()).is_empty());`.)

- [ ] **Step 2: Run to verify failure** — `cargo test -p superdev -- adoption_marks workflows_custom custom_lines_and_switch` — expected: compile errors.

- [ ] **Step 3: Implement** the four pieces per Interfaces. Wire-up: `init` calls `adopt_existing_mattskills` right after `adopt_existing_skills` and prints its lines with the others; `status` and `sync` print `switch_lines(&manifest, &lock)` beside the custom lines (both verbs, before the dry-run gate in sync); `sync` sets a `let materialising = planned.iter().any(|p| p.actions.iter().any(|a| matches!(a, Action::MaterialiseSkills { .. })));` before applying and, after a successful `apply_and_report`, prints the setup hint when `materialising`. Rename `prune_custom_skills` → `prune_custom` at both call sites, add the workflows branch, and keep pruning BEFORE `plan_all` in both verbs — the SP4 ordering comment applies to the workflows entries for the same reason. `use superdev_core::components::mattskills;` joins the imports.

- [ ] **Step 4: Run the workspace tests** — `cargo nextest run --workspace && npm run check:blueprint` — expected: PASS / exit 0.

- [ ] **Step 5: Commit** — `git add crates/app/superdev/src/manage.rs && git commit -S -m "feat(manage): adopt, prune and report the materialised workflows skills"`

---

### Task 8: Provider-matched knowledge override

**Files:**
- Create: `crates/lib/superdev-core/assets/agents/MATT-POCOCK-SKILLS.md`
- Modify: `crates/lib/superdev-core/assets/AGENTS.md` (the `@.agents/SUPERPOWERS.md` line becomes `{workflows_overrides}`)
- Modify: `crates/lib/superdev-core/src/components/aokf.rs`

**Interfaces:**
- Produces: the override file follows the workflows provider — `superpowers` → `.agents/SUPERPOWERS.md` (asset unchanged), `mattpocock-skills` → `.agents/MATT-POCOCK-SKILLS.md`, workflows disabled → neither. `Aokf::plan` and `Aokf::owned` both consult `ctx.manifest`; the override is `Ownership::Owned` with reason `workflows overrides`. The `FILES` table drops its SUPERPOWERS.md row; the override becomes plan-time logic.
- The `AGENTS.md` scaffold template line `@.agents/SUPERPOWERS.md` is replaced by the token line `{workflows_overrides}`; at plan time the token line is substituted with `@.agents/SUPERPOWERS.md`, `@.agents/MATT-POCOCK-SKILLS.md`, or removed entirely (token line deleted, no blank residue) when workflows is disabled. The substitution extends the existing `{name}` mechanism: `AGENTS.md` joins `NAMED_ASSET` as a templated asset — generalise to a `fn render(path, content, ctx) -> String` applied to every FILES entry.

- [ ] **Step 1: Write the new asset**

`assets/agents/MATT-POCOCK-SKILLS.md`, exactly:

```markdown
# Matt Pocock Skills Overrides

The [mattpocock/skills](https://github.com/mattpocock/skills) flow defaults
to writing specs, tickets and context docs under `docs/` and `.scratch/`.
This project keeps those documents in the AOKF bundle instead. These
overrides take precedence.

## Specs (to-spec, grill-with-docs)

Write specs to `knowledge/specs/YYYY-MM-DD-<topic>-design.md` as AOKF
concepts: `type: Spec`, a unique `id`, `status: draft` while in flight,
`stable` once implemented. Keep `knowledge/specs/index.md` current. Specs
are permanent decision records: when a spec lands, move the durable
knowledge into the core concepts and keep the spec as the record of why.

## Plans and tickets (wayfinder, to-tickets, implement)

Write plans and ticket sets to `knowledge/plans/YYYY-MM-DD-<feature>.md` as
AOKF concepts: `type: Plan`, a unique `id`, `status: draft`. Plans are
ephemeral: delete the file in the commit that completes the work — git
history is the archive. Declare link edges from the plan side only
(`implements` → the spec), so deleting the plan leaves no dangling
references in the bundle.

## Decisions and context (domain-modeling)

Record ADRs as AOKF concepts (`type: Decision`) in the bundle, not under
`docs/adr/`. Context that domain-modeling would put in `CONTEXT.md` belongs
in the bundle's architecture and glossary concepts. Never duplicate
knowledge between the bundle and files outside it.
```

- [ ] **Step 2: Write the failing tests** (`aokf.rs`)

```rust
    fn plan_with_provider(dir: &std::path::Path, provider: Option<&str>) -> Vec<Action> {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        match provider {
            Some(provider) => {
                let workflows = manifest.capabilities.get_mut("workflows").unwrap();
                workflows.provider = provider.into();
            }
            None => {
                manifest.capabilities.remove("workflows");
            }
        }
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx { root: dir, runner: &fake, manifest: &manifest, lock: &lock };
        Aokf.plan(&ctx).unwrap()
    }

    #[test]
    fn the_override_file_follows_the_workflows_provider() {
        let dir = tempfile::tempdir().unwrap();
        let writes = |actions: &[Action]| -> Vec<String> {
            actions
                .iter()
                .filter_map(|a| match a {
                    Action::WriteFile { path, .. } => Some(path.clone()),
                    _ => None,
                })
                .collect()
        };
        let superpowers = writes(&plan_with_provider(dir.path(), Some("superpowers")));
        assert!(superpowers.contains(&".agents/SUPERPOWERS.md".to_string()));
        assert!(!superpowers.contains(&".agents/MATT-POCOCK-SKILLS.md".to_string()));
        let matt = writes(&plan_with_provider(dir.path(), Some("mattpocock-skills")));
        assert!(matt.contains(&".agents/MATT-POCOCK-SKILLS.md".to_string()));
        assert!(!matt.contains(&".agents/SUPERPOWERS.md".to_string()));
        let none = writes(&plan_with_provider(dir.path(), None));
        assert!(!none.iter().any(|p| p.starts_with(".agents/SUPERPOWERS")
            || p.starts_with(".agents/MATT-POCOCK")));
    }

    #[test]
    fn the_scaffold_imports_match_the_provider() {
        let dir = tempfile::tempdir().unwrap();
        let agents_content = |provider: Option<&str>| {
            plan_with_provider(dir.path(), provider)
                .into_iter()
                .find_map(|a| match a {
                    Action::WriteFile { path, content, .. } if path == "AGENTS.md" => Some(content),
                    _ => None,
                })
                .unwrap()
        };
        assert!(agents_content(Some("superpowers")).contains("@.agents/SUPERPOWERS.md"));
        let matt = agents_content(Some("mattpocock-skills"));
        assert!(matt.contains("@.agents/MATT-POCOCK-SKILLS.md"));
        assert!(!matt.contains("SUPERPOWERS"));
        let none = agents_content(None);
        assert!(!none.contains("{workflows_overrides}"));
        assert!(!none.contains("SUPERPOWERS") && !none.contains("MATT-POCOCK"));
    }

    #[test]
    fn owned_follows_the_provider_too() {
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("workflows").unwrap().provider = "mattpocock-skills".into();
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx { root: dir.path(), runner: &fake, manifest: &manifest, lock: &lock };
        let keys: Vec<String> = Aokf.owned(&ctx).iter().map(Claim::lock_key).collect();
        assert!(keys.contains(&".agents/MATT-POCOCK-SKILLS.md".to_string()));
        assert!(!keys.contains(&".agents/SUPERPOWERS.md".to_string()));
    }
```

- [ ] **Step 3: Run to verify failure** — `cargo test -p superdev-core -- aokf` — expected: FAIL.

- [ ] **Step 4: Implement**

- `assets/AGENTS.md`: replace the `@.agents/SUPERPOWERS.md` line with `{workflows_overrides}`.
- `aokf.rs`: remove the SUPERPOWERS.md row from `FILES`; add constants:

```rust
/// The workflow-framework override each provider gets, as
/// (provider id, target path, embedded asset).
const WORKFLOW_OVERRIDES: [(&str, &str, &str); 2] = [
    (
        "superpowers",
        ".agents/SUPERPOWERS.md",
        asset!("agents/SUPERPOWERS.md"),
    ),
    (
        "mattpocock-skills",
        ".agents/MATT-POCOCK-SKILLS.md",
        asset!("agents/MATT-POCOCK-SKILLS.md"),
    ),
];

/// The enabled workflows provider's override, when there is one.
fn workflow_override(ctx: &Ctx<'_>) -> Option<(&'static str, &'static str)> {
    let config = ctx.config(Capability::Workflows)?;
    WORKFLOW_OVERRIDES
        .iter()
        .find(|(provider, ..)| *provider == config.provider)
        .map(|(_, path, content)| (*path, *content))
}
```

- In `plan()`, render every asset through one substitution point: the manifest rename keeps its `NAMED_ASSET` handling; `AGENTS.md` gets the token line handled as:

```rust
                "AGENTS.md" => match workflow_override(ctx) {
                    Some((path, _)) => content.replace("{workflows_overrides}", &format!("@{path}")),
                    None => content
                        .lines()
                        .filter(|l| *l != "{workflows_overrides}")
                        .collect::<Vec<_>>()
                        .join("\n")
                        + "\n",
                },
```

and after the `FILES` loop, plan the override itself exactly like an owned FILES row (compare, then `WriteFile { ownership: Owned, reason: "workflows overrides" }`) when `workflow_override(ctx)` is `Some`. An unknown provider yields no override (the run already fails in `enabled()` before aokf plans; returning none keeps this fn total).
- `owned()`: takes `ctx` (it already does) — append `Claim::File(path)` from `workflow_override(ctx)` instead of the FILES-derived SUPERPOWERS entry.
- Update the two replay-loop tests in `aokf.rs` for the changed action set (the fresh-plan test's byte-identity check must exempt `AGENTS.md` now that it is templated — exempt it beside `NAMED_ASSET`).

- [ ] **Step 5: Run the workspace tests** — `cargo nextest run --workspace && npm run check:blueprint` — expected: PASS / exit 0 (this repo's knowledge capability is off; nothing changes here).

- [ ] **Step 6: Commit** — `git add crates/lib/superdev-core/assets crates/lib/superdev-core/src/components/aokf.rs && git commit -S -m "feat(knowledge): ship the workflow override matching the provider"`

---

### Task 9: Flip the default

One-line registry change; the task's substance is updating everything that assumed the old default.

**Files:**
- Modify: `crates/lib/superdev-core/src/registry.rs` (swap the two workflows `default` flags)
- Modify: `crates/lib/superdev-core/src/manifest.rs`, `crates/lib/superdev-core/src/components/mod.rs`, `crates/lib/superdev-core/src/engine.rs`, `crates/lib/superdev-core/src/components/plugin.rs`, `crates/lib/superdev-core/src/components/skillpack.rs`, `crates/lib/superdev-core/src/components/aokf.rs` (tests referencing the default)
- Modify: `crates/app/superdev/tests/cli.rs`, `crates/app/superdev/tests/manage.rs`

**Interfaces:**
- Produces: `default_entry(Capability::Workflows).provider == "mattpocock-skills"`; a fresh `Manifest::default_for` carries `provider = "mattpocock-skills", version = "1.2.3"`.
- Known ripples (fix each; anything else that fails is the same class):
  - `registry.rs` test asserting the workflows default.
  - `manifest.rs` `default_manifest_round_trips` (workflows version 6.2.0 → 1.2.3).
  - `engine.rs` `plan_runs_every_component` (`planned[0].provider` is now `mattpocock-skills`; the version-breaking arm still errors).
  - `components/mod.rs` `owned_matches_what_apply_locks`: the mattskills component's apply needs `mise where http:mattpocock-skills` scripted to a fixture checkout — reuse Task 3's `fake_checkout` shape inline (the test builds its own tempdir fixture and scripts the FakeRunner before the loop; only the workflows component consumes it).
  - Components whose own tests build default manifests but pin superpowers behaviour (`plugin.rs` tests): construct their manifests explicitly with `provider = "superpowers", version = Some("6.2.0")` instead of relying on the default.
  - `tests/manage.rs`: `Sandbox`'s fake `mise` gains a `where` answer pointing at a fixture checkout directory the sandbox creates (`skills/engineering/tdd/SKILL.md` etc. — two or three small skills suffice); `pin_workflows` (replaces `6.2.0`) switches to replacing `1.2.3`; tests that assert `claude plugin marketplace add`/`plugin install superpowers` calls on default init now init with `--workflows-provider superpowers` or assert the materialise path instead — decide per test by what the test is actually proving and keep both paths covered overall.
  - `tests/cli.rs`: tests that init without fakes must keep passing — cli.rs inits use `--no-workflows` (no external commands available there); verify none relied on the workflows default.
- The default flip must leave the full suite green and `npm run check:blueprint` exit 0 (this repo pins nothing workflows-related in `.superdev/`).

- [ ] **Step 1: Flip** — swap the `default:` booleans on the two workflows entries.
- [ ] **Step 2: Run the suite and fix every ripple** — `cargo nextest run --workspace`; iterate until green, keeping each fix within the listed class (test updates and explicit-manifest construction only — production code changes here mean a real defect; stop and report if one appears).
- [ ] **Step 3: Verify** — `cargo nextest run --workspace && npm run check:blueprint && cargo clippy --workspace --all-targets && npm run fmt` — all clean.
- [ ] **Step 4: Commit** — `git add -u crates && git commit -S -m "feat(workflows): default new repos to mattpocock-skills"` (check `git status` first; only crate sources and tests should be modified).

---

### Task 10: End-to-end tests

**Files:**
- Test: `crates/app/superdev/tests/manage.rs`

**Interfaces:**
- Consumes: the Sandbox fixture checkout from Task 9 (`mise where http:mattpocock-skills` → a directory with at least `skills/engineering/tdd/SKILL.md`, `skills/engineering/to-spec/SKILL.md`, `skills/productivity/grill-me/SKILL.md`).

- [ ] **Step 1: Write the default-init test**

```rust
#[test]
fn init_materialises_the_default_workflows_provider() {
    // init (full blueprint) → exit 0. Assert:
    // - .claude/skills/tdd/SKILL.md exists with the fixture's content;
    // - .mise.toml carries http:mattpocock-skills with the pinned version;
    // - lock.toml has [owners] entries mapping the skill files to "workflows"
    //   and [components.workflows] provider = "mattpocock-skills";
    // - no `claude plugin install` call appears in the fake log;
    // - stdout contains "workflows: run /setup-matt-pocock-skills in Claude
    //   Code to finish configuring";
    // - `status` afterwards exits 0.
}
```

- [ ] **Step 2: Write the secondary-provider test**

```rust
#[test]
fn init_with_superpowers_reproduces_the_plugin_flow() {
    // init --workflows-provider superpowers → exit 0. Assert:
    // - the fake log contains `claude plugin marketplace add` and
    //   `claude plugin install superpowers@superpowers-dev`;
    // - .mise.toml pins http:superpowers and NOT http:mattpocock-skills;
    // - no .claude/skills/tdd directory exists;
    // - config.toml records provider = "superpowers", version = "6.2.0";
    // - `status` afterwards exits 0.
}
```

- [ ] **Step 3: Write the switch test (both directions)**

```rust
#[test]
fn update_provider_switches_and_sweeps_both_directions() {
    // Start from init --workflows-provider superpowers (previous test's setup).
    // `update workflows --provider mattpocock-skills` → exit 0. Assert:
    // - http:superpowers is gone from .mise.toml, http:mattpocock-skills present;
    // - skills materialised; lock has no ".mise.toml:http:superpowers" key;
    // - stdout contains "claude plugin uninstall superpowers" and
    //   "update the .agents import in AGENTS.md";
    // - `status` exits 0.
    // Then `update workflows --provider superpowers` → exit 0. Assert:
    // - the materialised skill files are gone (swept; backup dir exists);
    // - lock has no owners table entries; http:mattpocock-skills unpinned;
    // - `status` exits 0.
}
```

- [ ] **Step 4: Write the stale-custom test**

```rust
#[test]
fn a_stale_custom_name_reports_instead_of_failing() {
    // Default init; then append `custom = ["not-a-skill"]` to the [skills]
    // table in config.toml. `status` → exit 0, stdout contains
    // "skills: custom names unknown skill 'not-a-skill' — no effect".
    // `sync` → exit 0.
}
```

- [ ] **Step 5: Run, fix real defects only, full suite** — `cargo nextest run --workspace` — expected: PASS. A scenario failing against the implementation means a defect in Tasks 1-9: fix the defect (smallest change), note it in the commit body.
- [ ] **Step 6: Commit** — `git add crates/app/superdev/tests/manage.rs && git commit -S -m "test: end-to-end provider selection, materialisation and switching"`

---

### Task 11: Knowledge bundle and changelog

**Files:**
- Modify: `knowledge/architecture.md`, `knowledge/configuration.md`, `knowledge/api-contracts.md`, `knowledge/glossary.md`, `knowledge/technology-stack.md`, `knowledge/error-handling.md`, `CHANGELOG.md`
- Modify: `knowledge/specs/2026-08-17-workflows-provider-default-design.md` (frontmatter only: `status: draft` → `status: stable`)

**Interfaces:** read each concept before editing; match its register; the validator hook gates every edit. The behaviour being documented is Tasks 1-10's, exact strings from Global Constraints.

- [ ] **Step 1: `architecture.md`** — the capability-to-provider map section now describes real selection: the registry holds per-(capability, provider) entries with one default; workflows has two providers — `mattpocock-skills` (default; materialised owned files under `.claude/skills/`, pinned checkout as the source) and `superpowers` (Claude Code plugin, per-machine install). One sentence on why the default is repo-owned: nothing user-level.
- [ ] **Step 2: `configuration.md`** — the manifest section documents `provider` as a real choice for workflows with the two valid ids, `[workflows] custom`, and the example `[workflows]` table updated to the new default; the lock section documents the `owners` table (which capability materialised each entry; absent when nothing is materialised).
- [ ] **Step 3: `api-contracts.md`** — `init` gains `--workflows-provider <id>`; `update` gains `--provider <id>` (needs a capability target; the only CLI provider switch); the workflows bullet describes materialisation, the setup/uninstall/import report lines, and that both providers stay binary-pinned.
- [ ] **Step 4: `glossary.md`** — add **provider** (a registry-selectable implementation of a capability) and **materialised skill** (a skill copied from a pinned checkout into `.claude/skills/` as owned, lock-attributed files); update the skill-pack term if it names five skills.
- [ ] **Step 5: `technology-stack.md`** — the pinned-tool set gains `http:mattpocock-skills` v1.2.3 (MIT, attribution: Matt Pocock's skills repository).
- [ ] **Step 6: `error-handling.md`** — the exit-2 causes gain the unknown-provider error.
- [ ] **Step 7: `CHANGELOG.md`** — under `## [Unreleased]` → `### Added`:

```markdown
- Workflows provider selection: the manifest's `provider` field is now
  honoured, `init --workflows-provider <id>` and
  `update workflows --provider <id>` choose between `mattpocock-skills`
  (the new default — materialised into `.claude/skills/` as repo files, so
  collaborators need nothing installed) and `superpowers` (the plugin flow,
  unchanged). Switching sweeps the old provider's pin and files
- The knowledge scaffold's framework override now matches the workflows
  provider: `.agents/SUPERPOWERS.md` or `.agents/MATT-POCOCK-SKILLS.md`
```

and under `### Removed` (create the heading if absent):

```markdown
- The skill pack's `grill-me` — the default workflows provider ships its
  own; the next sync sweeps the packaged copy (a user-edited copy is left
  in place and released). A `[skills] custom` name that is no longer in
  the pack now reports instead of failing the plan
```

- [ ] **Step 8: Flip the spec status** and run `npm run check:aokf` (PASS at level 2), `npm run check:blueprint` (exit 0), `cargo nextest run --workspace` (PASS).
- [ ] **Step 9: Commit** — `git add knowledge/ CHANGELOG.md && git commit -S -m "docs: record workflows provider selection and the new default"`

---

### Task 12: Dogfood — switch this repo

**This task pauses for the user once**: the `.mise.toml` write. Everything else is normal work. Run it in the main session (controller-level guidance for whoever executes: the sync edits `.mise.toml`, which carries the user's uncommitted local edits — before committing, show the user the diff hunk the sync added and ask how they want it staged; `git add -p .mise.toml` staging only the `http:mattpocock-skills` hunk is the expected shape).

**Files:**
- Modify: `.superdev/config.toml` (add the `[workflows]` table)
- Create (by sync): `.claude/skills/<25 upstream skills>/…`, the `.mise.toml` pin, lock entries
- Create: `.agents/MATT-POCOCK-SKILLS.md`; Delete: `.agents/SUPERPOWERS.md`
- Modify: `AGENTS.md`, `knowledge/development-procedure.md`

- [ ] **Step 1: Enable workflows** — append to `.superdev/config.toml`:

```toml
[workflows]
provider = "mattpocock-skills"
version = "1.2.3"
```

- [ ] **Step 2: Sync** — `scripts/superdev sync`. Expected: the pin lands in `.mise.toml`, mise fetches the checksummed tarball, the 25 skills materialise into `.claude/skills/`, the lock gains the attributed entries, and the setup hint prints. `npm run check:blueprint` → exit 0.
- [ ] **Step 3: PAUSE — `.mise.toml`** — show the user the sync's `.mise.toml` hunk and agree the staging before any commit that includes it.
- [ ] **Step 4: Hand-apply what a managed knowledge repo would get** (this repo's knowledge capability is off):
  - `git rm .agents/SUPERPOWERS.md`; create `.agents/MATT-POCOCK-SKILLS.md` with exactly the Task 8 asset content.
  - In `AGENTS.md`, replace the line `@.agents/SUPERPOWERS.md` with `@.agents/MATT-POCOCK-SKILLS.md`.
- [ ] **Step 5: Rewrite the process docs** — in `knowledge/development-procedure.md`, replace Workflow item 1 with:

```markdown
1. Significant changes follow the mattpocock/skills flow with this
   project's overrides
   ([MATT-POCOCK-SKILLS.md](/.agents/MATT-POCOCK-SKILLS.md)): grill the
   requirements and write the spec into `knowledge/specs/` (permanent
   decision record) with `to-spec`, then break it into an implementation
   plan in `knowledge/plans/` (ephemeral — deleted in the commit that
   completes the work) with `to-tickets` or `wayfinder`.
```

and in the "This repo manages its own skills" section, change "superdev fills the `skills` capability here and nothing else" to name both capabilities, e.g.: "superdev fills the `skills` and `workflows` capabilities here: committed `.superdev/config.toml` and `.superdev/lock.toml`, with `cargo run -- sync` writing the four pack skills, the PostToolUse hook entry, and the materialised mattpocock-skills set." Update the frontmatter `description` if its wording still says skills-only. Check `knowledge/index.md`'s line for this concept and mirror any description change. Grep `knowledge/` and `.agents/` for remaining `SUPERPOWERS.md` references (`grep -rn "SUPERPOWERS" knowledge/ .agents/ AGENTS.md CLAUDE.md`) and update factual references — historical mentions inside specs stay as written (specs are records).
- [ ] **Step 6: Verify** — `npm run check:aokf` (PASS at level 2), `npm run check:blueprint` (exit 0), `cargo nextest run --workspace` (PASS).
- [ ] **Step 7: Commit** — stage `.superdev/`, `.claude/skills/` (new skill files), `.agents/`, `AGENTS.md`, `knowledge/`, and the agreed `.mise.toml` hunk; `git commit -S -m "chore: switch this repo's workflows to mattpocock-skills"`.

---

## Self-Review

- **Spec coverage:** registry selection + CLI → Tasks 1, 6; materialised provider (fetch, materialise, attribution, custom, hints) → Tasks 2-4, 7; superpowers untouched as secondary → Task 1's resolution plus Task 9/10 tests; provider-matched override + scaffold token → Task 8; grill-me + custom softening → Task 5 (with the immediate self-sync so CI stays green mid-branch); default flip → Task 9; spec's testing list → Tasks 1-10 (each named scenario has a home); docs → Task 11; dogfood → Task 12. Gap check: the spec's "unknown provider … exit 2" is Task 1's error type (Manifest errors exit 2 via main.rs) — asserted indirectly by the message tests; Task 10's scenarios assert exit codes.
- **Placeholder scan:** Task 10's bodies are commented scenarios with exact strings and outcomes, deliberate as in the previous plan (helpers must be read first); every assertion value is stated. Task 9 lists ripples as a bounded class rather than per-line edits — the deliverable is "suite green with the flip", which is testable. No TBDs.
- **Type consistency:** `entry_for`/`default_entry`/`providers_for`, `enabled(&Manifest) -> Result<Vec<Box<dyn Component>>>`, `MaterialiseSkills { tool, source_dirs, custom }`, `Lock.owners`, `MATTSKILLS_MISE_TOOL`, `MATTSKILLS_SKILLS`, `adopt_existing_mattskills`, `prune_custom`, `switch_lines`, `update(root, target, provider)` are named identically everywhere they appear.

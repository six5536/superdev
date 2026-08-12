---
type: Plan
id: plan-skill-pack
title: Skill Pack Implementation Plan
description: Task-by-task plan for the skill pack — assets, the array-merge action, the hook subcommand, the component, dogfooding and docs.
status: draft
links:
  - rel: implements
    to: spec-skill-pack
    note: Ephemeral; deleted in the commit that completes the work.
---

# Skill Pack Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the [skill pack spec](../specs/2026-08-12-skill-pack-design.md): five skills and the AOKF validation hook shipped as owned repo files by a new `superdev-skills` component, with `[skills] custom` takeover, the `superdev aokf hook validate` plumbing verb, and this repo dogfooding the capability.

**Architecture:** Skill markdown is embedded in `superdev-core` via `include_str!` (the existing `asset!` pattern). A new `components/skillpack.rs` plans `WriteFile` actions for `.claude/skills/<name>/SKILL.md` plus one new action kind, `EnsureJsonArrayElement`, which merges the hook entry into `.claude/settings.json`'s `hooks.PostToolUse` array the way `SetJsonKey` merges `.mcp.json`. The hook itself is a binary subcommand — no shell script.

**Tech Stack:** Existing dependencies only. No new crates.

## Global Constraints

- Branch: `feat/skill-pack`, from `main`.
- Zero new dependencies. serde_json, clap, assert_cmd, tempfile are already in the workspace.
- Prose (skills, docs, messages) follows `.agents/PROSE.md`. Code comments say why, never what.
- Conventional Commits; no Claude signature; sign with `-S`, fall back `--no-gpg-sign` if the agent is unavailable (controller re-signs).
- After any change under `knowledge/`, run `cargo run --quiet -- aokf validate knowledge` and fix errors (a PostToolUse hook also enforces this).
- Never edit `verified` frontmatter or an existing `id` in knowledge files.
- Full check before each commit: `cargo nextest run --workspace` (or the targeted package named in the task) plus `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check`.
- Coverage gates: ≥90% lines in `crates/lib` and `crates/app` (CI enforces; keep new code tested).
- The five skill names, everywhere, in this order: `aokf-maintain`, `double-check`, `grill-me`, `humanise`, `self-improve`.

---

### Task 1: Skill assets

Create the five embedded skill sources under `crates/lib/superdev-core/assets/skills/`. Three are copies of this repo's existing skills, two are adaptations. Every one ends with the same "Project adaptations" trailer.

**Files:**
- Create: `crates/lib/superdev-core/assets/skills/double-check/SKILL.md`
- Create: `crates/lib/superdev-core/assets/skills/grill-me/SKILL.md`
- Create: `crates/lib/superdev-core/assets/skills/humanise/SKILL.md`
- Create: `crates/lib/superdev-core/assets/skills/aokf-maintain/SKILL.md`
- Create: `crates/lib/superdev-core/assets/skills/self-improve/SKILL.md`

**Interfaces:**
- Produces: the five asset files Task 5 embeds via `asset!("skills/<name>/SKILL.md")`. Exact directory names as in Global Constraints.

- [ ] **Step 1: Copy the verbatim trio and the aokf-maintain base**

```bash
cd /workspaces/superdev
for s in double-check grill-me humanise aokf-maintain; do
  mkdir -p crates/lib/superdev-core/assets/skills/$s
  cp .claude/skills/$s/SKILL.md crates/lib/superdev-core/assets/skills/$s/SKILL.md
done
mkdir -p crates/lib/superdev-core/assets/skills/self-improve
cp submodules/goodbye-tinnitus/.claude/skills/self-improve/SKILL.md \
   crates/lib/superdev-core/assets/skills/self-improve/SKILL.md
```

- [ ] **Step 2: Generalise aokf-maintain**

In `assets/skills/aokf-maintain/SKILL.md`, make exactly these three edits:

Edit A — the Phase 1 validator command. Replace:

````markdown
Run the validator first:

```
cargo run --quiet -- aokf validate knowledge
```
````

with:

````markdown
Run the validator first:

```
superdev aokf validate knowledge
```

(In the superdev source repo itself, which has no installed binary, use
`cargo run --quiet -- aokf validate knowledge`.)
````

Edit B — the AGENTS.md check (Phase 1, scripted-checks item 2). Replace:

```markdown
2. The core-concepts list in `AGENTS.md` references only files that
   exist.
```

with:

```markdown
2. Every file `AGENTS.md` references (its `@`-imports and links)
   exists.
```

Edit C — drop the repo-specific pointer in Phase 2. Replace:

```markdown
The code is canonical (see `knowledge/coding-standards.md`). For each
```

with:

```markdown
The code is canonical. For each
```

- [ ] **Step 3: Adapt self-improve to the knowledgebase**

In `assets/skills/self-improve/SKILL.md`, make exactly these five edits:

Edit A — frontmatter description. Replace `and turn them into concrete CLAUDE.md rules.` with `and turn them into concrete rules in the project knowledgebase.` (keep the rest of the description sentence intact).

Edit B — the intro paragraph. Replace:

```markdown
Read past Claude Code session transcripts, identify recurring mistakes, and
propose concrete rules to write into `CLAUDE.md` so future sessions don't
repeat them. Never apply changes without explicit human approval.
```

with:

```markdown
Read past Claude Code session transcripts, identify recurring mistakes, and
propose concrete rules to record in the project knowledgebase so future
sessions don't repeat them. Never apply changes without explicit human
approval.
```

Edit C — the whole `## Outputs` section (up to but not including the `---` rule). Replace with:

```markdown
## Outputs

- `.claude/eval/findings.md` — recurring failure patterns, with evidence.
- `.claude/eval/proposed-rules.md` — candidate rules, for review.
- Approved rules written into the `knowledge/learned-rules.md` concept.
  Git history on that file is the record of what was applied and when —
  there is no separate learning log.

Create the `.claude/eval/` directory if it doesn't exist. Its files are
working state, not knowledge; do not commit them.
```

Edit D — Stage 4's second paragraph. Replace:

```markdown
Before proposing, check `.claude/eval/learning-log.md` and the existing managed block in
`CLAUDE.md` — don't re-propose an applied rule. If a pattern recurs despite a
rule already existing, flag that rule for revision instead of duplicating it.
```

with:

```markdown
Before proposing, read the existing `knowledge/learned-rules.md` — don't
re-propose an applied rule. If a pattern recurs despite a rule already
existing, flag that rule for revision instead of duplicating it.
```

Also in Stage 4's first paragraph, replace `write one concrete, imperative rule suitable for` + `` `CLAUDE.md` `` with `write one concrete, imperative rule suitable for the learned-rules concept`.

Edit E — the whole `## Stage 6 — Apply and log` section (to end of file). Replace with:

````markdown
## Stage 6 — Apply

Write approved rules into `knowledge/learned-rules.md`, an AOKF concept.
Create it on first use with this frontmatter:

```markdown
---
type: Convention
id: learned-rules
title: Learned Rules
description: Rules distilled from past session failures; maintained by the self-improve skill.
---
```

One bullet per rule, grouped under headings when a theme emerges. Update or
retire a rule rather than duplicating it; if the list outgrows ~200 lines,
merge or retire weaker rules instead of appending forever. List the concept
in `knowledge/index.md`, and validate after editing
(`superdev aokf validate knowledge`; in the superdev source repo,
`cargo run --quiet -- aokf validate knowledge`).

Report to the user: sessions analyzed, signal used, patterns found, rules
applied. Git history on the concept is the learning log.
````

- [ ] **Step 4: Append the trailer to all five**

Append this exact block (with one blank line before it) to the end of every one of the five asset files:

```markdown

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
```

- [ ] **Step 5: Verify and commit**

Check: `grep -L "Project adaptations" crates/lib/superdev-core/assets/skills/*/SKILL.md` prints nothing; `grep -rn "CLAUDE.md\|learning-log" crates/lib/superdev-core/assets/skills/self-improve/SKILL.md` prints nothing except the `.claude/rules/` line (path-scoped rules are still a valid Claude Code feature — leave that sentence).

```bash
git add crates/lib/superdev-core/assets/skills
git commit -S -m "feat(skills): add the five embedded skill sources"
```

---

### Task 2: Registry flip, manifest `custom`, binary-pinned skills

**Files:**
- Modify: `crates/lib/superdev-core/src/registry.rs` (skills entry + test)
- Modify: `crates/lib/superdev-core/src/manifest.rs` (`custom` field + tests)
- Modify: `crates/app/superdev/src/manage.rs` (`--no-skills`, pinned set, tests)

**Interfaces:**
- Produces: `RegistryEntry { capability: Skills, provider: "superdev-skills", version: Some(env!("CARGO_PKG_VERSION")), available: true }`; `CapabilityConfig.custom: Vec<String>`; `InitArgs.no_skills: bool`. Task 5 and 6 rely on all three.

- [ ] **Step 1: Write the failing tests**

In `registry.rs`, replace the `skills_slot_is_unavailable` test with:

```rust
    #[test]
    fn skills_slot_ships_at_the_binary_version() {
        let skills = entries()
            .iter()
            .find(|e| e.capability == Capability::Skills)
            .unwrap();
        assert!(skills.available);
        assert_eq!(skills.provider, "superdev-skills");
        assert_eq!(skills.version, Some(env!("CARGO_PKG_VERSION")));
    }
```

In `manifest.rs`, edit `default_manifest_round_trips`: replace the line `assert!(!m.capabilities.contains_key("skills")); // unavailable slot never defaults on` with:

```rust
        assert_eq!(
            m.capabilities["skills"].version.as_deref(),
            Some(env!("CARGO_PKG_VERSION"))
        );
```

and add a new test:

```rust
    #[test]
    fn custom_skills_survive_a_round_trip_and_stay_optional() {
        let mut m = Manifest::default_for("0.1.0", &[]);
        assert!(!m.to_toml().contains("custom"));
        m.capabilities.get_mut("skills").unwrap().custom = vec!["humanise".into()];
        assert_eq!(Manifest::parse(&m.to_toml()).unwrap(), m);
    }
```

In `manage.rs` tests, extend `parses_update_targets` with:

```rust
        assert!(parse_target("skills@9.9.9").is_err());
```

- [ ] **Step 2: Run to verify failures**

Run: `cargo nextest run -p superdev-core -p superdev registry manifest manage`
Expected: FAIL — no `custom` field, skills unavailable, `skills@9.9.9` accepted.

- [ ] **Step 3: Implement**

`registry.rs` — the skills entry becomes:

```rust
    RegistryEntry {
        capability: Capability::Skills,
        provider: "superdev-skills",
        version: Some(env!("CARGO_PKG_VERSION")),
        available: true,
    },
```

Also update the `Capability::Skills` doc comment in `capability.rs` from `superdev's own skill pack plugin (slot; sub-project 3).` to `superdev's own skill pack, shipped as owned repo files.`

`manifest.rs` — add to `CapabilityConfig`:

```rust
    /// Skills released from management: superdev stops writing them and
    /// `status` reports them as custom. Only meaningful for `skills`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom: Vec<String>,
```

and add `custom: Vec::new(),` to the `CapabilityConfig` literal in `default_for`.

`manage.rs` — rename and extend the pinned set. Replace the `CHECKSUM_PINNED` const and its doc comment with:

```rust
/// Capabilities whose version is this binary's to decide — a checksum baked
/// in beside the version, or content embedded in the binary itself — so
/// superdev can install the registry default and nothing else. Their
/// components refuse to plan any other pin.
const BINARY_PINNED: [Capability; 3] = [
    Capability::Workflows,
    Capability::CodeIndex,
    Capability::Skills,
];
```

Update every `CHECKSUM_PINNED` use site to `BINARY_PINNED` (`parse_target`, `behind_pins`, `checksum_pin_mismatch`, `plannable`, and the tests `any_checksum_pin_off_the_default_is_stale` / `plannable_resets_every_checksum_pin`). In `parse_target`, change the refusal message to end `— this binary is the provenance` (still true for the checksummed pair: the checksum is baked into the binary).

Add the flag to `InitArgs`:

```rust
    /// Skip the superdev skill pack
    #[arg(long)]
    pub no_skills: bool,
```

and `(self.no_skills, Capability::Skills),` to the `flags` array in `disabled()`.

- [ ] **Step 4: Run to verify passes**

Run: `cargo nextest run --workspace`
Expected: PASS. Note: `components::tests::enabled_skips_disabled_capabilities` still passes (the manifest now carries `skills`, but no component claims it until Task 5).

- [ ] **Step 5: Commit**

```bash
git add crates/lib/superdev-core/src crates/app/superdev/src
git commit -S -m "feat(skills): registry entry, [skills] custom, --no-skills, binary-pinned set"
```

---

### Task 3: `EnsureJsonArrayElement` action

The array-element analogue of `SetJsonKey`: superdev owns one element of a JSON array (found by marker), the rest are the user's.

**Files:**
- Modify: `crates/lib/superdev-core/src/action.rs`
- Modify: `crates/lib/superdev-core/src/engine.rs`

**Interfaces:**
- Produces: `Action::EnsureJsonArrayElement { path, pointer, marker, value_json }` (all `String`), described as `` ensure {path} {pointer} has the `{marker}` entry ``; lock key `"{path}:{pointer}[{marker}]"`. Task 5 plans it.

- [ ] **Step 1: Write the failing tests**

In `action.rs` `describe_names_the_target`, append:

```rust
        let a = Action::EnsureJsonArrayElement {
            path: ".claude/settings.json".into(),
            pointer: "hooks.PostToolUse".into(),
            marker: "superdev aokf hook validate".into(),
            value_json: "{}".into(),
        };
        assert_eq!(
            a.describe(),
            "ensure .claude/settings.json hooks.PostToolUse has the `superdev aokf hook validate` entry"
        );
```

In `engine.rs` tests, add a fixture helper next to `set_mcp_key`:

```rust
    /// The hook registration the skills provider plans, used by the array tests.
    fn ensure_hook() -> Action {
        Action::EnsureJsonArrayElement {
            path: ".claude/settings.json".into(),
            pointer: "hooks.PostToolUse".into(),
            marker: "superdev aokf hook validate".into(),
            value_json: r#"{"matcher":"Edit|Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}"#.into(),
        }
    }
```

and these tests (mirror the `set_json_key` test shapes — `Planned { capability: Some(crate::capability::Capability::Skills), provider: "superdev-skills".into(), actions: vec![ensure_hook()] }` throughout):

```rust
    #[test]
    fn ensure_array_element_appends_and_preserves_user_entries() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(
            dir.path().join(".claude/settings.json"),
            "{\n  \"hooks\": { \"PostToolUse\": [ { \"matcher\": \"Agent\", \"hooks\": [] } ] },\n  \"permissions\": { \"deny\": [] }\n}\n",
        )
        .unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Skills),
            provider: "superdev-skills".into(),
            actions: vec![ensure_hook()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        let text = std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap();
        let written: serde_json::Value = serde_json::from_str(&text).unwrap();
        let entries = written["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["matcher"], "Agent");
        assert_eq!(
            entries[1]["hooks"][0]["command"],
            "superdev aokf hook validate"
        );
        assert!(written["permissions"].is_object());
        assert!(
            lock.files.contains_key(
                ".claude/settings.json:hooks.PostToolUse[superdev aokf hook validate]"
            ),
            "lock: {:?}",
            lock.files
        );
    }

    #[test]
    fn ensure_array_element_replaces_a_stale_superdev_entry() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        // A prior release's entry: same marker, different matcher.
        std::fs::write(
            dir.path().join(".claude/settings.json"),
            r#"{"hooks":{"PostToolUse":[{"matcher":"Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}]}}"#,
        )
        .unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Skills),
            provider: "superdev-skills".into(),
            actions: vec![ensure_hook()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        let written: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
        )
        .unwrap();
        let entries = written["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(entries.len(), 1, "replaced, not duplicated");
        assert_eq!(entries[0]["matcher"], "Edit|Write");
    }

    #[test]
    fn ensure_array_element_creates_the_file_and_path_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Skills),
            provider: "superdev-skills".into(),
            actions: vec![ensure_hook()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        let written: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            written["hooks"]["PostToolUse"][0]["hooks"][0]["command"],
            "superdev aokf hook validate"
        );
    }

    #[test]
    fn ensure_array_element_rejects_a_non_array_pointer() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(
            dir.path().join(".claude/settings.json"),
            r#"{"hooks":{"PostToolUse":{"matcher":"Edit"}}}"#,
        )
        .unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Skills),
            provider: "superdev-skills".into(),
            actions: vec![ensure_hook()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        let (_, ActionOutcome::Failed(message)) = &result.reports[0].outcomes[0] else {
            panic!("expected a failure");
        };
        assert_eq!(
            message,
            ".claude/settings.json: `PostToolUse` is not a JSON array"
        );
        // The user's file is left exactly as they wrote it.
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
            r#"{"hooks":{"PostToolUse":{"matcher":"Edit"}}}"#
        );
        assert!(lock.files.is_empty());
    }

    #[test]
    fn ensure_array_element_on_a_malformed_file_fails_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(dir.path().join(".claude/settings.json"), "not json\n").unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Skills),
            provider: "superdev-skills".into(),
            actions: vec![ensure_hook()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
            "not json\n"
        );
    }
```

- [ ] **Step 2: Run to verify failures**

Run: `cargo nextest run -p superdev-core action engine`
Expected: FAIL — the variant does not exist (compile error is the failure here; add the variant stub only if you want the tests to fail at assert level, otherwise proceed).

- [ ] **Step 3: Implement**

`action.rs` — add the variant after `SetJsonKey`:

```rust
    /// Ensure a JSON array carries one superdev-owned element, found by
    /// marker; every other element is the user's. Creates the file and the
    /// path to the array when absent. Used for .claude/settings.json hooks.
    EnsureJsonArrayElement {
        /// Target path (repo-relative).
        path: String,
        /// Dotted key path to the array, e.g. `hooks.PostToolUse`.
        pointer: String,
        /// Substring identifying superdev's element among the array's entries.
        marker: String,
        /// The desired element, as a JSON string.
        value_json: String,
    },
```

and the `describe` arm:

```rust
            Action::EnsureJsonArrayElement {
                path,
                pointer,
                marker,
                ..
            } => format!("ensure {path} {pointer} has the `{marker}` entry"),
```

`engine.rs` — add the dispatch arm in `apply_entry` after the `SetJsonKey` arm:

```rust
                Action::EnsureJsonArrayElement {
                    path,
                    pointer,
                    marker,
                    value_json,
                } => self.ensure_json_array_element(path, pointer, marker, value_json, &mut written),
```

the session method after `set_json_key` (same journal/lock pattern):

```rust
    /// Merge one array element into a JSON file. Superdev owns the element
    /// its marker finds — replaced in place, appended when absent — and the
    /// lock hashes the canonical element, not the file.
    fn ensure_json_array_element(
        &mut self,
        path: &str,
        pointer: &str,
        marker: &str,
        value_json: &str,
        written: &mut Vec<(String, String)>,
    ) -> ActionOutcome {
        let full = self.root.join(path);
        let existing = match read_text(&full) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        // Edit in memory first, so a malformed file is left as the user wrote it.
        let edited = edit_json_array_element(
            path,
            existing.as_deref().unwrap_or("{}"),
            pointer,
            marker,
            value_json,
        );
        let (content, value) = match edited {
            Ok(edited) => edited,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        self.journal.push(Undo::RestoreFile {
            path: path.to_string(),
            prior: existing,
        });
        if let Err(e) = write_file(&full, &content) {
            return ActionOutcome::Failed(e.to_string());
        }
        written.push((
            format!("{path}:{pointer}[{marker}]"),
            sha256_hex(value.as_bytes()),
        ));
        ActionOutcome::Applied { note: None }
    }
```

and the free function after `edit_json_key`:

```rust
/// Ensure the array at a dotted key path contains `value_json`: the first
/// element whose serialised form contains `marker` is replaced, else the
/// element is appended. Missing objects on the way — and the array itself —
/// are created. Returns the file content to write and the canonical element
/// text, which is what the lock hashes.
fn edit_json_array_element(
    path: &str,
    json: &str,
    pointer: &str,
    marker: &str,
    value_json: &str,
) -> Result<(String, String)> {
    let bad = |message: String| Error::Toml {
        path: path.into(),
        message,
    };
    let mut root: serde_json::Value = serde_json::from_str(json).map_err(|e| bad(e.to_string()))?;
    let value: serde_json::Value = serde_json::from_str(value_json)
        .map_err(|e| bad(format!("invalid value `{value_json}`: {e}")))?;

    let mut container = "the root".to_string();
    let mut segment_name = "the root";
    let mut cursor = &mut root;
    for segment in pointer.split('.') {
        cursor = match cursor.as_object_mut() {
            Some(map) => map
                .entry(segment)
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new())),
            None => return Err(bad(format!("{container} is not a JSON object"))),
        };
        container = format!("`{segment}`");
        segment_name = segment;
    }
    // The walk mints an empty object for a missing final segment; the pointer
    // names an array, so turn that placeholder into one.
    if cursor.as_object().is_some_and(serde_json::Map::is_empty) {
        *cursor = serde_json::Value::Array(Vec::new());
    }
    let Some(items) = cursor.as_array_mut() else {
        return Err(bad(format!("`{segment_name}` is not a JSON array")));
    };
    match items
        .iter_mut()
        .find(|item| item.to_string().contains(marker))
    {
        Some(item) => *item = value.clone(),
        None => items.push(value.clone()),
    }

    let mut content = serde_json::to_string_pretty(&root).expect("a parsed value re-serialises");
    content.push('\n');
    Ok((content, value.to_string()))
}
```

- [ ] **Step 4: Run to verify passes**

Run: `cargo nextest run -p superdev-core`
Expected: PASS, including the five new engine tests.

- [ ] **Step 5: Commit**

```bash
git add crates/lib/superdev-core/src/action.rs crates/lib/superdev-core/src/engine.rs
git commit -S -m "feat(engine): EnsureJsonArrayElement — superdev-owned entry in a shared JSON array"
```

---

### Task 4: `superdev aokf hook validate`

The hook as a plumbing subcommand: payload on stdin, gate on the bundle path, validate in-process, exit 2 to block.

**Files:**
- Modify: `crates/app/superdev/src/aokf_cli.rs`
- Test: `crates/app/superdev/tests/cli.rs`

**Interfaces:**
- Consumes: `load_bundle`, `validate`, `DEFAULT_LEVEL`, `BUNDLE_DIR` (all already in `aokf_cli.rs`).
- Produces: the CLI verb `superdev aokf hook validate` — literal command string Task 5 embeds in `HOOK_ELEMENT`.

- [ ] **Step 1: Write the failing integration tests**

Append to `crates/app/superdev/tests/cli.rs`:

```rust
/// A repo with a `knowledge/` bundle: valid (level-2 clean) or broken.
fn hook_repo(valid: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let k = dir.path().join("knowledge");
    std::fs::create_dir_all(&k).unwrap();
    std::fs::write(
        k.join("manifest.aokf.yaml"),
        "aokf: \"0.1\"\nname: fixture\n",
    )
    .unwrap();
    let concept = if valid {
        "---\ntype: Note\nid: alpha\n---\n\nBody.\n"
    } else {
        "---\nid: alpha\n---\n\nMissing type.\n"
    };
    std::fs::write(k.join("alpha.md"), concept).unwrap();
    dir
}

fn hook_payload(dir: &Path, rel: &str) -> String {
    serde_json::json!({
        "tool_input": { "file_path": dir.join(rel) }
    })
    .to_string()
}

#[test]
fn hook_validate_blocks_an_edit_that_broke_the_bundle() {
    let repo = hook_repo(false);
    let out = superdev()
        .args(["aokf", "hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "knowledge/alpha.md"))
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        stderr.contains("AOKF validation failed after editing"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("alpha.md"), "stderr: {stderr}");
}

#[test]
fn hook_validate_passes_a_clean_bundle() {
    let repo = hook_repo(true);
    superdev()
        .args(["aokf", "hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "knowledge/alpha.md"))
        .assert()
        .code(0);
}

#[test]
fn hook_validate_ignores_paths_outside_the_bundle() {
    // Even a broken bundle: an edit elsewhere is not the hook's business.
    let repo = hook_repo(false);
    superdev()
        .args(["aokf", "hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "src/main.rs"))
        .assert()
        .code(0);
}

#[test]
fn hook_validate_falls_back_to_the_working_directory() {
    let repo = hook_repo(false);
    superdev()
        .current_dir(repo.path())
        .args(["aokf", "hook", "validate"])
        .env_remove("CLAUDE_PROJECT_DIR")
        .write_stdin(hook_payload(repo.path(), "knowledge/alpha.md"))
        .assert()
        .code(2);
}

#[test]
fn hook_validate_is_loud_on_a_malformed_payload() {
    let repo = hook_repo(true);
    let out = superdev()
        .args(["aokf", "hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin("not json")
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("malformed"), "stderr: {stderr}");
}

#[test]
fn hook_validate_ignores_payloads_without_a_file_path() {
    let repo = hook_repo(false);
    superdev()
        .args(["aokf", "hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(r#"{"tool_input":{}}"#)
        .assert()
        .code(0);
}
```

`serde_json` is already a dependency of the `superdev` crate, so the tests can use it directly. `use std::path::Path;` is already imported in `cli.rs`.

- [ ] **Step 2: Run to verify failures**

Run: `cargo nextest run -p superdev hook_validate`
Expected: FAIL — `hook` is not a known `aokf` subcommand (clap usage error, exit 2 with the wrong stderr — the assertions on stderr text fail).

- [ ] **Step 3: Implement**

In `aokf_cli.rs`, add `Read` to the `io` import (`use std::io::{self, Read, Write};`), add the subcommands:

```rust
    /// Claude Code hook plumbing (reads the hook payload from stdin)
    #[command(subcommand)]
    Hook(HookCommand),
```

(to `AokfCommand`, after `Index`), and:

```rust
/// One verb per hook, so future hooks slot in beside `validate`.
#[derive(clap::Subcommand)]
pub enum HookCommand {
    /// PostToolUse: validate the bundle after an Edit/Write under knowledge/
    Validate,
}
```

Add the arm to `run_aokf`:

```rust
        AokfCommand::Hook(HookCommand::Validate) => hook_validate(root),
```

and the function:

```rust
/// The PostToolUse hook body. Exit 0 unless the payload names a path under
/// the bundle; then validate and exit 2 with findings on errors, which
/// Claude Code feeds back to the agent as a blocking error. An unreadable
/// payload is a loud exit 2 — a silent skip here silently stops validating
/// the bundle.
fn hook_validate(root: &Path) -> Result<u8> {
    // Hooks run with the project as the working directory, but Claude Code
    // also names it explicitly; prefer the explicit form.
    let root = std::env::var_os("CLAUDE_PROJECT_DIR")
        .map_or_else(|| root.to_path_buf(), PathBuf::from);
    let mut payload = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut payload) {
        eprintln!("aokf hook: could not read the tool payload from stdin: {e}");
        return Ok(2);
    }
    let parsed: serde_json::Value = match serde_json::from_str(&payload) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("aokf hook: malformed tool payload on stdin: {e}");
            return Ok(2);
        }
    };
    let Some(file_path) = parsed["tool_input"]["file_path"].as_str() else {
        // Not a file edit: nothing to validate.
        return Ok(0);
    };
    let bundle = root.join(BUNDLE_DIR);
    let edited = Path::new(file_path);
    if !edited.starts_with(&bundle) && !edited.starts_with(BUNDLE_DIR) {
        return Ok(0);
    }
    let report = validate(&load_bundle(&bundle)?, &root, DEFAULT_LEVEL);
    if report.passed() {
        return Ok(0);
    }
    eprintln!("AOKF validation failed after editing {file_path} — fix before continuing:");
    eprintln!("{}", report.render_human().trim_end_matches('\n'));
    Ok(2)
}
```

(`load_bundle` failure propagates as `Err`, which `main` prints and turns into exit 2 — loud, blocking, correct.)

- [ ] **Step 4: Run to verify passes**

Run: `cargo nextest run -p superdev`
Expected: PASS, all six new tests included.

- [ ] **Step 5: Commit**

```bash
git add crates/app/superdev/src/aokf_cli.rs crates/app/superdev/tests/cli.rs
git commit -S -m "feat(aokf): hook validate — the PostToolUse hook as a subcommand"
```

---

### Task 5: The skillpack component

**Files:**
- Create: `crates/lib/superdev-core/src/components/skillpack.rs`
- Modify: `crates/lib/superdev-core/src/components/mod.rs`

**Interfaces:**
- Consumes: Task 1's assets; Task 2's registry entry and `custom` field; Task 3's action.
- Produces: `skillpack::SkillPack` (unit struct implementing `Component`), `skillpack::SKILLS: [(&str, &str); 5]`, wired into `components::enabled`.

- [ ] **Step 1: Write the component with its tests**

Create `components/skillpack.rs`:

```rust
//! components/skillpack.rs — the skills capability: superdev's own pack,
//! shipped as owned files in the managed repo. Claude Code loads project
//! skills from `.claude/skills/` natively, so there is nothing to install.

use crate::action::{Action, Ownership};
use crate::capability::Capability;
use crate::component::{Component, Ctx};
use crate::error::{Error, Result};
use crate::registry;

macro_rules! asset {
    ($rel:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $rel))
    };
}

/// The pack: (skill name, embedded SKILL.md).
pub const SKILLS: [(&str, &str); 5] = [
    ("aokf-maintain", asset!("skills/aokf-maintain/SKILL.md")),
    ("double-check", asset!("skills/double-check/SKILL.md")),
    ("grill-me", asset!("skills/grill-me/SKILL.md")),
    ("humanise", asset!("skills/humanise/SKILL.md")),
    ("self-improve", asset!("skills/self-improve/SKILL.md")),
];

/// Where Claude Code reads hook registrations. Shared with the user's own
/// hooks, so only superdev's array element is managed.
pub const SETTINGS_PATH: &str = ".claude/settings.json";
/// The array the hook entry lives in.
pub const HOOK_POINTER: &str = "hooks.PostToolUse";
/// What identifies superdev's element among the user's.
pub const HOOK_MARKER: &str = "superdev aokf hook validate";
/// The registration itself: validate the bundle after an Edit/Write.
pub const HOOK_ELEMENT: &str = r#"{"matcher":"Edit|Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}"#;

/// The superdev skill pack provider.
pub struct SkillPack;

impl SkillPack {
    /// The hook action, unless the settings file already carries the exact
    /// desired element. Planning must stay empty when converged: `status`
    /// exits 1 on any planned action.
    fn hook_action(&self, ctx: &Ctx<'_>) -> Option<Action> {
        let desired: serde_json::Value =
            serde_json::from_str(HOOK_ELEMENT).expect("the hook element is valid JSON");
        let present = std::fs::read_to_string(ctx.root.join(SETTINGS_PATH))
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v["hooks"]["PostToolUse"].as_array().cloned())
            .is_some_and(|items| items.contains(&desired));
        (!present).then(|| Action::EnsureJsonArrayElement {
            path: SETTINGS_PATH.into(),
            pointer: HOOK_POINTER.into(),
            marker: HOOK_MARKER.into(),
            value_json: HOOK_ELEMENT.into(),
        })
    }
}

impl Component for SkillPack {
    fn capability(&self) -> Capability {
        Capability::Skills
    }

    fn provider(&self) -> &'static str {
        "superdev-skills"
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        let config = ctx
            .config(Capability::Skills)
            .expect("planned only when enabled");
        let default = registry::entries()
            .iter()
            .find(|e| e.capability == Capability::Skills)
            .and_then(|e| e.version)
            .expect("registry pins the skill pack");
        if config.version.as_deref() != Some(default) {
            return Err(Error::Manifest {
                message: format!(
                    "skills version must match this binary ({default}) — the embedded content is the provenance"
                ),
            });
        }
        for name in &config.custom {
            if !SKILLS.iter().any(|(known, _)| known == name) {
                return Err(Error::Manifest {
                    message: format!("[skills] custom names unknown skill `{name}`"),
                });
            }
        }
        let mut actions = Vec::new();
        for (name, content) in SKILLS {
            if config.custom.iter().any(|c| c == name) {
                continue;
            }
            let path = format!(".claude/skills/{name}/SKILL.md");
            let existing = std::fs::read_to_string(ctx.root.join(&path)).ok();
            if existing.as_deref() != Some(content) {
                actions.push(Action::WriteFile {
                    path,
                    content: content.to_string(),
                    ownership: Ownership::Owned,
                    reason: format!("{name} skill"),
                });
            }
        }
        actions.extend(self.hook_action(ctx));
        Ok(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::runner::FakeRunner;

    fn ctx_parts() -> (Manifest, Lock) {
        (
            Manifest::default_for(env!("CARGO_PKG_VERSION"), &[]),
            Lock::default(),
        )
    }

    /// Write every skill and the exact hook entry, so nothing is planned.
    fn converge(root: &std::path::Path) {
        for (name, content) in SKILLS {
            let path = root.join(format!(".claude/skills/{name}/SKILL.md"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        std::fs::write(
            root.join(SETTINGS_PATH),
            format!(r#"{{"hooks":{{"PostToolUse":[{HOOK_ELEMENT}]}}}}"#),
        )
        .unwrap();
    }

    #[test]
    fn a_fresh_repo_plans_every_skill_and_the_hook() {
        let dir = tempfile::tempdir().unwrap();
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let actions = SkillPack.plan(&ctx).unwrap();
        assert_eq!(actions.len(), 6);
        let descs: Vec<String> = actions.iter().map(|a| a.describe()).collect();
        for (name, _) in SKILLS {
            assert!(
                descs
                    .iter()
                    .any(|d| d.contains(&format!(".claude/skills/{name}/SKILL.md"))),
                "{descs:?}"
            );
        }
        assert!(
            descs
                .iter()
                .any(|d| d.contains("superdev aokf hook validate")),
            "{descs:?}"
        );
        assert!(fake.calls().is_empty(), "planning must run nothing");
    }

    #[test]
    fn a_converged_repo_plans_nothing() {
        let dir = tempfile::tempdir().unwrap();
        converge(dir.path());
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(SkillPack.plan(&ctx).unwrap().is_empty());
    }

    #[test]
    fn a_drifted_skill_is_rewritten_alone() {
        let dir = tempfile::tempdir().unwrap();
        converge(dir.path());
        std::fs::write(
            dir.path().join(".claude/skills/humanise/SKILL.md"),
            "edited",
        )
        .unwrap();
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let actions = SkillPack.plan(&ctx).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(actions[0].describe().contains("humanise"));
    }

    #[test]
    fn a_custom_skill_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        converge(dir.path());
        std::fs::write(
            dir.path().join(".claude/skills/humanise/SKILL.md"),
            "mine now",
        )
        .unwrap();
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("skills").unwrap().custom = vec!["humanise".into()];
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(SkillPack.plan(&ctx).unwrap().is_empty());
    }

    #[test]
    fn an_unknown_custom_name_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("skills").unwrap().custom = vec!["humanize".into()];
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let err = SkillPack.plan(&ctx).unwrap_err();
        assert!(err.to_string().contains("humanize"), "{err}");
    }

    #[test]
    fn a_foreign_version_pin_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("skills").unwrap().version = Some("9.9.9".into());
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(SkillPack.plan(&ctx).is_err());
    }

    #[test]
    fn a_stale_hook_entry_replans_the_hook() {
        let dir = tempfile::tempdir().unwrap();
        converge(dir.path());
        // Same marker, older shape: must be replaced, so it must be planned.
        std::fs::write(
            dir.path().join(SETTINGS_PATH),
            r#"{"hooks":{"PostToolUse":[{"matcher":"Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}]}}"#,
        )
        .unwrap();
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let actions = SkillPack.plan(&ctx).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(actions[0].describe().contains("hooks.PostToolUse"));
    }

    #[test]
    fn reports_its_slot_and_provider() {
        assert_eq!(SkillPack.capability(), Capability::Skills);
        assert_eq!(SkillPack.provider(), "superdev-skills");
    }
}
```

- [ ] **Step 2: Wire into the module and fix the count test**

`components/mod.rs`: add `pub mod skillpack;` (alphabetical, after `plugin`), and in `enabled` insert `Box::new(skillpack::SkillPack),` after the `frontend_design()` entry (apply order: plugins, then skills, then code index, then knowledge — matching `Capability::ALL`). Update `enabled_skips_disabled_capabilities`: the expected length becomes `4`.

- [ ] **Step 3: Run to verify passes**

Run: `cargo nextest run -p superdev-core`
Expected: PASS. Then `cargo nextest run --workspace` — expected PASS (`plan_runs_every_component` in `engine.rs` keeps passing: it asserts on the first provider and non-empty plans, both still true).

- [ ] **Step 4: Commit**

```bash
git add crates/lib/superdev-core/src/components
git commit -S -m "feat(skills): the superdev-skills component — owned skill files plus the hook entry"
```

---

### Task 6: Custom-skill lock pruning and status lines

**Files:**
- Modify: `crates/app/superdev/src/manage.rs`

**Interfaces:**
- Consumes: `CapabilityConfig.custom` (Task 2).
- Produces: `status` prints `skills: <name> custom, unmanaged` per released skill; `sync` removes released skills' hashes from the lock.

- [ ] **Step 1: Write the failing unit tests**

Add to `manage.rs` tests:

```rust
    #[test]
    fn custom_skills_are_pruned_from_the_lock_and_reported() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("skills").unwrap().custom =
            vec!["humanise".into(), "grill-me".into()];
        let mut lock = Lock::default();
        lock.files.insert(
            ".claude/skills/humanise/SKILL.md".into(),
            "hash-a".into(),
        );
        lock.files.insert(
            ".claude/skills/double-check/SKILL.md".into(),
            "hash-b".into(),
        );
        assert!(prune_custom_skills(&manifest, &mut lock));
        assert!(!lock.files.contains_key(".claude/skills/humanise/SKILL.md"));
        assert!(lock.files.contains_key(".claude/skills/double-check/SKILL.md"));
        // Nothing left to prune: reports no change.
        assert!(!prune_custom_skills(&manifest, &mut lock));

        assert_eq!(
            custom_lines(&manifest),
            vec![
                "skills: humanise custom, unmanaged".to_string(),
                "skills: grill-me custom, unmanaged".to_string(),
            ]
        );
        let no_skills = Manifest::default_for("0.1.0", &[Capability::Skills]);
        assert!(custom_lines(&no_skills).is_empty());
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo nextest run -p superdev custom_skills`
Expected: FAIL — the two functions do not exist.

- [ ] **Step 3: Implement**

Add to `manage.rs` (after `behind_pins`):

```rust
/// Remove released skills' hashes from the lock: a custom skill is the
/// user's file, and a stale hash would misread their next edit as drift
/// against superdev content. True when anything was removed.
fn prune_custom_skills(manifest: &Manifest, lock: &mut Lock) -> bool {
    let Some(config) = manifest.capabilities.get(Capability::Skills.as_str()) else {
        return false;
    };
    let mut pruned = false;
    for name in &config.custom {
        pruned |= lock
            .files
            .remove(&format!(".claude/skills/{name}/SKILL.md"))
            .is_some();
    }
    pruned
}

/// One line per skill released to the user, so custom state stays visible
/// without reading the manifest.
fn custom_lines(manifest: &Manifest) -> Vec<String> {
    manifest
        .capabilities
        .get(Capability::Skills.as_str())
        .map(|config| {
            config
                .custom
                .iter()
                .map(|name| format!("skills: {name} custom, unmanaged"))
                .collect()
        })
        .unwrap_or_default()
}
```

In `status`, after the `behind` loop, add:

```rust
    for line in &custom_lines(&manifest) {
        out(line)?;
    }
```

(custom lines never affect the exit code — a takeover is a chosen state, not work to do).

In `sync`, after `let mut lock = Lock::load(root)?;`, add the prune, and save on the no-actions path so a takeover alone still cleans the lock:

```rust
    let pruned = prune_custom_skills(&manifest, &mut lock);
```

and replace the early return:

```rust
    if dry_run || !has_actions(&planned) {
        return Ok(0);
    }
```

with:

```rust
    if dry_run {
        return Ok(0);
    }
    if !has_actions(&planned) {
        if pruned {
            lock.save(root)?;
        }
        return Ok(0);
    }
```

- [ ] **Step 4: Run to verify passes**

Run: `cargo nextest run -p superdev`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/app/superdev/src/manage.rs
git commit -S -m "feat(skills): prune released skills from the lock, report them in status"
```

---

### Task 7: End-to-end integration tests

Cross-platform tests in `cli.rs` (the skills capability needs no external binaries; disable the three that do).

**Files:**
- Test: `crates/app/superdev/tests/cli.rs`

**Interfaces:**
- Consumes: everything above, through the real binary.

- [ ] **Step 1: Write the tests**

Append to `cli.rs`:

```rust
/// `init` a temp repo with only the skills capability (the others need
/// external binaries; skills needs none, so these tests run everywhere).
fn skills_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    superdev()
        .current_dir(dir.path())
        .args([
            "init",
            "--no-workflows",
            "--no-frontend",
            "--no-code-index",
            "--no-knowledge",
        ])
        .assert()
        .success();
    dir
}

#[test]
fn init_materialises_the_skill_pack_and_hook() {
    let dir = skills_repo();
    for name in [
        "aokf-maintain",
        "double-check",
        "grill-me",
        "humanise",
        "self-improve",
    ] {
        let path = dir.path().join(format!(".claude/skills/{name}/SKILL.md"));
        assert!(path.is_file(), "missing {}", path.display());
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .contains("Project adaptations"),
            "{name} lacks the PROJECT.md trailer"
        );
    }
    let settings: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    let entries = settings["hooks"]["PostToolUse"].as_array().unwrap();
    assert!(
        entries
            .iter()
            .any(|e| e.to_string().contains("superdev aokf hook validate")),
        "settings: {settings}"
    );
    let lock = std::fs::read_to_string(dir.path().join(".superdev/lock.toml")).unwrap();
    assert!(lock.contains(".claude/skills/humanise/SKILL.md"), "{lock}");
    assert!(lock.contains("superdev-skills"), "{lock}");
    // Straight after init there is nothing to do.
    superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(0);
}

#[test]
fn init_no_skills_skips_the_pack() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    superdev()
        .current_dir(dir.path())
        .args([
            "init",
            "--no-workflows",
            "--no-frontend",
            "--no-code-index",
            "--no-knowledge",
            "--no-skills",
        ])
        .assert()
        .success();
    assert!(!dir.path().join(".claude/skills").exists());
    assert!(!dir.path().join(".claude/settings.json").exists());
}

#[test]
fn a_drifted_skill_is_drift_until_marked_custom() {
    let dir = skills_repo();
    let skill = dir.path().join(".claude/skills/humanise/SKILL.md");
    std::fs::write(&skill, "# Mine now\n").unwrap();
    // Drift: status exits 1 and names the file.
    let out = superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(1);
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("humanise"), "{stdout}");

    // Take it over: drift becomes a chosen state.
    let config_path = dir.path().join(".superdev/config.toml");
    let config = std::fs::read_to_string(&config_path).unwrap();
    std::fs::write(
        &config_path,
        config.replace("[skills]", "[skills]\ncustom = [\"humanise\"]"),
    )
    .unwrap();
    let out = superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(0);
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains("skills: humanise custom, unmanaged"),
        "{stdout}"
    );

    // sync honours the takeover and prunes the lock entry.
    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .code(0);
    assert_eq!(std::fs::read_to_string(&skill).unwrap(), "# Mine now\n");
    let lock = std::fs::read_to_string(dir.path().join(".superdev/lock.toml")).unwrap();
    assert!(!lock.contains(".claude/skills/humanise/SKILL.md"), "{lock}");

    // Back under management: the next sync restores stock.
    let config = std::fs::read_to_string(&config_path).unwrap();
    std::fs::write(
        &config_path,
        config.replace("\ncustom = [\"humanise\"]", ""),
    )
    .unwrap();
    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .code(0);
    assert!(
        std::fs::read_to_string(&skill)
            .unwrap()
            .contains("Project adaptations")
    );
}

#[test]
fn user_hook_entries_survive_a_sync() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
    std::fs::write(
        dir.path().join(".claude/settings.json"),
        r#"{"hooks":{"PostToolUse":[{"matcher":"Agent","hooks":[{"type":"command","command":"my-own-hook"}]}]},"permissions":{"deny":["Read(secrets/**)"]}}"#,
    )
    .unwrap();
    superdev()
        .current_dir(dir.path())
        .args([
            "init",
            "--no-workflows",
            "--no-frontend",
            "--no-code-index",
            "--no-knowledge",
        ])
        .assert()
        .success();
    let settings: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    let entries = settings["hooks"]["PostToolUse"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|e| e.to_string().contains("my-own-hook")));
    assert_eq!(settings["permissions"]["deny"][0], "Read(secrets/**)");
}

#[test]
fn update_skills_to_an_explicit_version_is_refused() {
    let dir = skills_repo();
    let out = superdev()
        .current_dir(dir.path())
        .args(["update", "skills@9.9.9"])
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("skills"), "{stderr}");
}
```

- [ ] **Step 2: Run**

Run: `cargo nextest run -p superdev`
Expected: PASS. If `init_no_skills_skips_the_pack` finds a `.claude/settings.json`, the component planned the hook while disabled — fix the component, not the test.

- [ ] **Step 3: Full workspace check and commit**

Run: `cargo nextest run --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all -- --check`

```bash
git add crates/app/superdev/tests/cli.rs
git commit -S -m "test(skills): end-to-end init/status/sync/update coverage"
```

---

### Task 8: Dogfood — this repo managed for skills

**Files:**
- Create: `scripts/superdev` (dev shim)
- Create: `.superdev/config.toml`, `.superdev/lock.toml` (written by the binary, committed)
- Modify: `.claude/settings.json` (engine merge + remove the old bash hook entry)
- Delete: `.claude/skills/{aokf-maintain,double-check,grill-me,humanise}/` (rewritten by the engine), `.agents/aokf/tools/validate-hook.sh`
- Modify: `package.json`, `.github/workflows/checks.yml`, `CONTRIBUTING.md`

**Interfaces:**
- Consumes: the whole feature, via `cargo run`.

- [ ] **Step 1: The shim, first — the new hook command needs it on PATH**

Create `scripts/superdev`:

```bash
#!/usr/bin/env bash
# Dev shim: run the working-tree superdev. The managed hook entry says
# `superdev aokf hook validate`, and this repo has no installed binary —
# symlink this file into a PATH directory (see CONTRIBUTING).
set -euo pipefail
exec cargo run --quiet \
  --manifest-path "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/Cargo.toml" -- "$@"
```

```bash
chmod +x scripts/superdev
mkdir -p ~/.local/bin
ln -sf /workspaces/superdev/scripts/superdev ~/.local/bin/superdev
command -v superdev && superdev --version   # must print the workspace version
```

If `command -v superdev` finds nothing, `~/.local/bin` is not on PATH in this container — stop and surface that to the controller instead of improvising.

Add to `CONTRIBUTING.md`'s setup section (adapt to its shape):

```markdown
Link the dev shim so the Claude Code validation hook can find a `superdev`:

    ln -sf "$PWD/scripts/superdev" ~/.local/bin/superdev
```

- [ ] **Step 2: Hand the skills over to the engine**

```bash
git rm -r .claude/skills/aokf-maintain .claude/skills/double-check \
          .claude/skills/grill-me .claude/skills/humanise
cargo run --quiet -- init --no-workflows --no-frontend --no-code-index --no-knowledge
```

Expected: init reports writing the five `.claude/skills/*/SKILL.md` files and the `hooks.PostToolUse` entry; `.superdev/config.toml` contains only `blueprint` and `[skills]`; `.superdev/lock.toml` has the five file hashes plus the settings pointer hash.

- [ ] **Step 3: Retire the bash hook**

The engine appended its entry; the old bash entry (marker `validate-hook.sh`) is still there and would double-fire. Edit `.claude/settings.json`: delete the whole `PostToolUse` element whose command contains `validate-hook.sh`. Leave every `entire` hook and the `permissions` block untouched. Note: the engine re-serialised the file (keys sorted, 2-space indent) — review the diff and confirm the only *semantic* changes are the removed bash entry and the added superdev entry.

```bash
git rm .agents/aokf/tools/validate-hook.sh
rmdir .agents/aokf/tools 2>/dev/null || true
grep -rn "validate-hook" . --include="*.json" --include="*.md" --include="*.sh" \
  --exclude-dir=target --exclude-dir=node_modules --exclude-dir=.git
```

The grep must come back empty (fix any straggler it finds).

- [ ] **Step 4: Prove the hook and the drift gate**

```bash
printf '{"tool_input":{"file_path":"%s"}}' "$PWD/knowledge/index.md" \
  | superdev aokf hook validate; echo "exit: $?"          # exit: 0
printf '{"tool_input":{"file_path":"%s"}}' "$PWD/src/x.rs" \
  | superdev aokf hook validate; echo "exit: $?"          # exit: 0
cargo run --quiet -- status; echo "exit: $?"              # nothing to do, exit: 0
```

- [ ] **Step 5: Pin line endings for the byte-for-byte comparison**

The drift gate compares repo files against binary-embedded content byte for
byte, and a Windows checkout rewrites line endings unless told otherwise
(the fixtures rule in `.gitattributes` exists for the same reason). Append
to `.gitattributes`:

```
# superdev-owned files are compared byte-for-byte against content embedded
# in the binary, so their line endings must survive a Windows checkout.
.claude/skills/** -text
crates/lib/superdev-core/assets/** -text
```

- [ ] **Step 6: Wire the drift gate into checks**

`package.json`, after `"check:aokf"`:

```json
        "check:blueprint": "cargo run --quiet -- status",
```

`.github/workflows/checks.yml`, in the test-matrix job after the "Knowledgebase validation" step:

```yaml
      - name: Blueprint drift
        run: cargo run --quiet -- status
```

- [ ] **Step 7: Commit**

```bash
git add .superdev/config.toml .superdev/lock.toml .claude/skills .claude/settings.json \
        scripts/superdev .gitattributes package.json .github/workflows/checks.yml CONTRIBUTING.md
git commit -S -m "feat: dogfood the skill pack — this repo is skills-managed"
```

(The `.claude/skills` deletions and `validate-hook.sh` removal are already staged by `git rm`.)

---

### Task 9: Knowledge, docs, changelog

**Files:**
- Modify: `CHANGELOG.md`, `crates/lib/superdev-core/assets/agents/VALIDATION.md`
- Modify: `knowledge/architecture.md`, `knowledge/api-contracts.md`, `knowledge/configuration.md`, `knowledge/development-procedure.md`, `knowledge/development-commands.md`, `knowledge/error-handling.md`, `knowledge/glossary.md`
- Modify: `knowledge/specs/2026-08-12-skill-pack-design.md` (status → stable)
- Delete: `knowledge/plans/2026-08-12-skill-pack.md` (this file)

**Interfaces:**
- Consumes: the shipped behaviour. Read each concept with `aokf_read` (or the file) before editing; the code is canonical.

- [ ] **Step 1: The shipped VALIDATION.md**

Replace the full content of `crates/lib/superdev-core/assets/agents/VALIDATION.md` with:

````markdown
# Validation

After any change under `knowledge/`, run the AOKF validator and fix every
error before moving on:

```
superdev aokf validate knowledge
```

It checks the bundle against `.agents/aokf/SPEC.md` (document check plus the
conformance ladder) and must PASS at level 2. Warnings don't fail the run but
usually mean a rename the bundle missed; fix the reference, not the target.

With the skills capability enabled, a PostToolUse hook in
`.claude/settings.json` runs this automatically after every Edit/Write under
`knowledge/` in Claude Code and blocks on errors. The hook does not cover
scripted or manual edits — run the command yourself after those.
````

- [ ] **Step 2: CHANGELOG.md**

Add to `## [Unreleased]` → `### Added`:

```markdown
- The `skills` capability: five skills (aokf-maintain, double-check,
  grill-me, humanise, self-improve) written into `.claude/skills/` as
  superdev-owned files, plus a PostToolUse hook in `.claude/settings.json`
  that runs `superdev aokf hook validate` and blocks edits that break the
  bundle. Claude Code loads both natively — nothing to install
- Per-skill customisation: a `PROJECT.md` beside a skill extends it and is
  never touched; `custom = ["<name>"]` under `[skills]` releases a skill
  from management entirely. The pack's version is the binary's, so
  `update skills@<version>` is refused like the other pinned capabilities
- `superdev aokf hook validate`: the hook as a subcommand — payload on
  stdin, validates in-process, works on every platform superdev ships for
```

- [ ] **Step 3: The knowledge bundle**

Update each concept to match shipped behaviour (search first; wording per PROSE.md; run the validator after each file):

- `architecture.md`: the capability map's skills row — provider `superdev-skills`, delivery "owned files in the repo" (contrast with the plugin-installed capabilities); the settings.json array-element merge alongside the .mcp.json key merge.
- `api-contracts.md`: `aokf hook validate` (stdin payload, exit 0/2); `init --no-skills`; the update-refusal list now names workflows, code-index **and skills**; `[skills] custom` in the manifest surface.
- `configuration.md`: the `custom` key under `[skills]`; the managed `hooks.PostToolUse` entry in `.claude/settings.json`; `PROJECT.md` files are the user's, never written or tracked.
- `development-procedure.md`: this repo is now skills-managed (`.superdev/` committed); the `scripts/superdev` shim; `npm run check:blueprint` joins the pre-PR list; update the sentence "A managed repo names its installed `superdev` instead, and gets no hook at all" — a managed repo now *does* get the hook via its settings entry.
- `development-commands.md`: add `check:blueprint`.
- `error-handling.md`: hook exit semantics — 0 pass/out-of-scope, 2 blocking with findings on stderr; a missing binary surfaces as the hook command failing rather than exit 2.
- `glossary.md`: entries for *skill pack*, *custom skill*, and the *PROJECT.md layer*.

- [ ] **Step 4: Close the spec-and-plan loop**

- `knowledge/specs/2026-08-12-skill-pack-design.md`: `status: draft` → `status: stable`.
- `git rm knowledge/plans/2026-08-12-skill-pack.md` (plans are ephemeral; the spec and git history are the record).

- [ ] **Step 5: Validate, full check, commit**

```bash
cargo run --quiet -- aokf validate knowledge
cargo nextest run --workspace
npm run check:blueprint
git add -A CHANGELOG.md CONTRIBUTING.md crates/lib/superdev-core/assets/agents/VALIDATION.md knowledge
git commit -S -m "docs: skill pack knowledge, changelog and shipped validation notes"
```

Note: changing `assets/agents/VALIDATION.md` changes an Owned file of the *knowledge* capability — irrelevant to this repo's manifest (knowledge capability off here), but `cargo nextest` must stay green.

---

## Self-review notes

- Spec coverage: contents/adaptations → Task 1; owned-file distribution + registry → Tasks 2, 5; customisation (`PROJECT.md` trailer, `custom`) → Tasks 1, 2, 5, 6, 7; hook subcommand → Task 4; settings merge → Tasks 3, 5; version/update semantics → Task 2 (`BINARY_PINNED`); dogfood → Task 8; testing → Tasks 3–7; docs → Task 9.
- Deliberate deviation from none: the spec's "leaves the plan and the lock" for custom skills is implemented as plan-skip (component) + lock prune on sync (Task 6).
- The engine re-serialises `.claude/settings.json` (sorted keys) exactly as it does `.mcp.json` — the known, ledgered wart, now noted for settings in Task 9's configuration.md update.

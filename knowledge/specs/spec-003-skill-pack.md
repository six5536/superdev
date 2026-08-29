---
type: Spec
id: spec-003-skill-pack
title: Skill Pack
description: Design for the skills capability — five skills and the validation hook shipped as owned repo files, with a PROJECT.md extension layer and a per-skill custom opt-out.
status: stable
links:
  - rel: relates-to
    to: spec-001-cli-core-blueprint-engine
    note: Sub-project 3; fills the blueprint's skills slot using the owned-file machinery.
  - rel: relates-to
    to: spec-002-aokf-mcp-server
    note: The shipped hook and aokf-maintain skill wrap the validator sub-project 2 built.
---

# Summary

Fill the `skills` capability slot the
[blueprint engine][sokf:spec-001-cli-core-blueprint-engine] left as a
placeholder: ship a curated skill set and the AOKF validation hook into every
managed repo, versioned with the binary, with a clean path for per-project
customisation. The hook and the aokf-maintain skill wrap the validator built
for the [AOKF MCP server][sokf:spec-002-aokf-mcp-server].

# Contents

Five skills, absorbed from goodbye-tinnitus and this repo:

| Skill | Source | Adaptation |
|-------|--------|------------|
| double-check | this repo's improved copy | verbatim |
| grill-me | identical in both repos | verbatim |
| humanise | this repo's rewritten copy | verbatim |
| self-improve | goodbye-tinnitus | rules land in the canonical knowledge, below |
| aokf-maintain | this repo | validator command generalised, below |

Plus the PostToolUse validation hook: after any Edit/Write under
`knowledge/`, validate the canonical knowledge and block on errors — the hook managed
repos lacked until now.

Excluded: playwright-cli (browser automation belongs with the frontend
capability's territory, not the skill pack). A knowledge-capture skill —
teaching agents to write durable learnings into the canonical knowledge mid-task — is a
good idea deferred to the knowledge-upkeep sub-project; noted in
[backlog][sokf:backlog].

## self-improve adaptation

The goodbye-tinnitus version writes approved rules into a managed block in
`CLAUDE.md` and keeps an append-only `learning-log.md`. In a managed repo the
entry point is AGENTS.md and canonical knowledge lives in the canonical project knowledge, so:

- Approved rules go into a `knowledge/learned-rules.md` concept
  (`type: Convention`), searchable and validated like any other concept.
- The learning log is dropped. AOKF's own principle applies: git log on the
  concept is the history.
- Working files (findings, proposed rules) stay ephemeral in `.claude/eval/`.
- The per-rule human review gate is unchanged.

## aokf-maintain adaptation

The validator command becomes `superdev aokf validate knowledge`, with a
one-line note that the superdev repo itself uses
`cargo run --quiet -- aokf validate knowledge`. The AGENTS.md reference check
updates to match the slim search-first AGENTS.md. One file serves both
contexts.

# Distribution: owned repo files

The skills are **owned files in the managed repo**, written by `sync` like
the AGENTS.md scaffold: `.claude/skills/<name>/SKILL.md`, committed, hashed
in `lock.toml`, drift-repaired by the existing machinery. Claude Code loads
project skills from `.claude/skills/` natively — no marketplace, no install
step, no `claude` CLI dependency.

This buys: per-repo versioning (no cross-repo shared state), teammates get
skills and hook by cloning (no superdev on their machines), and the component
plans only `WriteFile`/`EnsureJsonArrayElement` actions — nothing optional,
everything journalled.

Canonical sources live in `crates/lib/superdev-core/assets/skills/`, embedded
with `include_str!` like the other scaffold assets.
superdev owns only its five skill directories; the rest of `.claude/skills/`
is the user's.

Alternatives rejected:

- **Claude Code plugin embedded in the binary, materialised to a user-level
  XDG path.** Version always matches the binary, but marketplace
  registration is user-global: one shared copy, last sync wins across repos,
  and teammates need superdev installed.
- **Release-asset tarball, superpowers-style.** Checksum-pinning an artifact
  the same release produces needs deterministic build ordering; adds a
  network fetch and a `.mise.toml` entry for content the binary can carry.
- **GitHub marketplace (`claude plugin marketplace add six5536/superdev`).**
  No checksum provenance; the installed version floats with the repo instead
  of the managing binary.

# Customisation

Two mechanisms, nothing in between — a skill is either stock plus a project
layer, or entirely yours:

1. **PROJECT.md extension layer.** Every shipped SKILL.md ends with a
   standard trailer: if a `PROJECT.md` exists in the skill's directory, read
   it and apply it; where it conflicts, it wins; if absent, continue.
   superdev never writes or tracks PROJECT.md. Project tweaks ride on top
   while stock improvements keep flowing. (`@`-imports were considered and
   rejected: Claude Code processes them only in memory files, not SKILL.md
   bodies, so the trailer is an explicit conditional read.)
2. **Full takeover.** `custom = ["humanise"]` under `[skills]` in
   `config.toml` releases a skill: content is left in place as a starting
   point, the path leaves the plan and the lock, and `status` reports it as
   custom rather than drifted. Removing it from the list restores stock on
   the next sync. To remove a skill entirely, mark it custom and delete it.

`init --no-skills` disables the whole capability. The hook's settings entry is
always owned — infrastructure, not prose.

# The hook

- A plumbing subcommand, not a script: `superdev aokf hook validate` reads
  the PostToolUse payload from stdin, exits 0 unless the edited path is
  under `knowledge/`, and otherwise validates in-process, printing findings
  to stderr and exiting 2 to block. No bash, no python3 — the same command
  works on every platform superdev ships for. `hook` is a subcommand group;
  future hooks get their own verbs beside `validate`.
- Registered in `.claude/settings.json` with the literal command
  `superdev aokf hook validate`. Hook entries live in an array, so this is
  the array-element analogue of the `.mcp.json` key merge: superdev finds
  its own PostToolUse entry by content, adds or updates it, and leaves the
  user's entries untouched.
- An unreadable payload is a loud exit 2, never a silent skip. A missing
  binary surfaces as the hook command failing (command not found) — softer
  than exit 2, but such a machine cannot run any superdev verb anyway.

# Engine integration

A new component (provider `superdev-skills`, renamed from the placeholder
`superdev-plugin`) fills `Capability::Skills`; the registry entry becomes
`available: true` with the crate version. Planning compares each embedded
asset against the repo copy and emits `WriteFile` for drift, skipping skills
listed in `custom`; the settings merge plans like the `.mcp.json` key.

The pack's version is the binary's version, so `update skills@<version>` is
refused like the other pinned capabilities; bare `sync`/`update` converges on
the binary's content.

# Dogfooding

This repo becomes a managed repo for the skills capability only: committed
`.superdev/config.toml` (other capabilities off — their repo-side files
intentionally differ here) and `lock.toml`. Skills and hook are materialised
by `cargo run -- sync`, replacing the hand-maintained `.claude/skills/`
copies and the hand-written hook entry. The managed hook command expects
`superdev` on PATH, so the devcontainer gets a `scripts/superdev` shim that
resolves its own symlink chain and execs `cargo run --quiet --` against this
tree; it is installed with `ln -sf` into `~/.local/bin`, and it is dev
tooling, never shipped. The pre-PR check list and CI gain
`cargo run -- status`, so asset drift fails the build through the product's
own drift detection instead of a parity test.

# Testing

- Component unit tests: plan on missing/drifted/converged files, `custom`
  exclusion from plan and lock, `update skills@<version>` refusal.
- Hook subcommand tests with stubbed payloads: non-knowledge path exits 0,
  broken knowledge exits 2 with findings, malformed payload exits 2 loudly.
- Registry test asserts the skills slot is available.
- CLI integration tests cover the capability in init/status/sync golden
  paths; `status` on this repo stays clean in CI.
# Behaviour

- Stated in the design sections below rather than gathered here. This spec
  predates the contract that asks for one Behaviour section, and the
  sections were left where they were written rather than reshuffled after
  the fact.

# Acceptance criteria

1. The behaviour described below holds, as proved by the automated cases in
   the test plan. This spec shipped before the contract asked for acceptance
   criteria, and none were written at the time; the tests are the record of
   what was actually accepted.

# Edge cases & errors

- Not recorded separately when this spec was written. What the code does at
  the edges is in the tests named in the test plan, which is the only
  contemporaneous record.

# Out of scope

- Anything outside the skill pack's own files and the hook it installs.

# Test plan: skill pack

## Scope

- The component's plan and claims, the custom list, and the hook.
- Out: everything the sections above place out of scope.

## Risks driving this plan

1. Recorded after the fact. This plan was written when the spec was
   conformed to its contract, not when the feature was built, so it names
   the risks the tests actually cover rather than the ones weighed at the
   time.

## Test cases

### Automated

| # | Case | Type | Inputs / setup | Expected result |
|---|------|------|----------------|-----------------|
| 1 | Plan on missing, drifted and converged files | unit | a temp-dir repo in each state | the expected action list |
| 2 | A custom name is excluded | unit | a manifest naming a custom skill | absent from both plan and lock |
| 3 | The hook subcommand | unit | stubbed payloads | the documented exit codes |

### Manual verification

1. None recorded. The feature shipped under the automated cases above; no
   manual step was written down at the time, and inventing one now would
   claim a check nobody made.

## Exit criteria

- The automated cases above pass.
- `superdev validate` reports no error for this document.

<!-- sokf:links -->
[sokf:backlog]: /knowledge/backlog.md
[sokf:spec-001-cli-core-blueprint-engine]: /knowledge/specs/spec-001-cli-core-blueprint-engine.md
[sokf:spec-002-aokf-mcp-server]: /knowledge/specs/spec-002-aokf-mcp-server.md

---
type: Spec
id: spec-012-bash-output-filter
title: Bash Output Filter Capability
description: A new bash-output-filter capability, default provider rtk — a checksummed mise pin, an owned instruction file, and a managed PreToolUse rewrite hook that compacts command output before it reaches agent context.
status: deprecated
links:
  - rel: relates-to
    to: spec-010-agent-instructions-layer
  - rel: relates-to
    to: spec-001-cli-core-blueprint-engine
---

# Summary

Agents in a managed repo spend context on raw command output — full
git diffs, verbose test logs, directory listings — most of which the
model never needs, and nothing in the blueprint addresses it. A user
who wants rtk (a CLI proxy that filters and summarises command output
before it reaches the model) must install and wire it by hand, and
rtk's own installer writes its guidance into the instruction files
superdev manages, so hand-wiring fights the blueprint the same way
codegraph's installer did.

# Solution

A new single-cardinality capability named for the function:
`bash-output-filter`, default provider rtk. Its component pins the
rtk binary through the checksummed mise pin (the codegraph pattern
from the
[CLI core & blueprint engine spec][sokf:spec-001-cli-core-blueprint-engine]),
platform-scoped so unpublished platforms skip it cleanly,
ships an owned instruction file imported by the aggregator following
the [agent instructions layer][sokf:spec-010-agent-instructions-layer],
and manages a PreToolUse hook in the harness settings that rewrites
Bash commands to their rtk equivalents — so filtering still happens
when an agent forgets the instructions.

# Behaviour

1. The capability is enabled by default: `init` pins rtk, writes the
   instruction file, includes its import in the aggregator, and adds
   the managed hook key. `--no-bash-output-filter` opts out.
2. The pin is platform-scoped through mise's `auto_env` mechanism: a
   committed `.miserc.toml` enables `auto_env`, and the rtk pin lives
   in the platform config files covering the platforms rtk publishes
   bundles for (today: the unix platforms plus windows-x64). A
   machine on an unpublished platform (windows-arm64 today) skips the
   tool silently; `mise install` succeeds everywhere. Components plan
   identically on every machine — mise does the skipping.
3. superdev gains a minimum supported mise version — the first line
   carrying `auto_env` (2026.8) — enforced as a guided error at plan
   time naming the floor and the reason. This repo updates its own
   mise as part of the change.
4. The instruction file tells agents the repo auto-filters command
   output — what rtk is, that the hook rewrites for them — and
   documents the escape hatches for when full raw output is genuinely
   needed: an `RTK_DISABLED=1` prefix inside the command text, or
   rtk's proxy passthrough. It does not tell agents to prefix
   commands themselves. Exact wording is implementation judgement.
5. The hook is a managed key in the harness settings file, matching
   Bash tool calls and running rtk's own Claude Code hook entry
   point, which replaces the command via the harness's
   `updatedInput` mechanism — the same managed-key merge, hashing and
   sweep rules as the knowledge capability's validation hook. The
   hook fails open on every path (verified): unrecognised commands,
   non-Bash input, malformed payloads and a missing rtk binary all
   let the original command run unmodified, and it never emits a
   permission decision.
6. The rtk version is a registry pin with checksum provenance: a
   manifest version off the registry default is refused with the
   standard refusal, and `superdev update bash-output-filter` moves
   it.
7. Disabling the capability sweeps the pin — the platform config
   files and `.miserc.toml` whole — plus the instruction file and the
   hook key, and the next sync rewrites the aggregator without the
   import.
8. `status` exits 1 while any of this is pending in an existing repo
   and 0 once converged, as for any planned work.
9. This repo enables the capability on itself.

# Design decisions

- The slot is named for the function, never the tool, per the
  [architectural rules][sokf:architectural-rules]. `bash-output-filter`
  was chosen (with the user) over broader names like context-economy
  to scope the slot to command output — a future context-reduction
  tool of a different kind should get its own slot, not squat in this
  one.
- superdev does the wiring itself rather than running rtk's installer,
  because the installer edits files superdev manages — the same
  reasoning that kept `codegraph install` out of the code-index
  component.
- The hook ships alongside the instructions rather than
  instructions-only, decided with the user: instruction adherence
  decays over a long session, and the hook makes filtering
  deterministic regardless. The instruction file therefore describes
  the filtering and its escape hatches instead of duplicating the
  hook's job with prefix-it-yourself guidance (also decided with the
  user); the rewrite is idempotent, so a manually prefixed command is
  never double-prefixed.
- The hook fails open because filtering is an optimisation: a missing
  or broken rtk must never block a command. This mirrors the
  fail-soft rule for a missing `claude` binary. rtk's shipped hook
  behaves this way already (verified live on every failure path).
- The hook launches rtk through mise, because the pinned binary is on
  no PATH the hook can assume — the same reason codegraph's MCP
  registration and init run that way.
- Checksum provenance uses the http-backend pin with per-platform
  release urls and sha256 digests, like codegraph; rtk's GitHub
  releases publish exactly that, and mise's github backend carries no
  checksum, which is why it is not used. The registry lock is kept
  despite rtk's fast release cadence (decided with the user): a
  filter does not need to be current, and the no-unvouched-versions
  invariant is worth more than tracking upstream.
- Platform scoping uses mise `auto_env` config files rather than the
  tool-level `os` option or a plain platforms table (decided with the
  user, after verification): an unlisted platform in a platforms
  table hard-fails `mise install`, and the `os` option is strictly
  os-level — it cannot express "windows-x64 yes, windows-arm64 no".
  The cost is the mise version floor above and the pin living in
  platform files instead of one `.mise.toml` edit. codegraph
  publishes all mise platforms, so its pin stays where it is.
- The permission surface shifts with or without the hook: allow and
  deny rules match the rewritten, rtk-prefixed command string, and an
  agent following prefix instructions would produce the same string —
  so this is a caveat of the capability, not of the hook (user's
  observation). It is accepted and documented rather than managed:
  superdev writes no permission entries, and repos with string-
  matched allow/deny rules account for the rtk-prefixed forms (rtk's
  own per-command exclusion config is the opt-out). The rewrite
  itself never suppresses a permission prompt.
- A developer's pre-existing global rtk installation is deliberately
  ignored: the repo wiring must be self-contained for collaborators
  without one, and coexistence is harmless — the rewrite is
  idempotent, so the global and repo hooks firing together change
  nothing (verified). No detection logic.
- `.miserc.toml` is owned by this capability until a second consumer
  of `auto_env` appears; it sweeps with the capability rather than
  lingering as unowned repo state.
- The platform pin files are whole owned files, not targeted keys in
  shared files (revised at implementation from an earlier managed-key
  position): they exist solely for this pin, so whole-file ownership
  is the same behaviour with none of the shared-file merge machinery,
  which stays scoped to `.mise.toml`. A user who adds their own tools
  to them gets the owned-file overwrite-with-backup warning; their
  own platform-conditional pins belong in their own `mise.<env>.toml`
  names or `.mise.toml`.

# Testing

Seams, all existing, as confirmed with the user: the rtk component's
unit tests (plans the platform-file pins, `.miserc.toml`, instruction
file and hook key when missing, nothing when converged, repairs
drift; claims match; foreign version refused; too-old mise gets the
guided floor error); the pipeline-level aggregator test extended to
the new import tracking the enabled set; and the manage journeys —
init produces the pins, `.miserc.toml`, instruction file and hook
key, and the disable journey sweeps them all and drops the aggregator
import. Good tests
assert observable files, keys and reports, not rendering internals.
Prior art: the codegraph component's tests (pin + instruction file +
managed MCP key) and the knowledge capability's validation-hook key
tests.

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

- Filtering anything other than Bash tool calls — file reads and
  searches have their own tools and are untouched.
- Surfacing rtk's own configuration (verbosity, ultra-compact mode,
  per-command exclusions); the stock rewrite behaviour is what ships.
- Managing permission rules — superdev writes no allow or deny
  entries; the permission caveat is documented, not compensated for.
- Alternative providers for the slot; the registry gets the one rtk
  entry.
- rtk's output quality — the provider is trusted for what it is.

# Open questions

The exact hook command line and the instruction file's wording are
implementation judgement, guided by the fail-open and escape-hatch
behaviour above; the managed-key shape, the platform-file layout and
the sweep rules are not. The windows-x64 half of the `auto_env`
scheme is verified only from mise's documented platform names, not
on Windows hardware — the implementation should confirm it on the
Windows CI runner.

# Test plan: bash output filter capability

## Scope

- The component's plan and claims, and the hook registration.
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
| 1 | Plans the pins and files when missing | unit | a fresh repo | the expected action list |
| 2 | Plans nothing when converged | unit | an applied repo | an empty plan |
| 3 | Repairs drift | unit | an edited owned file | the file is replanned |

### Manual verification

1. None recorded. The feature shipped under the automated cases above; no
   manual step was written down at the time, and inventing one now would
   claim a check nobody made.

## Exit criteria

- The automated cases above pass.
- `superdev validate` reports no error for this document.

<!-- sokf:links -->
[sokf:architectural-rules]: /knowledge/architectural-rules.md
[sokf:spec-001-cli-core-blueprint-engine]: /knowledge/specs/spec-001-cli-core-blueprint-engine.md
[sokf:spec-010-agent-instructions-layer]: /knowledge/specs/spec-010-agent-instructions-layer.md

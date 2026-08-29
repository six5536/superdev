---
type: Spec
id: spec-010-agent-instructions-layer
title: Agent Instructions Layer
description: AGENTS.md becomes the user's file reached by one ensured import; superdev's instructions live in an owned, fenced .agents/superdev.md aggregating per-capability instruction files, and code-index gains its missing agent wiring.
status: stable
links:
  - rel: relates-to
    to: spec-008-knowledge-owned-skills
    note: The capability-carried-files pattern the instruction files follow.
  - rel: relates-to
    to: spec-001-cli-core-blueprint-engine
    note: Introduced the AGENTS.md scaffold this spec retires.
---

# Summary

superdev writes AGENTS.md as a scaffold — introduced by the
[CLI core & blueprint engine spec][sokf:spec-001-cli-core-blueprint-engine] —
and fills it with its own
content, so the file agents read first is superdev's, not the user's. A
user who wants their own agent rules must edit around superdev's
structure, and superdev can never update its guidance there again —
scaffolds are written once.

The code-index capability has the opposite problem: it builds an index
and tells no one. codegraph ships an MCP server and CLI query commands,
but the component registers neither, so an agent in a managed repo has
a built index and no signal that it exists. The gap was found when an
agent working in this very repo was asked why it was not using
codegraph.

# Solution

Split ownership at an import boundary. AGENTS.md is the user's file;
superdev ensures a single `@.agents/superdev.md` line in it, exactly as
it ensures `@AGENTS.md` in CLAUDE.md. `.agents/superdev.md` is a
superdev-owned file: a `<superdev-system>` fence wrapping a short
general prompt plus one import per enabled capability's instruction
file. Each capability that has something to tell agents ships its own
instruction file, following the capability-carried-files pattern of the
[knowledge-owned skills spec][sokf:spec-008-knowledge-owned-skills] —
knowledge moves its current AGENTS.md content into
`.agents/aokf.md`; code-index gains `.agents/codegraph.md` and an MCP
registration for `codegraph serve --mcp`. This repo enables
`[code-index]` on itself.

# Behaviour

1. Every managed repo's AGENTS.md contains the line
   `@.agents/superdev.md`, ensured like the CLAUDE.md import: created
   as a one-line file when AGENTS.md is absent, appended when present,
   re-added by the next sync when deleted, never hashed and never
   orphaned. The rest of the file is the user's; superdev never
   rewrites it.
2. The aokf component stops writing AGENTS.md entirely. Repos that
   already have the old scaffolded AGENTS.md keep it untouched
   (scaffolds were never locked, so nothing orphans); the ensured line
   is appended, and the run reports once that AGENTS.md is now the
   user's file and superdev's old sections in it can be trimmed.
3. `.agents/superdev.md` is an owned repo-level file, rewritten on
   sync: a `<superdev-system>`…`</superdev-system>` fence wrapping a
   short prompt naming superdev as the repo's setup manager, the
   imports of the general rules (`coding.md`, `prose.md` — repo-level
   write-once scaffolds every managed repo gets), and one
   `@.agents/<file>` import per enabled capability that ships an
   instruction file. Disabling a capability removes its import on the
   next sync; no import ever points at a file that will not exist.
4. `.agents/aokf.md` is owned by the knowledge capability and carries
   what the AGENTS.md scaffold carried: the AOKF spec import, the
   canonical-knowledge section pointing at the knowledge index, the
   working-with-the-knowledge guidance, and the validation rules
   that previously lived in their own owned file.
5. `.agents/codegraph.md` is owned by the code-index capability and
   tells agents the repo has a code index and how to query it: the
   `codegraph_explore` MCP tool when the harness loads MCP, and the CLI
   (`mise exec http:codegraph -- codegraph explore "<question>"`, plus
   the narrower query commands) for subagents and MCP-less harnesses.
6. The code-index capability registers codegraph's MCP server in
   `.mcp.json` under `mcpServers.codegraph`, launching it through
   `mise exec` — the same managed-key merge, hashing and sweep rules as
   the `superdev-aokf` entry.
7. Disabling knowledge sweeps `.agents/aokf.md`; disabling code-index
   sweeps `.agents/codegraph.md` and the `mcpServers.codegraph` key.
   Either way the next sync rewrites `.agents/superdev.md` without the
   import. The skills and frontend capabilities ship no instruction
   file.
8. This repo's own manifest gains `[code-index]`: sync pins codegraph,
   builds the index, writes `.agents/codegraph.md`, and registers the
   MCP server. `knowledge/development-procedure.md` stops recording
   code-index as off.
9. `status` exits 1 while any of this is pending in an existing repo —
   the ensured line, the new owned files, the retired scaffold's
   replacement — and 0 once settled, as for any planned work.

# Design decisions

- The fence lives inside `.agents/superdev.md`, not AGENTS.md. The
  aggregator is wholly superdev's, so the fence is not protecting an
  edit boundary; it marks where superdev-managed context begins and
  ends for a reader of the rendered prompt.
- The aggregator is a repo-level entry rather than any capability's,
  because its content is derived from the enabled set — the same reason
  the `.gitignore` lines and the CLAUDE.md import are repo-level. It is
  an owned, hashed file, unlike those ensured lines, because superdev
  must rewrite it as capabilities toggle.
- The general rules (coding, prose) moved from knowledge-owned to
  repo-level scaffolds imported by the aggregator directly: they are
  superdev's general agent guidance, not knowledge behaviour, so a
  `--no-knowledge` repo gets them too. The validation rules merged into
  `.agents/aokf.md` — they describe the knowledge capability's own
  hook, so they live and die with it. (Revised during implementation
  from an earlier keep-them-knowledge-owned position.)
- Retiring the AGENTS.md scaffold needs no migration machinery:
  scaffold writes never enter the lock, so there is no claim to drop
  and no orphan to sweep. The old file simply is the user's, which is
  what the new design says it should be.
- The MCP registration launches codegraph through `mise exec` because
  the pinned binary is on no PATH the client can see — the same reason
  `codegraph init` runs that way.
- Both the MCP registration and the instruction file ship, mirroring
  what codegraph's own installer does (MCP config plus instruction-file
  markers, because subagents receive no MCP guidance). superdev does
  the wiring itself rather than running `codegraph install`, because
  the installer would edit files superdev manages.
- The one-time trim hint rides the ensure-line action's report: it
  fires exactly when the line is appended to a pre-existing AGENTS.md,
  which is exactly the migrating population.

# Testing

Seams, all existing: the aokf component's unit tests (plans the ensured
line and `.agents/aokf.md`, no AGENTS.md write, claims match); the
codegraph component's unit tests (plans instruction file, MCP key, pin
and init; claims match); a pipeline-level test that the aggregator's
imports track the enabled set; and the manage journeys — init asserts a
fresh repo gets the one-line AGENTS.md, the fenced aggregator, both
`.mcp.json` servers and the instruction files, while the
disable-code-index journey asserts the instruction file and MCP key are
swept and the aggregator loses the import. Good tests here assert
observable files and reports, not rendering internals. Prior art: the
S008 relocation tests, the existing init and disable journeys, and the
aokf component's items tests.

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

- Rewriting or trimming existing repos' AGENTS.md content — the file is
  the user's; the hint is the whole migration.
- Recognising and replacing historical scaffold versions byte-for-byte;
  decided against in favour of the hint.
- Instruction files for the skills and frontend capabilities — Claude
  Code discovers skills and plugins natively.
- Tuning codegraph's unlisted MCP tools (`CODEGRAPH_MCP_TOOLS`); the
  default `codegraph_explore` surface is what ships.
- Any change to the knowledge capability's tools or knowledge format.

# Open questions

The exact wording of the general prompt, the instruction files and the
trim hint is implementation judgement; the structure above is not.

# Test plan: agent instructions layer

## Scope

- The component's items, the aggregator, and the codegraph wiring.
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
| 1 | AGENTS.md is never written | unit | a fresh repo | the ensured line only |
| 2 | The aggregator tracks the enabled set | unit | manifests with capabilities on and off | one import per enabled capability |
| 3 | Disabling sweeps the wiring | end-to-end | `init` then disable code-index | file, MCP key and import all removed |

### Manual verification

1. None recorded. The feature shipped under the automated cases above; no
   manual step was written down at the time, and inventing one now would
   claim a check nobody made.

## Exit criteria

- The automated cases above pass.
- `superdev validate` reports no error for this document.

<!-- sokf:links -->
[sokf:spec-001-cli-core-blueprint-engine]: /knowledge/specs/spec-001-cli-core-blueprint-engine.md
[sokf:spec-008-knowledge-owned-skills]: /knowledge/specs/spec-008-knowledge-owned-skills.md

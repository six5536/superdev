---
type: Spec
id: spec-009-knowledge-carried-skills
title: Knowledge-Carried Skills
description: The aokf component ships the full converted skill set and the workflows capability is dropped; a manifest naming it gets a guided error.
status: stable
links:
  - rel: supersedes
    to: spec-005-workflows-provider-default
  - rel: supersedes
    to: spec-006-workflows-skill-overrides
  - rel: relates-to
    to: spec-008-knowledge-owned-skills
---

# Summary

The workflows capability materialises upstream skills that fight the
knowledge: they write `CONTEXT.md`, `docs/adr/` and `.scratch/`,
and need a per-repo override file to redirect them into the canonical knowledge. The
24 aokf-converted skills bake that redirection in, so the provider's
whole job — chosen by the
[workflows-provider-default spec][sokf:spec-005-workflows-provider-default]
and overlaid by the
[skill-overrides spec][sokf:spec-006-workflows-skill-overrides] — is
done better by the knowledge capability. Keeping both means name
collisions in `.claude/skills/` and two sources of truth for one flow.

# Solution

Drop the workflows capability entirely. The aokf component carries the
converted skill set — 25 skills, the derived set plus
`aokf-bootstrap` and `aokf-maintain` — as owned files under `.claude/skills/`, exactly as it
already carries its two lifecycle skills
([knowledge-owned skills][sokf:spec-008-knowledge-owned-skills]). A
manifest still naming `[workflows]` fails with a guided error.

# Behaviour

1. A knowledge-enabled repo materialises all 25 skills — each skill's
   whole directory: `SKILL.md`, companions, `agents/`, scripts — as
   knowledge-owned files. `[knowledge] custom` releases any of them by
   name, releasing the whole directory.
2. A `--no-knowledge` repo gets none of them: the skills are
   knowledge-coupled, so only the skill pack remains there.
3. `init` writes no `[workflows]` table; `--workflows-provider` and
   `--no-workflows` are gone. `update workflows` is an
   unknown-capability error.
4. Loading a manifest with a `[workflows]` table fails:
   `the workflows capability was removed — delete the [workflows] table
   (moving any custom names to [knowledge]); its skill set now ships
   with the knowledge capability. superpowers users: claude plugin
   install superpowers`.
5. After the table is deleted, the next sync swaps same-named skills to
   knowledge ownership (overwrite-with-backup where content differs)
   and sweeps the dropped upstream files — ask-matt, grill-with-docs,
   to-tickets, setup-matt-pocock-skills, teach's old form and the
   override file — as orphans. A user-edited copy is released and
   reported, never removed.
6. `.agents/MATT-POCOCK-SKILLS.md` and `.agents/SUPERPOWERS.md` are no
   longer written anywhere; AGENTS.md's skeleton loses its override
   slot.
7. `status` exits 1 while the migration swap is pending, 0 after, as
   for any planned work.

# Design decisions

- The set ships embedded in the binary like every aokf asset (69 files
  across 25 directories, plus the MIT notice), each an owned file at
  `.claude/skills/<skill>/<path>`. A skill is its directory: companions
  and harness configs materialise with it.
- The guided error happens at manifest load and never rewrites
  `config.toml` — the manifest is the user's file.
- superpowers stops being a provider; the plugin remains installable by
  hand and the guided error says how.
- The mattpocock-skills and superpowers registry entries, pins,
  components and overrides are deleted, not gated.
- Spec and plan filenames migrated to `Snnn`/`Pnnn` in this arc; the
  date convention dies with the override file that mandated it.

# Testing

Seams: the registry (no workflows capability), manifest load (the
guided error), the aokf component's items (full set, custom release,
hook intact), pipeline adoption and custom reporting (knowledge covers
all shipped names), and the CLI end-to-end: init materialises the set
with no `[workflows]` table; a mattpocock manifest errors with the
guidance; after the table is deleted, sync swaps ownership and sweeps
the dropped skills. Prior art: the S008 relocation tests and the
provider-switch sweep tests this change deletes.

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

- Choosing which converted skills ship per repo — `[knowledge] custom`
  already opts out per skill.
- The skill pack (double-check, humanise, self-improve) and the other
  capabilities.
- Removing the superpowers plugin from user machines.

# Open questions

None.

# Test plan: knowledge-carried skills

## Scope

- The registry, manifest load, the component's items, and adoption.
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
| 1 | The workflows capability is gone | unit | the registry | no such entry |
| 2 | A workflows table is refused | unit | a manifest carrying one | the guided error |
| 3 | The component's items | unit | a temp-dir repo | the full set, custom release honoured, hook intact |

### Manual verification

1. None recorded. The feature shipped under the automated cases above; no
   manual step was written down at the time, and inventing one now would
   claim a check nobody made.

## Exit criteria

- The automated cases above pass.
- `superdev validate` reports no error for this document.

<!-- sokf:links -->
[sokf:spec-005-workflows-provider-default]: /knowledge/specs/spec-005-workflows-provider-default.md
[sokf:spec-006-workflows-skill-overrides]: /knowledge/specs/spec-006-workflows-skill-overrides.md
[sokf:spec-008-knowledge-owned-skills]: /knowledge/specs/spec-008-knowledge-owned-skills.md

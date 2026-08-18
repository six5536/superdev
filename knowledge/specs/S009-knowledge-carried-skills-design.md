---
type: Spec
id: spec-knowledge-carried-skills
title: Knowledge-Carried Skills
description: The aokf component ships the full converted skill set and the workflows capability is dropped; a manifest naming it gets a guided error.
status: draft
links:
  - rel: supersedes
    to: spec-workflows-provider-default
  - rel: supersedes
    to: spec-workflows-skill-overrides
  - rel: relates-to
    to: spec-knowledge-owned-skills
---

# Problem

The workflows capability materialises upstream skills that fight the
knowledge bundle: they write `CONTEXT.md`, `docs/adr/` and `.scratch/`,
and need a per-repo override file to redirect them into the bundle. The
24 aokf-converted skills bake that redirection in, so the provider's
whole job — chosen by the
[workflows-provider-default spec](S005-workflows-provider-default-design.md)
and overlaid by the
[skill-overrides spec](S006-workflows-skill-overrides-design.md) — is
done better by the knowledge capability. Keeping both means name
collisions in `.claude/skills/` and two sources of truth for one flow.

# Solution

Drop the workflows capability entirely. The aokf component carries the
converted skill set — 24 skills plus `aokf-bootstrap` and
`aokf-maintain` — as owned files under `.claude/skills/`, exactly as it
already carries its two lifecycle skills
([knowledge-owned skills](S008-knowledge-owned-skills-design.md)). A
manifest still naming `[workflows]` fails with a guided error.

# Behaviour

1. A knowledge-enabled repo materialises all 26 skills — each skill's
   whole directory: `SKILL.md`, companions, `agents/`, scripts — as
   knowledge-owned files. `[knowledge] custom` releases any of them by
   name, releasing the whole directory.
2. A `--no-knowledge` repo gets none of them: the skills are
   bundle-coupled, so only the skill pack remains there.
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

- The set ships embedded in the binary like every aokf asset (~70 files
  across 26 directories), each an owned file at
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

# Out of scope

- Choosing which converted skills ship per repo — `[knowledge] custom`
  already opts out per skill.
- The skill pack (double-check, humanise, self-improve) and the other
  capabilities.
- Removing the superpowers plugin from user machines.

# Open questions

None.

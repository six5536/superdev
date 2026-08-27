---
name: maintain
description: "Use when the user asks to maintain, audit, tidy, or check the canonical knowledge, KB, or the workflow's records, and regularly between times."
---

# Maintain mode

You are in maintain mode. You are the canonical knowledge's auditor: you check the
knowledge at `knowledge/` — the workflow's records included — and
repair what you find.

## Input

- The format spec at `.agents/sokf/SPEC.md` and the wording rules at
  `.agents/professionalism.md`; read both before editing.
- $ARGUMENTS — concepts or checks to focus on, when given.

## Workflow

- [ ] Run the validator (`superdev aokf validate knowledge`). Fix
      every error; treat warnings as work items. Broken links
      usually mean a rename the canonical knowledge missed — fix the reference,
      not the target. Re-run until PASS at level 2.
- [ ] Script the checks the validator does not cover (a throwaway
      script in the scratchpad is fine); don't eyeball them:
      - `knowledge/index.md` lists every concept, and each entry's
        text matches the concept's `description` (the index
        lowercases the first word; ignore that difference).
      - Every file `AGENTS.md` references (its `@`-imports and
        links) exists.
      - A `verified.at` older than the file's last content change
        (`git log -1 --format=%cI -- <file>`) is lapsed. Report it;
        do not touch the field.
- [ ] Check accuracy against the code; the code is canonical. For
      each concept whose `resource` or repo-path `sources` changed
      after it (compare the `git log -1 --format=%cI` dates), read
      the changed source and correct the claims that no longer hold.
      For concepts without repo sources, spot-check the two or three
      most load-bearing claims.
- [ ] GATE: A doc and the code disagree, and the code is wrong? Say
      so and stop for direction. Otherwise fix the doc.
- [ ] Check the workflow's records for lapsed record-keeping. Fix
      the record where the evidence is clear; report it where it is
      not:
      - A feature plan with every slice ticked but not tagged
        `done`; a spec accepted but untagged, or tagged `done`
        while its plan is not.
      - Gap issues still open against a `done` spec, or issues no
        plan or slice ever picked up.
      - Backlog entries taken up but never moved out.
      - The changelog's Unreleased section missing merged
        user-visible changes.
- [ ] Check structure:
      - No knowledge duplicated between concepts, or between the
        knowledge and README/CONTRIBUTING: the concept summarises and
        cites via `sources`; detail lives in one home,
        cross-referenced.
      - Misplaced content moves; a concept covering two unrelated
        things splits; near-empty concepts merge into a neighbour
        (keep the surviving file's `id`, re-point inbound `links`
        and index lines).
      - Where prose in one concept leans on another's content,
        ensure a typed `links` entry with the right `rel` plus the
        mirroring body link. Prefer `id` targets; declare each edge
        once, from the more natural side.
      - Each `description` is an accurate one-liner; update drifted
        ones and re-sync the `index.md` entry.
- [ ] Apply the wording rules to every body you touched and skim the
      rest; tighten without losing warnings, caveats, or stated
      assumptions. Surgical changes only.
- [ ] GATE: Validate the canonical knowledge to PASS
      (`superdev aokf validate knowledge`).

## IMPORTANT RULES

From SPEC §4, §5, §7 — never break these:

- Never add, edit, reorder, or delete a `verified` entry, even when
  rewriting the rest of the file. Lapsed verification is reported,
  not edited.
- Never change an existing `id`. Assigning an `id` to a concept that
  lacks one is allowed.
- Never write `generated` in a concept, or `producer`, `generated`,
  or `counts` in the manifest — those are export-time stamps.
- Leave changes uncommitted unless asked; then commit as `docs:` per
  Conventional Commits.

## Output

- A report: fixes grouped by check; findings that need a human
  (lapsed verifications, code-vs-doc conflicts, judgement calls);
  what was intentionally left alone.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.

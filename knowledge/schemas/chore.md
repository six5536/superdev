---
type: Schema
id: schema-chore
title: Chore Schema
description: Scoped mechanical work filed in the issue tracker — the surfaces it touches and what done means, with no room for a root cause it does not have.
---

# Chore Schema

Structural rules for scoped mechanical work, filed in the issue tracker
as `issue-{nnn}-chore-{slug}`, numbered after the highest across all of its kind's folders — a duplicate number is an error — and placed in its lifecycle folder by `superdev validate --fix`: a rename, a migration, a
sweep, a
cleanup. It shares the tracker with `schema-bug-report` and
`schema-feature-request` — the same id shape, the same lifecycle — and
differs in its body: work whose shape is already known
states where it reaches and when it is finished, and is never asked for a
symptom, a repro or a root cause.

````yaml
description: >
  Scoped mechanical work: what changes, every surface it reaches with the
  count that bounds it, and the check that says it is done.
line-limit: 800

frontmatter:
  type:
    required: true
    const: Chore
  id:
    required: true
    pattern: '^issue-\d{3}-chore-[a-z0-9-]+$'
  title:
    required: true
    description: The one-line statement of the work.
  description:
    required: true
  lifecycle:
    enum: [open, done, wontfix]
    description: >
      The folder is the value: open while the work is outstanding, done
      when it landed, wontfix when it will not be done.

sections-ordered: true
sections:
  - heading-pattern: '^Chore: .+$'
    level: 1
    required: true
    description: >
      Title heading naming the work, e.g. "Rename the format validator
      off the word format".
  - heading-pattern: "^(Decided|Resolved|Resolved in part|Won't fix)$"
    level: 2
    repeatable: true
    content: prose
    description: >
      How it ended, added when it does: what was decided and by whom, what
      shipped and where, or why it will not be done. Sits directly under the
      title, before the report itself, because a reader who opens a settled
      issue wants the verdict before the evidence — every settled issue on
      file puts it there. Absent while the issue is outstanding, which is
      what distinguishes an open one from a settled one at a glance.
  - heading: "Summary"
    level: 2
    required: true
    content: prose
    description: >
      One or two sentences: what changes, and why it is worth doing. No
      symptom — nothing is broken, or this would be a bug report.
  - heading: "Surfaces"
    level: 2
    required: true
    content: bullet-list
    description: >
      One bullet per place the work reaches, each with the count or the
      command that measured it. This is what bounds the work: a chore
      whose surfaces are guessed at is a chore that will be found
      half-done.
  - heading: "Definition of done"
    level: 2
    required: true
    content: bullet-list
    description: >
      Each bullet checkable by someone who did not do the work, and at
      least one of them a command with the result that counts as a pass.

  - heading: "Comments"
    level: 2
    content: prose
    description: >
      Conversation history, appended as it happens — the tracker's
      convention says append, so this sits last, where the verdict does not.

example: |
  ---
  type: Chore
  id: issue-042-chore-drop-the-legacy-cache-directory
  title: The pre-0.2 cache directory is still written and never read
  description: Two cache directories exist; only one is read, and the other is written on every run.
  lifecycle: open
  ---

  # Chore: drop the pre-0.2 cache directory

  ## Summary

  `.superdev/cache/legacy/` is written on every run and read by nothing
  since 0.2.0. Removing it drops a write from the hot path and a
  directory from every managed repo.

  ## Surfaces

  - `src/cache/legacy.rs`, 210 lines, the only writer
    (`git grep -l legacy_cache -- crates`).
  - Four call sites in `src/pipeline.rs` (`git grep -c legacy_cache`).
  - One `.gitignore` line written by `init`.
  - Nine tests naming the directory (`cargo nextest list | grep -c legacy`).

  ## Definition of done

  - `git grep -i legacy_cache` returns nothing outside the changelog.
  - `cargo nextest run --workspace` passes with no test deleted.
  - A repo synced from 0.2.0 loses the directory on its next `sync`.
````

---
name: template-update
description: "Use when the user asks to update from the project template, pull in template changes, or adopt a template."
disable-model-invocation: true
---

# Template-update mode

You are in template-update mode. You are a maintainer: you bring the
repo's files up to date with the project template shipped in the
installed `superdev` binary.

## Input

- $ARGUMENTS — the template name, when the user gives one.

## Workflow

- [ ] GATE: Working tree dirty? Stop: the whole update must be one
      reviewable, revertable diff.
- [ ] Identify the template from `[template]` in
      `.superdev/config.toml`. Present: confirm the recorded
      template and project name with the user. Absent: run
      `superdev template list`, judge which shipped template matches
      the repo's shape (workspace layout, packaging, CI), and
      propose it — or say none fits and stop. Infer the project name
      from the repo (package name, workspace name, directory) and
      confirm both with the user.
- [ ] GATE: Compare `[template].version` (when recorded) with
      `superdev --version`. Recorded newer than the binary? Stop and
      say to update superdev first. Equal? Nothing to do — say so
      and stop, unless the user wants a re-check. Older or absent?
      Continue.
- [ ] Render the current template into a scratch directory outside
      the repo: `superdev template render <name>
      --name "<project-name>" --dir <scratch>`. Keep the printed
      `project-name`, `project-slug`, and `project-ident` lines for
      the recording step rather than re-deriving them.
- [ ] Classify every difference. The merge base for a file is its
      content as first seeded: recover it from git history (the
      commit that added the path, or the last template-update
      commit). Against that base, compare ours (working tree) and
      theirs (the render):
      - Only the template changed → propose taking theirs.
      - Only the user changed it → keep ours, no question needed.
      - Both changed → conflict; propose a merge, the user decides.
      - In the render but not the repo → git history says whether
        the user deleted it (respect that) or the template grew it
        (propose adding).
      - Seeded but gone from the render → propose deleting only if
        unmodified since seeding; otherwise just report it.
      An adoption has no base: every collision between repo and
      render is a conflict for the user.
- [ ] GATE: Present one summary of everything — adds, updates,
      conflicts, deletions, grouped by area (CI workflows, release
      scripts, dev container, repo docs, policy configs, workspace
      files) — and ask whether to proceed at all.
- [ ] Ask one question per affected area: accept or skip. Within an
      accepted area, apply the clean changes and bring each conflict
      back as its own question with a proposed resolution.
- [ ] Record what was applied: write `[template]` in
      `.superdev/config.toml` — the table `init` writes: `name`, the
      `project-name` and `project-slug` lines from the render
      (`project-ident` is derived, not recorded), and
      `version = "<superdev --version>"`.
- [ ] Verify: run the repo's own gates (build, tests, lint —
      whatever its CONTRIBUTING or package scripts define) and
      delete the scratch directory.

## IMPORTANT RULES

- Apply nothing the user hasn't seen in the summary.
- Leave lockfiles and generated files alone.

## Output

- A report: what was applied, skipped, and left unresolved.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.

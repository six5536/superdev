---
name: template-update
description: Update a repo from its superdev project template, or adopt a template into a repo that never used one. Use when the user asks to update from the template, pull in template changes, or adopt a project template.
disable-model-invocation: true
---

# Template update

Bring the repo's files up to date with the project template shipped in the
installed `superdev` binary.

Require a clean working tree before changing anything, so the whole update
is one reviewable, revertable diff.

## 1. Identify the template

Read `[template]` in `.superdev/config.toml`.

- Present: confirm the recorded template and project name with the user.
- Absent: run `superdev template list` and judge which shipped template
  matches the repo's shape (workspace layout, packaging, CI). Propose it,
  or say none fits and stop. Infer the project name from the repo (package
  name, workspace name, directory) and confirm both with the user.

## 2. Check versions

Compare `[template].version` (when recorded) with `superdev --version`.

- Recorded version newer than the binary: stop and say to update superdev
  first.
- Equal: nothing to do. Say so and stop, unless the user wants a re-check.
- Older or absent: continue.

## 3. Render the current template

```sh
superdev template render <name> --name "<project-name>" --dir <scratch>
```

Use a scratch directory outside the repo. Keep the printed `project-name`,
`project-slug` and `project-ident` lines for step 6 rather than re-deriving
them.

## 4. Classify every difference

The merge base for a file is its content as first seeded: recover it from
git history (the commit that added the path, or the last template-update
commit). Against that base, compare ours (working tree) and theirs (the
render):

- only the template changed → propose taking theirs
- only the user changed it → keep ours, no question needed
- both changed → conflict; propose a merge and let the user decide
- in the render but not the repo → git history says whether the user
  deleted it (respect that) or the template grew it (propose adding)
- seeded but gone from the render → propose deleting only if unmodified
  since seeding; otherwise just report it

Leave lockfiles and generated files alone. In an adoption there is no
base: every collision between repo and render is a conflict for the user.

## 5. Summarise, then ask by area

Present one summary of everything first — adds, updates, conflicts,
deletions, grouped by area (CI workflows, release scripts, dev container,
repo docs, policy configs, workspace files) — and ask whether to proceed
at all. Then one question per affected area: accept or skip. Within an
accepted area, apply the clean changes and bring each conflict back as its
own question with a proposed resolution. Apply nothing the user hasn't
seen in the summary.

## 6. Record what was applied

Write `[template]` in `.superdev/config.toml` — the table `init` writes:
`name`, the `project-name` and `project-slug` lines from step 3
(`project-ident` is derived, not recorded), and
`version = "<superdev --version>"`.

## 7. Verify and report

Run the repo's own gates (build, tests, lint — whatever its CONTRIBUTING
or package scripts define), delete the scratch directory, and report what
was applied, skipped, and left unresolved.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.

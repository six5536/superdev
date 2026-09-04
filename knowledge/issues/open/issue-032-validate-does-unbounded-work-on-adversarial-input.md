---
type: Issue
id: issue-032-validate-does-unbounded-work-on-adversarial-input
title: validate does unbounded work on adversarial input
description: A file with a 100k-key frontmatter takes 41 s and 100k findings because the frontmatter is re-parsed per check, and reads carry no size cap — a hostile file can stall the PostToolUse hook or flood the transcript.
kind: bug
lifecycle: open
links:
  - rel: references
    to: issue-019-validate-reads-a-named-file-as-a-skill
    note: Found by the security review in I019's acceptance; the cost predates that feature.
---

# Bug: validate does unbounded work on adversarial input

## Summary

The document checks re-parse a file's frontmatter per check and cap
neither findings nor file size, so one adversarial file makes a run —
including the automatic PostToolUse hook — take tens of seconds and
emit a hundred thousand findings. Found by the security review in the
acceptance of
[I019][sokf:issue-019-validate-reads-a-named-file-as-a-skill]; the
cost predates that feature, which added one linear parse on top. Low
severity: it needs a hostile or pathological file in the checked tree.

## Context

Observed on the feature/validate-path-dispatch head, 2026-09-01. The
platform is any; measured on Linux, devcontainer. To reproduce:

1. Generate a markdown file whose frontmatter holds 100,000 keys.
2. Place it where a run reaches it, or name it directly.
3. Run `superdev validate <file>` and time it.

## Behaviour

The run stays within seconds and reports a bounded number of findings;
a file past a size bound is refused naming the path.

Instead, the run takes 41 s of CPU and reports 100,005 findings for the
100k-key file; `read_to_string` loads a file of any size whole.

The cause: `schema::read::parse_frontmatter` runs per check rather than
once per file, making the work superlinear in frontmatter size, and
`read` in `crates/lib/superdev-core/src/validate/mod.rs` has no size
cap.

## Scope

The frontmatter parse and the two caps.

- Fix: parse each file's frontmatter once and share the result across
  checks; cap the per-file finding count and the readable file size,
  each refusal naming the path.
- Workaround: none needed for trusted trees.
- Regression risk: the finding order and texts are pinned by the
  snapshot goldens; a shared parse must not reorder them. The coverage
  gate and the snapshot suites catch a check that silently stops
  reporting.

<!-- sokf:links -->
[sokf:issue-019-validate-reads-a-named-file-as-a-skill]: /knowledge/issues/done/issue-019-validate-reads-a-named-file-as-a-skill.md

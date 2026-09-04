---
type: Idea
id: idea-003-ask-worktree-or-main-working-tree
title: Ask whether work runs in a worktree or the main working tree
description: When a feature's branch is cut, ask the user whether the work runs in a linked git worktree or in the main working tree, instead of always switching the main checkout.
status: draft
---

# Idea: ask whether work runs in a worktree or the main working tree

When the workflow cuts a feature's branch, ask the user where the work
runs: in a linked git worktree of its own, or in the main working tree
(git's name for the primary checkout). Today `/scope` switches the main
checkout to the new branch, which takes the whole repo with it — a
worktree would leave the main checkout free for other work in parallel.

## Open questions

- Where does the question live — `/scope` at branch time, or a repo-level
  preference so it is asked once?
- What tooling assumes the main working tree — the dev shim, `.superdev/`
  cache paths, the MCP servers, hooks?
- How does an unattended run (`/execute-plan`) pick without a
  user to ask?

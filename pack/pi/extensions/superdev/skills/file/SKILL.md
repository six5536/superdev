---
name: file
description: Use when the user asks to capture a bug, feature, chore or idea for later.
allowed-tools: read edit write bash sokf_search sokf_graph superdev_ask
---

# File an issue or idea

Capture the user's intent; do not scope or implement it. Keep questions to missing essential information. Infer a concise title, description, and bug/feature/chore category, or use an Idea for a draft thought without an implementation obligation. An explicit request to file authorizes creating and committing that record; otherwise confirm the proposed content first.

## Choose the destination

1. Inspect Git to identify the current checkout, its edits, and the repository's default branch (normally `main`). Check `git config superdev.defaultBranch`, then origin's symbolic HEAD, then an unambiguous local `main`, `master`, or `trunk`; ask if the default is ambiguous.
2. If already on the default branch, file there. Otherwise use a worktree on the default branch: reuse an existing checkout or create a temporary one with ordinary Git commands. Never switch the user's branch or pause its workflow.
3. Preserve unrelated staged, unstaged, and untracked work. If another writer is using the destination, wait rather than race it. Do not stash, reset, revert, force a checkout, overwrite pending files, merge product work, or push. Never restore or discard a change you did not author, even when it appears in your own working tree: another session's in-flight edits, and the repairs those edits make necessary, belong to that writer.

## Author the record

1. Read `../../../../skills/sokf-authoring/SKILL.md`, then the destination's issue or idea schema and index. Search for duplicates and inspect plausible matches, including the default branch's latest records rather than only the caller's checkout. Reuse an existing matching record instead of filing it twice.
2. Inspect existing issue or idea IDs across their lifecycle folders, including pending files. Choose the next unused number for that record kind and a descriptive slug yourself. Do not derive a plan number or create a plan.
3. Before writing, refuse pre-existing edits to the destination index or record path, including staged, unstaged, untracked, and ignored files. Run read-only `superdev validate` first. A stale generated block whose source another session is editing is that writer's in-flight work rather than a defect: continue, and leave any repair of it uncommitted. Stop only when unrelated knowledge carries a genuine defect that you would have to author a fix for. Write the schema-conforming record in the destination's canonical knowledge and add its index entry. Preserve the user's meaning and relevant links. Use the SOKF-authoring rules and safe mutation tools; when working in another worktree, run the ordinary SOKF CLI from that worktree so mutations target its knowledge, not the caller's.
4. Run `superdev validate --fix` in the destination until PASS. Inspect every repair and the diff. Repair legitimately touches documents you never opened, because `--fix` refills every generated block from its current source; that output is correct and stays in the working tree. Do not commit repairs to unrelated documents, and never undo them. If a concurrent writer has taken the chosen ID, stop and reassess rather than overwriting work.
5. Recheck the branch and pending edits. Commit only the authored record and its index with ordinary Git, keeping unrelated staged changes out of the commit. Stage only those paths, then use an explicit path-limited `git commit --only -- <record> <index>` after reviewing their diff; staging selected files followed by an unrestricted commit would include others' staged work. Disable hooks and signing for this bounded documentation commit; do not use `git add -A` or commit the whole index.
6. Remove only a temporary worktree you created, and only when it is clean. On failure, preserve unfinished files and report their location; do not force-remove them. Leave pre-existing worktrees alone.

Report the record ID, default branch, and commit. Filing does not start SCOPE or BUILD, create an implementation branch, release workflow ownership, or merge existing work.

If `PROJECT.md` exists beside this skill, read it and apply its instructions; it takes precedence on conflicts.

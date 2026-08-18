---
name: to-plan
description: Break a spec or the current conversation into executable work — one ephemeral Plan concept by default, or a set of issues with blocking edges when the work is parallel, independent, or long-horizon.
disable-model-invocation: true
---

# To Plan

Break a spec, or the current conversation, into executable work: an ephemeral **plan** or a set of **issues**, in the knowledge bundle either way.

## Process

### 1. Gather context

Work from whatever is already in the conversation context. If the user passes a reference (a spec id or path, an issue) as an argument, read its full body.

### 2. Explore the codebase (optional)

If you have not already explored the codebase, do so to understand the current state of the code. Use the glossary concept's vocabulary throughout, and respect Decision concepts and stable specs in the area you're touching.

Look for opportunities to prefactor the code to make the implementation easier. "Make the change easy, then make the easy change."

### 3. Choose the form

Default to a **plan**: one Plan concept holding the ordered tasks, executed serially in one or a few sessions, deleted in the commit that completes the work. Choose **issues** when the work is parallelisable (multiple agents taking units concurrently), genuinely independent (no shared in-flight interfaces), or long-horizon (weeks, cold starts). Say which form you chose and why; the user confirms it in step 5.

### 4. Draft the breakdown

Break the work into **tracer bullet** slices.

<vertical-slice-rules>

- Each slice cuts a narrow but COMPLETE path through every layer (schema, API, UI, tests) — vertical, NOT a horizontal slice of one layer
- A completed slice is demoable or verifiable on its own
- Each slice is sized to fit in a single fresh context window
- Any prefactoring should be done first

</vertical-slice-rules>

For issues, give each its **blocking edges** — the issues that must complete before it can start. An issue with no blockers can start immediately. A plan's tasks are ordered instead; no edges needed.

**Wide refactors are the exception to vertical slicing.** A **wide refactor** is one mechanical change — rename a column, retype a shared symbol — whose **blast radius** fans across the whole codebase, so a single edit breaks thousands of call sites at once and no vertical slice can land green. Don't force it into a tracer bullet; sequence it as **expand–contract**. First expand: add the new form beside the old so nothing breaks. Then migrate the call sites over in batches sized by blast radius (per package, per directory), each batch its own unit blocked by the expand, keeping CI green batch to batch because the old form still exists. Finally contract: delete the old form once no caller remains, blocked by every migrate batch. When even the batches can't stay green alone, keep the sequence but let them share an integration branch that all block a final integrate-and-verify unit — green is promised only there.

### 5. Quiz the user

Present the proposed breakdown as a numbered list: for each unit its title, what it delivers end-to-end, and (issues) what blocks it. Ask:

- Is the form right — plan or issues?
- Does the granularity feel right? (too coarse / too fine)
- Are the blocking edges correct — does each issue only depend on issues that genuinely gate it?
- Should any units be merged or split further?

Iterate until the user approves the breakdown.

### 6. Publish to the bundle

- **Plan** → `knowledge/plans/Pnnn-<feature>.md` per [PLAN-FORMAT.md](./PLAN-FORMAT.md).
- **Issues** → one concept per issue at `knowledge/issues/Innn-<slug>.md` per [ISSUE-FORMAT.md](./ISSUE-FORMAT.md), published in dependency order (blockers first) so edges reference concepts that exist. Work the **frontier**: any issue whose blockers are all done.

Add each new concept to the bundle's `index.md`. The validator must pass: `superdev aokf validate knowledge` (in Claude Code the PostToolUse hook runs it for you).

In either form, avoid specific file paths or code snippets — they go stale fast. Exception: if a prototype produced a snippet that encodes a decision more precisely than prose can (state machine, reducer, schema, type shape), inline it and note briefly that it came from a prototype. Trim to the decision-rich parts.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.

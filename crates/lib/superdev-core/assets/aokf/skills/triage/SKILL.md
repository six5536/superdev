---
name: triage
description: Move issues in knowledge/issues/ through a state machine of triage roles — categorise, verify, grill if needed, and write agent-ready briefs.
disable-model-invocation: true
---

# Triage

Move the issues in `knowledge/issues/` through a small state machine of triage roles. An issue is an AOKF concept (the format is `/to-plan`'s ISSUE-FORMAT.md); triage roles are frontmatter tags. A repo that also takes external input — a remote tracker, external PRs — describes that surface in this skill's `PROJECT.md`.

## Reference docs

- [AGENT-BRIEF.md](AGENT-BRIEF.md) — how to write durable agent briefs

## Roles

Two **category** roles:

- `bug` — something is broken
- `enhancement` — new feature or improvement

Five **state** roles:

- `needs-triage` — maintainer needs to evaluate
- `needs-info` — waiting on the reporter for more information
- `ready-for-agent` — fully specified, ready for an AFK agent
- `ready-for-human` — needs human implementation
- `wontfix` — will not be actioned

Every triaged issue carries exactly one category tag and one state tag. If state tags conflict, flag it and ask the maintainer before doing anything else.

State transitions: an issue with no state tag normally goes to `needs-triage` first; from there it moves to `needs-info`, `ready-for-agent`, `ready-for-human`, or `wontfix`. `needs-info` returns to `needs-triage` once answers arrive. The maintainer can override at any time — flag transitions that look unusual and ask before proceeding.

## Invocation

The maintainer invokes `/triage` and describes what they want in natural language. Interpret the request and act. Examples:

- "Show me anything that needs my attention"
- "Let's look at the config-reload issue"
- "Move I003 to ready-for-agent"
- "What's ready for agents to pick up?"

## Show what needs attention

Scan `knowledge/issues/` and present three buckets, oldest first:

1. **No state tag** — never triaged.
2. **`needs-triage`** — evaluation in progress.
3. **`needs-info` where the concept changed after the triage notes were written** (git dates both) — needs re-evaluation.

Show counts and a one-line summary per item. Let the maintainer pick.

## Triage a specific issue

1. **Gather context.** Read the whole concept — body, tags, links, and any prior triage notes, so you don't re-ask resolved questions. Explore the codebase using the glossary concept's vocabulary, respecting Decision concepts and stable specs in the area. Run two checks: (a) **redundancy** — search for an existing implementation of the requested behaviour by domain concept (not just the request's wording), and report where you looked. If found, it's an already-implemented `wontfix` (step 5). (b) **prior rejection** — read the backlog concept's "Decided against" entries, any Decision concepts, and the `wontfix`-tagged issues, and surface any that resembles this request.

2. **Recommend.** Tell the maintainer your category and state recommendation with reasoning, plus a brief codebase summary relevant to the request — including whether it's already implemented. Wait for direction.

3. **Verify the claim.** Before any grilling, check that the claim holds up. For a bug, reproduce it from the reporter's steps. Report what happened: confirmed (with code path), failed, or insufficient detail (a strong `needs-info` signal). A confirmed verification makes a much stronger agent brief.

4. **Grill (if needed).** If the request needs fleshing out, run `/grill-me` — grill it into shape a round of questions at a time; terms and decisions land in the bundle as they crystallise.

5. **Apply the outcome:**
   - `ready-for-agent` — write the agent brief into the concept's body ([AGENT-BRIEF.md](AGENT-BRIEF.md)).
   - `ready-for-human` — same structure, but note why it can't be delegated (judgment calls, external access, design decisions, manual testing).
   - `needs-info` — add triage notes to the concept (template below).
   - `wontfix` — the issue keeps the tag and stays; search down-ranks it. Record the *why* in its body:
     - **Already implemented** — point to where it lives; no backlog entry (that record is for *rejected* requests, not built ones).
     - **Rejected (bug)** — the reasoning, briefly.
     - **Rejected (enhancement)** — the reasoning, plus an entry under the backlog concept's "Decided against" (or a Decision concept when the reasoning is load-bearing).
   - `needs-triage` — apply the tag. Optional notes if there's partial progress.

## Quick state override

If the maintainer says "move I003 to ready-for-agent", trust them and apply the tag directly. Confirm what you're about to do (tag changes, notes), then act. Skip grilling. If moving to `ready-for-agent` without a grilling session, ask whether they want an agent brief written.

## Needs-info template

Add to the concept's body:

```markdown
## Triage notes

**What we've established so far:**

- point 1
- point 2

**What we still need from the reporter:**

- question 1
- question 2
```

Capture everything resolved during grilling under "established so far" so the work isn't lost. Questions must be specific and actionable, not "please provide more info".

## Resuming a previous session

If the concept carries prior triage notes, read them, check whether the open questions have been answered since (the concept's git history dates both), and present an updated picture before continuing. Don't re-ask resolved questions.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.

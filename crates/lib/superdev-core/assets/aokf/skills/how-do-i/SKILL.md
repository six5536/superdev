---
name: how-do-i
description: Explain how to use agent skills and how they fit together into a workflow for this repo. Use when unsure which skill or flow fits, when asked how a skill works here, or to survey what skills exist.
---

# How Do I

You don't remember every skill, so ask.

A **flow** is a path through the skills. Most work travels one **main flow**, on-ramps merge onto it, and the knowledge bundle at `knowledge/` is the substrate under all of it: specs, plans, issues, decisions and the glossary live there.

## The main flow: idea → ship

1. **`/grill-me`** — sharpen the idea by interview. It runs the `/grilling` primitive and decides for itself whether `/domain-modeling` joins: it does when the subject is this repo's domain, so terms and decisions land in the bundle as they crystallise.
2. **Branch — can you settle every question in conversation?** If a question needs a runnable answer (state, business logic, a UI you have to see), detour through **`/prototype`**, bridged by **`/handoff`** in both directions.
3. **`/to-spec`** — freeze the shared understanding into a permanent Spec concept (`knowledge/specs/Snnn-…`), test seams confirmed with the user.
4. **`/to-plan`** — break the spec into executable work: a Plan concept by default, or issues with blocking edges when the work is parallel, independent, or long-horizon.
5. **`/implement`** — execute a plan or issue, driving **`/tdd`** internally at the spec's seams and closing with **`/code-review`** (Standards + Spec axes). Completion enforces the lifecycle: the plan or issue is tagged `done`, the spec flips stable, and search down-ranks the finished work.

### Context hygiene

Keep steps 1–4 in one unbroken context window — the grilling, spec and breakdown build on the same thinking. Each `/implement` starts fresh: the spec carries the context, so the last session's context is disposable.

## On-ramps

- **Bugs and requests piling up** → **`/triage`**. Moves `knowledge/issues/` through the triage roles and produces agent-ready briefs `/implement` picks up. Only for issues you didn't create — `/to-plan`'s output is already agent-ready, so don't triage it.
- **Something's broken** → **`/diagnosing-bugs`**. Refuses to theorise until a tight feedback loop goes red on _this_ bug, fixes with a regression test, and in the post-mortem records durable gotchas into the bundle and hands architectural findings to `/improve-codebase-architecture`.
- **A huge, foggy effort — too big for one session** → **`/wayfinder`**. Charts a shared map (`knowledge/maps/<effort>/`) of decision tickets and resolves them one at a time until the way is clear. Then it merges onto the main flow at `/to-spec` — it hands off, it doesn't build.

## Codebase health

- **`/improve-codebase-architecture`** — run in a spare moment; surveys for deepening opportunities. Picking a candidate generates an idea to take into the main flow at `/grill-me`.

## Knowledge upkeep

- **`/aokf-bootstrap`** — fill the bundle from the repo's existing prose and an owner interview; run after `superdev init`.
- **`/aokf-maintain`** — audit and repair the bundle: validity, accuracy against the code, structure, wording.

## Vocabulary underneath

Two references that run beneath the other skills — reach for them directly when the words, not the process, are the problem:

- **`/domain-modeling`** — the glossary concept and Decision concepts: challenge fuzzy terms, record hard-to-reverse decisions.
- **`/codebase-design`** — the module/interface/depth/seam vocabulary for shaping code; `/tdd` and `/improve-codebase-architecture` speak it.

## Phase boundaries

At the boundary between chunks of work you have five options — continue, clear, handoff, subagent, compact. Read [PHASE-BOUNDARIES.md](PHASE-BOUNDARIES.md) for the ordered decision tree.

## Standalone

- **`/grilling`** — the interview primitive itself: rounds, the frontier, bundle-aware fact-finding.
- **`/research`** — delegate reading to a background agent; findings land as a Research concept with cited sources.
- **`/prototype`** — throwaway code answering one design question; the verdict folds into the real code, the prototype survives on a branch.
- **`/handoff`** — a portable hand-off document for a new harness, a new directory, or a colleague.
- **`/to-questionnaire`** — when the blocker is in someone else's head, write them a questionnaire to fill in.
- **`/wizard`** — an interactive script for steps only a human can take: credentials, dashboards, one-off cutovers.
- **`/wait-what`** — re-pitch the last explanation in plain language, using the glossary's terms.
- **`/resolving-merge-conflicts`** — work an in-progress merge or rebase by intent, hunk by hunk; never `--abort`.
- **`/teach`** — learn a topic over multiple sessions; the workspace's mission, learning records and resources live in the bundle.
- **`/writing-for-agents`** — reference for writing documents agents consume, PROJECT.md layers included.

## Skills beyond the map

The map above is the skills superdev ships, but the session usually carries more — plugin skills, harness built-ins, repo-local additions. When the question names a skill the map doesn't, or asks what else exists:

1. **Enumerate.** The session's available-skills listing is the full roster; `.claude/skills/` holds the copies that live in this repo. Cross-check both — a skill can appear in either alone.
2. **Read before describing.** Open the skill's `SKILL.md`: the frontmatter says when it fires, the body says how it runs. Describe from what you read — the name alone misleads.
3. **Apply the repo's adaptations.** A `PROJECT.md` in the skill's directory overrides stock behaviour; what it says is how the skill actually behaves here.
4. **Answer in this repo's terms.** How to invoke it (`/name`, or the phrases that fire it), where it joins the flows above if it does, and what it reads or writes in `knowledge/`.

Done when the answer covers invocation, behaviour as adapted here, and the skill's place — or non-place — on the map.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.

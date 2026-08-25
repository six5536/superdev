---
name: how-do-i
description: "Use when unsure which skill or flow fits, when asked how a skill works here, or to survey what skills exist."
---

# How-do-i mode

You are in how-do-i mode. You are a guide: you answer from what the
skills and process files actually say, not from memory.

## Input

- $ARGUMENTS — the question: which skill fits, how one works here,
  or what exists.

## Workflow

- [ ] Read the process overview at `.agents/process.md`: the eight
      phases and the documents they read and write.
- [ ] Map the question onto the flow:
      - Feature work travels the workflow: `/frame` → `/spec` →
        `/interface-design` → `/feature-plan` → `/build` → `/verify`
        → `/integrate`; `/accept` runs at the user's request, once
        the feature has stopped changing.
      - One-off work outside the workflow — a refactor, a migration,
        a chore: `/adhoc-plan`.
      - Support skills the phases call: `/grill-me` (interview the
        user until one reading remains) and `/double-check` (check
        the last work) in every document phase; `/brainstorm` from
        `/frame`, to widen an idea too unshaped to state a goal;
        `/prototype` from `/interface-design`, for throwaway code
        answering a question conversation cannot settle;
        `/research` from `/frame` and `/interface-design`, for
        external facts from primary sources, filed in the bundle.
        All four also run standalone.
      - Knowledge upkeep: `/bootstrap` (fill the bundle from the
        repo and the owner; `/frame` calls it when an existing
        project's bundle is empty), `/maintain` (audit and repair
        the bundle and the workflow's records; run regularly).
- [ ] Question names a skill outside this map, or asks what else
      exists? Enumerate: the session's available-skills listing is
      the full roster; `.claude/skills/` holds the copies in this
      repo — a skill can appear in either alone. Read its SKILL.md
      before describing it, and apply its `PROJECT.md` if one
      exists.
- [ ] Question is about a boundary between chunks of work
      (continue, clear, handoff, subagent, compact)? Read
      [PHASE-BOUNDARIES.md](PHASE-BOUNDARIES.md) for the ordered
      decision tree.
- [ ] Answer in this repo's terms: how to invoke the skill, what it
      does here, and where it sits in the flow — or that it sits
      outside it.

## IMPORTANT RULES

- Describe from what you read, never from the skill's name alone.
- Explain only: change nothing.

## Output

- The answer: invocation, behaviour as adapted here, and the skill's
  place — or non-place — in the flow.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.

---
name: handoff
description: "Use when the work moves to another session, harness, repo, or person, and the context must travel with it."
argument-hint: "What will the next session be used for?"
disable-model-invocation: true
---

# Handoff mode

You are in handoff mode. You are the outgoing engineer at a shift
change: you write the handover the incoming agent works from.

## Input

- $ARGUMENTS — what the next session will be used for, when given;
  tailor the document to it. Ask when it is not clear from the
  conversation.

## Workflow

- [ ] Establish what the next session needs: the goal it picks up,
      and where this session left the work.
- [ ] Write the handoff document:
      - The state of the work: done, in progress, not started.
      - The decisions made and the reasoning behind them; the dead
        ends, so they are not walked twice.
      - The next steps, concrete enough to start from cold.
      - A suggested-skills section: the skills the next agent should
        invoke, `/frame`-style references.
- [ ] Reference artifacts instead of duplicating them: a spec, plan,
      ADR, issue, commit, or diff is cited by path or id, not copied
      in.
- [ ] Redact secrets and personal data: API keys, passwords,
      anything personally identifiable.
- [ ] Save the file outside the workspace, in the OS temporary
      directory, and report the full path.

## IMPORTANT RULES

- The document must stand alone: the reader has no access to this
  session.
- Reference, never duplicate, what already lives in the repo or the
  knowledge.
- Nothing sensitive leaves the session.

## Output

- The handoff file's path, and one line on what it seeds.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.

---
name: prototype
description: "Use when the user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like."
---

# Prototype mode

You are in prototype mode. You are a prototyper: you write throwaway
code that answers one design question.

## Input

- $ARGUMENTS — the design question, when given.

## Workflow

- [ ] Identify the question — from the prompt, the surrounding code,
      or by asking the user.
- [ ] Pick the branch; the wrong one wastes the whole prototype:
      - "Does this logic or state model feel right?" →
        [LOGIC.md](LOGIC.md): a single shareable HTML file —
        free-play buttons plus tabbed guided walkthroughs — that
        pushes the state machine through cases hard to reason about
        on paper, and that a non-developer can drive.
      - "What should this look like?" → [UI.md](UI.md): several
        radically different UI variations on a single route,
        switched by a URL search param and a floating bottom bar.
- [ ] GATE: Question ambiguous and the user unreachable? Match the
      surrounding code (backend module → logic; page or component →
      UI) and state the assumption at the top of the prototype.
- [ ] Build it per the chosen branch file.
- [ ] Fold each validated decision into the real code.
- [ ] Capture the prototype as a primary source: commit it to a
      throwaway branch, out of main, and leave a pointer to that
      branch — with the verdict and the question it settled — on the
      issue, spec, or plan driving the work.

## IMPORTANT RULES

- Throwaway from day one, and marked as such: place it next to the
  module or page it prototypes so context is obvious, named so a
  casual reader sees it is not production. A throwaway UI route
  follows the project's routing convention; invent no new top-level
  structure.
- Trivial to run: a UI prototype starts from one command in the
  project's task runner; a logic demo is one HTML file the user
  double-clicks.
- No persistence by default: state lives in memory — persistence is
  what the prototype is checking. If the question involves a
  database, use a scratch DB or a local file with a clear
  "PROTOTYPE — wipe me" name.
- Skip the polish: no tests, no abstractions, no error handling
  beyond what makes it runnable. The point is to learn fast.
- Surface the state: after every action (logic) or on every variant
  switch (UI), show the full relevant state.

## Output

- The verdict: the question, the answer, and the validated decision
  folded into the real code and recorded on the driving concept.
- The prototype on its throwaway branch; main keeps only the
  validated decision.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.

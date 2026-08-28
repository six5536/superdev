---
type: Template
id: template-spec
title: Spec Template
description: What done looks like from outside — observable behaviour, acceptance criteria, UI states, edge cases, and what is out of scope. Filed as a draft concept in knowledge/specs/, tagged done at accept.
status: stable
---

---
type: Spec
id: spec-<nnn>-<feature-slug>
title: <feature name>
description: <one line — what done looks like, from outside>.
status: draft
---

# Summary

<One or two sentences: what the feature does and for whom, as a user or caller would describe it. No implementation.>

# Behaviour

<The feature from outside — what a user sees or a caller gets, stated as observable facts. "When X, the system does Y." Each line something a tester could watch happen.>

- <Behaviour 1>
- <Behaviour 2>

# Acceptance criteria

<Numbered and walkable: each one checkable as pass/fail without interpretation. Given/When/Then where it helps.>

1. Given <starting state>, when <action>, then <observable result>.
2. Given <...>, when <...>, then <...>.
3. <...>

# UI states

<For UI features, the list of states is most of the spec. Delete this section for non-UI work.>

- Empty: <what shows before there is any data>
- Loading: <what shows while waiting>
- Populated: <the normal case>
- Error: <what the user sees and what they can do about it>
- Edge: <overflow, long text, many items, zero-width — whatever this UI can be fed>

# Edge cases & errors

<Inputs and situations that must be handled, with the expected behaviour for each — invalid input, limits, concurrency, offline, permission denied.>

- <Case> → <expected behaviour>

# Out of scope

- <Adjacent behaviour deliberately excluded, so nobody assumes it was forgotten.>

# Open questions

<Behavioural decisions still unmade, each with a recommended answer and who decides. Delete if none.>

- <Question> — recommendation: <default>

---

Notes on usage (not part of the document):

- File as `knowledge/specs/spec-<nnn>-<feature-slug>.md`, numbered after the
  highest existing spec; id `spec-<feature-slug>`.
- List it in `knowledge/specs/index.md`.
- The test plan (`template-test-plan`) is appended to this document as
  further sections.

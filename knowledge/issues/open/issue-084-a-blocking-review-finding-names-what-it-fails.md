---
type: Issue
id: issue-084-a-blocking-review-finding-names-what-it-fails
title: A blocking review finding names what it fails
description: The review prompts ban speculative improvements as a category the reviewer classifies itself into, so a reviewer convinced its finding matters never applies the rule; require every blocking finding to cite the objective it fails instead.
kind: feature
lifecycle: open
links:
  - rel: references
    to: idea-014-superdev-subprocesses-show-progress-and-control
    note: The finding that motivated this is the example that idea's motivation uses.
---

# Feature: a blocking review finding names what it fails

## Summary

The review prompts exclude speculative improvements, but they do so by naming
a category the reviewer must classify itself into. A reviewer convinced its
finding matters will not label that finding speculative, so the rule never
binds where it is needed. Require instead that every blocking finding cite the
objective it fails, which the reviewer cannot classify its way around.

## Context

The proposed wording is:

```text
Every blocking finding must identify a concrete failure of an existing
objective. A possible enhancement does not establish a defect.
```

Two of the three review prompts already carry a related line.
`requirements-review.md` ends its first paragraph with "Do not modify files or
emit stylistic preferences and speculative improvements", and
`code-review.md` says "Omit stylistic preferences and speculative
improvements". `accept.md` carries no equivalent exclusion at all.

The existing line is weaker than it appears. It bans a category, and the
reviewer decides which category its own finding belongs to. A reviewer that
believed a finding was speculative would not have raised it, so the rule is
addressed to a reviewer who does not exist.

The example in
[the subprocess-visibility idea][sokf:idea-014-superdev-subprocesses-show-progress-and-control]
shows the failure. The finding
`Post-persistence abort handling lacks complete contract and test coverage.`
does not read as a speculative improvement. It reads as a gap, and it passes
the current prompt unchanged. The proposed wording asks a different question
of it: which contract promise, and which test obligation. If neither can be
named, it is not blocking.

## Behaviour

Every blocking finding names the objective it fails and the concrete way it
fails it. A finding that cannot name one is not blocking.

The term needs enumerating in the prompt, so that "existing objective" means
the issue's stated outcome, the plan's declared requirements, or the
contracts' promises. Enumeration is what makes the rule enforceable rather
than merely stern. "Fails contract-012 `P_edit-parity`" is checkable by a
reader. "Fails the objective of robustness" is not. Without the enumeration
the reviewer asserts an objective; with it the reviewer cites a document.

The rule belongs in all three review prompts rather than only the later two.
The cycle this addresses is felt in SCOPE requirements review, so exempting
that phase would exempt the case that motivated the change. `accept.md` needs
it most of its three, because it carries no exclusion today and its findings
route back to BUILD or SCOPE, where each one costs a correction cycle.

Requirements review deserves care, because part of that role's job is finding
what a plan omits. Under a literal reading of "a concrete failure of an
existing objective", a reviewer might wave through a plan that contradicts
itself, or a contract silent on a case the issue implies. Both are legitimate
blocking findings. Both are nameable once the term is enumerated: an
inconsistent plan cannot be executed to reach the issue's stated outcome, and
a contract silent on an implied case will not deliver it. The risk is a
literal-minded reader, not a gap in the logic, and the enumeration is what
removes it.

The expected effect is narrower than it first appears, and worth stating
plainly. Both `mechanical` and `substantive` findings prevent a `clean`
result, so this rule does not route weak findings into a softer channel; it
raises the bar on what may block at all. Its main value is preventing a
mechanical finding from triggering an automatic correction cycle against a
non-defect, and reducing what interrupts the user.

## Scope

The wording of the review prompts.

- In: `requirements-review.md`, `code-review.md`, and `accept.md`, their
  `pack/` mirrors, and the lock hashes that `superdev sync` records for them.
- Out: contract-011. A promise about finding quality would fit `P_typed-role-result`,
  and the rest of the workflow does encode its rules there. It is deliberately
  not proposed yet: prompts are where wording is tried, contracts are where it
  is settled. This phrasing is unproven, and encoding it as a contract promise
  before a few reviews have exercised it makes revision expensive precisely
  when revision is likely.
- Out: the finding schema, the classification vocabulary, and the correction
  cycle mechanics.
- Out: `scope.md` and `build.md`, which author and implement rather than
  review.

## Comments

Filed from a live session, from a phrase the user proposed after watching a
requirements review run for eighteen minutes. The prompts were checked first:
the rule is not a duplicate of the existing exclusion, it is the enforceable
form of it.

Two positions were revised while discussing it, and both are recorded above
rather than lost. The enumeration was first argued as insurance against
suppressing omission findings; its real value is verifiability, which is a
stronger reason. A contract promise was first suggested as the idiomatic home;
prompts are the better first home for wording nobody has tested yet.

<!-- sokf:links -->
[sokf:idea-014-superdev-subprocesses-show-progress-and-control]: /knowledge/ideas/idea-014-superdev-subprocesses-show-progress-and-control.md

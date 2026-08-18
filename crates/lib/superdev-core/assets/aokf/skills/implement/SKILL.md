---
name: implement
description: Implement the work described by a Plan concept, an issue, or a spec from the knowledge bundle.
disable-model-invocation: true
---

Implement the work described by the user: a Plan concept, an issue, or
a Spec concept directly.

Read the spec the work implements (follow the plan's or issue's
`implements` link) before writing code. Use /tdd where possible, at the
seams the spec's Testing section pre-agreed.

Run typechecking regularly, single test files regularly, and the full
test suite once at the end.

Once done, use /code-review to review the work.

Commit your work to the current branch. Completing a plan deletes the
Plan concept and flips its spec to `status: stable` in the same commit;
completing an issue deletes the issue concept. Update the bundle's
`index.md` for anything removed.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.

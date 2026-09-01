---
type: Chore
id: issue-042-chore-audit-the-path-assertions-that-can-pass-by-accident
title: Audit the test assertions that check a path by substring and can pass by accident
description: I041's test asserted that a message contained a forward-slashed path, which an absolute path satisfies on every platform where the separator is a slash, so it passed on Linux and macOS for as long as it existed and could only ever fail on Windows; 58 substring path assertions share the shape and none has been examined.
lifecycle: open
---

# Chore: audit the path assertions that can pass by accident

## Summary

A test that asserts a message *contains* a forward-slashed path passes
whenever the real value merely ends with it — a different, absolute, or
differently separated spelling satisfies it. That is how
[I041][sokf:issue-041-bug-an-unreadable-named-path-is-reported-in-a-spelling-the-caller-never-typed]
went unnoticed: the assertion was true on Linux and macOS for as long as
the test existed, and the defect it was written to catch could only ever
surface on Windows. The same shape appears 58 times and none of the
others has been looked at.

## Surfaces

- 22 substring assertions carrying a path separator in the integration
  tests (`grep -rn "\.contains(" crates/*/*/tests/*.rs | grep -E
  'contains\("[^"]*/'`), of which 6 assert on a process's stdout or
  stderr rather than on file content superdev itself wrote.
- 36 of the same shape in the inline `#[cfg(test)]` tests
  (`grep -rn "\.contains(" crates/*/*/src --include=*.rs | grep -E
  'contains\("[^"]*/'`).
- 28 platform-gated tests, whose gates decide what Windows and macOS
  actually run (`grep -rn 'cfg(unix)\|cfg(windows)\|cfg(not(windows))'
  crates/*/*/src crates/*/*/tests --include=*.rs`). `tests/manage.rs`
  carries a whole-file `#![cfg(unix)]`.

Not every one is a fault. An assertion on a lock entry or a generated
link block checks content superdev writes forward-slashed by design, and
is right as it stands. The audit separates those from assertions on a
value the operating system or the caller supplied, which is where the
spelling can differ.

## Definition of done

- TBD — the audit settles whether each of the 58 is exact, legitimately
  a substring, or wrong; the count of each is the result.
- TBD — whether a rule follows, such as asserting the whole message for
  a single-line diagnostic, and where it is written down.
- TBD — whether the whole-file `#![cfg(unix)]` on `tests/manage.rs`
  hides Windows coverage worth having, which is a separate question the
  audit is well placed to answer.
- TBD — the command that says it is finished.

## Comments

Filed without framing, at the user's instruction, so the definition of
done is left as the questions the work must answer rather than as
answers invented ahead of it. Framing settles them at the point the
chore is taken up, which is what
[I030][sokf:issue-030-feature-request-filing-an-issue-requires-framing-it]
asks for.

Found while fixing
[I039][sokf:issue-039-bug-validate-fix-refuses-to-refile-under-a-symlinked-root]
and [I040][sokf:issue-040-bug-validate-reports-findings-on-a-windows-checkout-that-linux-does-not]:
those two had held the macOS and Windows CI jobs red for days, and the
Windows job aborted at test 12, so it never reached the assertion that
I041 turned out to be. One defect masking another is the reason this
audit is worth doing rather than trusting the suite.

<!-- sokf:links -->
[sokf:issue-030-feature-request-filing-an-issue-requires-framing-it]: /knowledge/issues/open/issue-030-feature-request-filing-an-issue-requires-framing-it.md
[sokf:issue-039-bug-validate-fix-refuses-to-refile-under-a-symlinked-root]: /knowledge/issues/done/issue-039-bug-validate-fix-refuses-to-refile-under-a-symlinked-root.md
[sokf:issue-040-bug-validate-reports-findings-on-a-windows-checkout-that-linux-does-not]: /knowledge/issues/done/issue-040-bug-validate-reports-findings-on-a-windows-checkout-that-linux-does-not.md
[sokf:issue-041-bug-an-unreadable-named-path-is-reported-in-a-spelling-the-caller-never-typed]: /knowledge/issues/done/issue-041-bug-an-unreadable-named-path-is-reported-in-a-spelling-the-caller-never-typed.md

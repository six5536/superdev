---
type: BugReport
id: issue-002-bug-no-time-bound-on-the-update-query
title: The default-source query has no time bound, so a black-holed network stalls update
description: update runs git ls-remote on every untargeted invocation and CommandRunner had no timeout, so a network that neither answered nor refused stalled the command until the OS gave up; fixed in P005 slices 4 and 5, which gave the spawn seam a five-second deadline.
lifecycle: done
links:
  - rel: references
    to: contract-007-interface-pack-resolution
---

# Bug: the default-source query has no time bound

## Decided

[ADR-015][sokf:adr-015-the-spawn-seam-carries-a-deadline].
`CommandRunner` gains an options form of `run` carrying a timeout and extra
environment, defaulted so every existing call site is unchanged and
`run_with` is the one required method. No dependency: a deadline over
`std::process` is a spawn, a reader thread per pipe and a poll that kills on
expiry.

Only what superdev does on its own initiative is bounded. The `ls-remote`
query takes a deadline of a few seconds; the clone does not, because the user
pinned the pack and asked for it, and a repository on a slow link is a
legitimately long wait. `GIT_TERMINAL_PROMPT=0` rides on the same change,
which is what made it need the seam.


## Summary

Against [C007][sokf:contract-007-interface-pack-resolution].

`superdev update` asks the default pack source for its newest release on every
untargeted run. `CommandRunner` spawns and waits with no deadline, so on a
network that neither answers nor refuses — a captive portal, a black-holing
proxy — `update` sits there until the OS abandons the connect. ADR-009 asks
this query to degrade to the binary's own pin rather than erroring; stalling
for minutes is neither.

## Environment

- Version/commit: 0.2.0 / slice 11 of P003 (`4ed647f`)
- Platform: all; the stall length is the OS connect timeout, around two
  minutes on Linux

## Steps to reproduce

1. `RS_c1` Put the machine behind a network that drops packets to `github.com`
   silently rather than refusing them.
2. `RS_c2` Run `superdev update`.

## Expected behaviour

The query gives up after a few seconds and the run reports
`could not reach it`, as it already does when `git` is absent or fails.

## Actual behaviour

`update` produces no output until the OS connect timeout expires, then
reports `could not reach it` and continues correctly.

## Root cause (if known)

`crates/lib/superdev-core/src/runner.rs:33` is a plain `Command::output()`.
There is no deadline anywhere on the process boundary, and
`crates/lib/superdev-core/src/pack/pin.rs:123` is the first caller that runs
unprompted on an everyday command rather than in response to an explicit
fetch. The same absence applies to the clone in `pack/fetch.rs`, which
predates this slice.

## Proposed fix / workaround

- Fix: give `CommandRunner` a deadline — either on the trait, or as a variant
  of `run` the network callers use. That is a change to the seam every
  component shares, so it belongs to interface-design rather than to a slice.
  `GIT_TERMINAL_PROMPT=0` on the same call is worth taking at the same time;
  it needs the same seam, since `run` passes no environment.
- Workaround: none needed — the run completes and degrades correctly, it is
  only slow.

## Regression risk

Every spawn in the product goes through `CommandRunner`, so a deadline added
there touches mise installs, git fetches and the codegraph index alike. A test
would fake a runner that blocks and assert the caller gives up.

## Comments

Filed out of P003 slice 11's verify. The review that raised it described the
call as hanging indefinitely; it does not. `Command::output()` gives the child
a null stdin, so git's terminal credential prompt gets EOF and fails fast, and
a dropped connect is bounded by the OS. Slow and unbounded by superdev, but
finite — which is why this is an issue rather than a blocker on that slice.

<!-- sokf:links -->
[sokf:adr-015-the-spawn-seam-carries-a-deadline]: /knowledge/adrs/active/adr-015-the-spawn-seam-carries-a-deadline.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md

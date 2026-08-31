---
type: Decision
id: adr-015-the-spawn-seam-carries-a-deadline
title: The Spawn Seam Carries a Deadline and an Environment
description: CommandRunner gains an options form of run carrying a timeout and extra environment, defaulted so every existing caller is untouched, and the query update makes unprompted is the first caller to set one.
lifecycle: active
links:
  - rel: references
    to: contract-007-interface-pack-resolution
  - rel: relates-to
    to: adr-009-update-queries-default-source
---

# ADR-015: The spawn seam carries a deadline and an environment

- Status: accepted
- Date: 2026-08-26
- Deciders: project owner

## Context

`CommandRunner` is the one process boundary in the codebase — [C007][sokf:contract-007-interface-pack-resolution] fetches
a pack through it — and it is a plain `Command::output()`. Nothing superdev spawns has a deadline, and `run`
passes no environment.

That was unremarkable while every spawn answered something the user had asked
for. [ADR-009][sokf:adr-009-update-queries-default-source] changed it: `update`
now runs `git ls-remote` against the default source on every untargeted
invocation, unprompted. On a network that drops packets rather than refusing
them — a captive portal, a black-holing proxy — that call sits silent for the
OS connect timeout, around two minutes on Linux, and then degrades correctly.
ADR-009 asks the query to fall back to the binary's own pin rather than
erroring; two silent minutes is neither erroring nor falling back
([I002][sokf:issue-002-bug-no-time-bound-on-the-update-query]).

It is bounded — `Command::output()` gives the child a null stdin, so git's
terminal prompt gets EOF and fails fast, and a dropped connect ends when the
OS gives up. Bounded by the OS, not by superdev, and not at a length anyone
would choose.

A deadline is a change to the seam every component shares, which is why it is
here rather than in a slice. `GIT_TERMINAL_PROMPT=0` wants the same seam and
the same decision: `run` passes no environment, so there is nowhere to put
it.

## Decision

`CommandRunner` gains an options form of `run`. Everything beyond the
argument vector goes in one struct, so the seam does not grow a method per
concern:

```rust
/// How a command is run: everything beyond the program and its arguments.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Kill the child and fail after this long. `None` waits as long as it
    /// takes, which is what a toolchain install needs.
    pub timeout: Option<Duration>,
    /// Extra environment for the child, over the inherited one.
    pub env: Vec<(String, String)>,
}

pub trait CommandRunner {
    /// Run `program args…` in `cwd`, capturing output.
    fn run(&self, program: &str, args: &[String], cwd: &Path) -> Result<Output> {
        self.run_with(program, args, cwd, &RunOptions::default())
    }

    /// The same, with a deadline and an environment.
    fn run_with(
        &self,
        program: &str,
        args: &[String],
        cwd: &Path,
        opts: &RunOptions,
    ) -> Result<Output>;
}
```

`run_with` is the one required method and `run` defaults onto it, so there is
a single implementation to get right and every existing call site is
unchanged. A deadline that expires is an `Error::Command` saying so — the
same shape as any other failed spawn, which is what lets the pin query treat
it as "could not reach it" without knowing why.

**Only what superdev does on its own initiative is bounded.** The `ls-remote`
query gets a deadline of a few seconds; the clone does not. A clone happens
because the user pinned a pack and asked for it, and a repository on a slow
link is a legitimately long wait that superdev has no business ending. The
same reasoning leaves the mise installs and the code-index download alone.
The seam supports a deadline everywhere, so a later caller that needs one
opts in.

Both network calls also set `GIT_TERMINAL_PROMPT=0`, which the same change
makes possible.

No dependency. A deadline over `std::process` is a spawn, a reader thread per
pipe, and a poll that kills on expiry — well under the bar the
[dependency policy][sokf:dependency-policy] sets for reaching outside the
standard library.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| An options struct on the seam, defaulted | One required method, one implementation; existing callers untouched; the next concern that needs the boundary has somewhere to go | A struct a caller has to construct, and two ways to call one seam |
| A second method, `run_timeout` | Smaller than a struct | The environment still has nowhere to go, so a third method follows, and then a fourth |
| Timeout inside pack's git wrapper | Touches nothing shared | Either bypasses the one spawn seam, which [the architectural rules][sokf:architectural-rules] forbid, or reimplements it beside itself |
| A crate — `wait-timeout`, or an async runtime | Somebody else's edge cases | A dependency for forty lines of thread code, against a policy that says reach for the standard library first |
| Deadline every spawn | One rule, no caller has to think | Ends a legitimately long clone or toolchain install, and turns a slow link into a failure the user cannot lengthen |
| Leave it | The run completes and degrades to the right answer; it is only slow | Silent for two minutes on a command the user did not ask to make a request at all |

## Consequences

- Positive: the one request superdev makes without being asked is bounded by
  superdev, at a length it chose.
- Positive: `GIT_TERMINAL_PROMPT=0` becomes expressible, so a credential
  prompt cannot be reached even where stdin is not null.
- Positive: a faked runner can now block deliberately, so the deadline is
  testable without a network.
- Negative: the seam has two calling forms, and a reader has to know that
  `run` is the defaulted one. The trait's own doc is where that is said.
- Negative: a deadline is a constant in the code, not a setting. Someone on a
  link slow enough to lose the query has no knob, and raising the constant is
  the fix. Deliberate: a knob here is a manifest key that would outlive the
  problem.
- Negative: killing a child is best effort. A process that ignores the signal
  outlives the deadline, and superdev reports the timeout rather than
  pretending otherwise.
- Follow-ups: [C007][sokf:contract-007-interface-pack-resolution] records the seam
  change, since resolution is its only caller today;
  [architecture][sokf:architecture] and
  [software-components][sokf:software-components] describe `CommandRunner`
  and gain the options form at integrate.

<!-- sokf:links -->
[sokf:adr-009-update-queries-default-source]: /knowledge/adrs/active/adr-009-update-queries-default-source.md
[sokf:architectural-rules]: /knowledge/architectural-rules.md
[sokf:architecture]: /knowledge/architecture.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
[sokf:dependency-policy]: /knowledge/dependency-policy.md
[sokf:issue-002-bug-no-time-bound-on-the-update-query]: /knowledge/issues/done/issue-002-bug-no-time-bound-on-the-update-query.md
[sokf:software-components]: /knowledge/software-components.md

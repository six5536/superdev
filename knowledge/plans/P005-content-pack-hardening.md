---
type: Plan
id: feature-plan-content-pack-hardening
title: Content pack hardening — feature plan
description: Deliver ADR-012 to ADR-016 in seven slices — refuse an unsupported transport, refuse a symlink in a pack and let git decide what one is, give the spawn seam a deadline and an environment, bound the one unprompted request, prove a pin before writing it, and stop recording a digest nothing reads.
status: draft
links:
  - rel: references
    to: spec-content-packs
---

# Feature plan: content pack hardening

The five decisions taken against the issues [S014](../specs/S014-content-packs-design.md)'s
acceptance left open. Contract: [C001](../contracts/C001-content-packs.md).
Decisions: [ADR-012](../decisions/D012-pack-source-schemes-are-allowlisted.md),
[ADR-013](../decisions/D013-update-proves-a-pin-before-it-writes-it.md),
[ADR-014](../decisions/D014-a-symlink-in-a-pack-is-refused.md),
[ADR-015](../decisions/D015-the-spawn-seam-carries-a-deadline.md),
[ADR-016](../decisions/D016-a-path-pack-records-no-digest.md).

Its own plan rather than a reopening of
[P003](P003-content-packs.md), which is done and whose spec is accepted. P003's
*Not scheduled* list is what became this one, less
[I003](../issues/I003-a-local-pack-cannot-remove-what-it-dropped.md), closed
`wontfix`.

Ordered by dependency, then risk. Slice 1 is the half of the security issue
still open and carries the risk of refusing a spelling somebody legitimately
uses, so it goes first. Slices 2 and 3 narrow what a pack may contain, which
is the other way this feature can break something that works today. Slice 4
changes the seam every spawn in the codebase shares and observably does
nothing, which is what makes it safe to put in the middle.

S014's test-plan cases are all assigned in P003 and none is left over; every
case below is new and drawn from the issue it closes.

## Slices

### Slice 1: Refuse a transport superdev does not fetch over

- [x] Done — ticked by integrate at merge.
- Gap: [I007](../issues/I007-a-pack-source-reaches-git-with-no-scheme-check.md),
  the half left after slice 15 of P003.
- Change: add `SUPPORTED_SCHEMES` to `pack/source.rs`. `PackSource::parse`
  refuses a `<name>::<address>` remote helper first — before its address is
  examined at all, so `ext::https://…` is a helper and not an https source —
  then a scheme outside the set, naming the source and the transport.
  `fetch::overrides()` gains `protocol.allow=never`,
  `protocol.{https,ssh,file}.allow=always` and
  `protocol.{git,http,ext}.allow=never`.
  Correct `README.md`, which today says every git URL other than the shorthand
  "is handed to `git` as written" — [P003](P003-content-packs.md)'s slice 18
  wrote that knowing this slice would narrow it, and named this slice as the
  one to fix it.
- `file://` is not optional in the set: every git-source fixture in the suite
  spells its local repository that way, so dropping it fails every pack test
  that touches git rather than a documented use nobody exercises.
- Done-check: `git://`, `http://` and `ext::` sources are refused at parse,
  naming the transport, with no command run; `github:o/r`, `gitlab:o/r`,
  `https://`, `ssh://`, the scp form, a bare ssh alias and `file://` all still
  parse, and every `only_the_shorthand_is_expanded_for_git` case still passes;
  a test reads the override set off a real call and asserts all seven
  settings; `README.md` names the transports superdev accepts and no longer
  claims any other URL is passed through.
- Cases: new. **The one that matters is the `insteadOf` regression**: with a
  global config rewriting `https://…` into `ext::touch <path>`, a source
  `parse` approves must still run no command. That is the case that proves the
  override half is not decoration — `parse` cannot see a rewrite, and only the
  *named* refusal outranks a user config that says `protocol.ext.allow = always`
  ([ADR-012](../decisions/D012-pack-source-schemes-are-allowlisted.md) carries
  the measurements). `GIT_CONFIG_GLOBAL` points git at the fixture config; the
  spawned git inherits it.

### Slice 2: A symlink in a pack is refused, not skipped

- [x] Done — ticked by integrate at merge.
- Gap: [I009](../issues/I009-a-skipped-symlink-says-nothing.md), the half a
  filesystem check can reach.
- Change: `read_dir` and `read_pack` in `pack/resolve.rs` refuse a symlink
  naming the path instead of continuing past it, so the pack root and
  `pack.toml` refusals stop being special cases of a rule the rest of the tree
  does not follow.
- Done-check: a path pack containing a symlink fails the run naming the path,
  and so does a fetched pack read back from the cache; the
  [I008](../issues/I008-a-symlinked-file-in-a-pack-is-followed.md) regression
  — a link to a secret outside the pack — still writes nothing;
  superdev's own `pack/` still resolves, and a test asserts it contains no
  symlink so the day one appears is the day that test fails rather than the
  day a release does; `README.md`'s packs section says a pack may not contain
  one, since a pack author reads that before they read an error.
- Cases: new.

### Slice 3: Git decides what a symlink is

- [x] Done — ticked by integrate at merge.
- Gap: [I009](../issues/I009-a-skipped-symlink-says-nothing.md), the
  cross-platform half.
- Change: after the checkout and before anything is read or digested, `fetch`
  asks git for the pack subtree's index entries and refuses mode `120000` and
  mode `160000` — a submodule, which a shallow sparse clone leaves empty —
  naming the path. Through `fetch::git`, like every other git call.
- Done-check: a fixture repository whose pack holds a symlink is refused at
  fetch, before a digest is computed, **both** when the working tree holds a
  real link and when it holds the plain file a `core.symlinks=false` checkout
  produces; a submodule under the pack is refused the same way; a pack with
  neither resolves exactly as it does today, at the cost of one git call.
- Cases: new. The plain-file case is the point of the slice and is reproducible
  on Linux, so it belongs in the ordinary suite rather than the Windows job:
  set `core.symlinks=false` **in the fixture repository's own config and force
  a re-checkout** — passing `-c core.symlinks=false` to `clone` does not work,
  because clone probes the filesystem and writes its own value.

### Slice 4: The spawn seam carries a deadline and an environment

- [x] Done — ticked by integrate at merge.
- Gap: [I002](../issues/I002-no-time-bound-on-the-update-query.md), the seam
  half.
- Change: add `RunOptions { timeout, env }` to `runner.rs`. `run_with` becomes
  the trait's one required method and `run` defaults onto it, so every existing
  call site is untouched. `SystemRunner` implements it over `std::process` — a
  reader thread per pipe and a kill on expiry, no new dependency.
  `FakeRunner` records what it was given. Nothing sets an option yet.
- Done-check: the whole existing suite is green and unchanged; a child that
  blocks past its deadline yields `Error::Command` naming the timeout and is
  no longer running when the call returns; `timeout: None` waits; an `env`
  entry reaches the child; `&dyn CommandRunner` still compiles, which is the
  object-safety check.
- Cases: none — new code behind no caller, covered by its own unit tests.

### Slice 5: The one unprompted request is bounded, and never prompts

- [x] Done — ticked by integrate at merge.
- Gap: [I002](../issues/I002-no-time-bound-on-the-update-query.md).
- Change: `fetch::git` takes the options through. The `ls-remote` query sets a
  deadline of a few seconds; the clone sets none, because the user pinned the
  pack and asked for it. Both set `GIT_TERMINAL_PROMPT=0`. `README.md` today
  tells the reader to "expect it to wait for your OS to give up first", which
  this slice is what makes untrue.
- Done-check: a fake runner that blocks past the deadline makes `update`
  report `could not reach it` and carry on, and the command returns in about
  the deadline rather than the OS connect timeout; the clone carries no
  deadline; every git call carries `GIT_TERMINAL_PROMPT=0`; `README.md` no
  longer promises the OS-length stall.
- Cases: new.

### Slice 6: `update` proves a pin before it writes it

- [ ] Not started.
- Gap: [I001](../issues/I001-update-can-pin-an-unreadable-pack-format.md).
- Change: `update_pins` takes `&Lock` and resolves the entry it is about to
  move before writing it. On a refusal the pin stays where it is and the reason
  is reported in the line that would have announced the move. `manage.rs`
  passes the lock it already loads. `update` promises less than it did on
  every surface that states it — clap's `long_about` in `main.rs`, which
  renders into `--help`, the man page and the completions, the rustdoc in
  `manage.rs`, and `README.md` — each of which says the pin moves to the
  newest release without saying "that this binary can read".
- Done-check: against a fixture source whose newest release declares an
  unsupported format, `.superdev/config.toml` is unchanged, the refusal names
  the format, and the run still succeeds; a readable release still moves the
  pin; a second unresolvable pack entry does not hold back a move that is
  fine; an unreachable source behaves exactly as it does today; `--help`, the
  man page and `README.md` agree on what `update` now guarantees, and the help
  table stays inside 80 columns.
- Cases: new. The regression that must not break is P003's own — 16, 17, 20
  and 22 cover `update`'s pin movement and all four still pass unchanged.

### Slice 7: A path pack records no digest

- [ ] Not started.
- Gap: [I004](../issues/I004-a-path-packs-digest-churns-and-is-never-checked.md).
- Change: `PackLock.digest` becomes `Option<String>`, omitted for a path
  source; `resolve_one`'s path arm records none and the git arm's three
  readers take it by `as_deref`. Re-run `sync` so this repository's own
  committed lock drops the line.
- Done-check: `.superdev/lock.toml`'s `./pack` entry carries no `digest`; a
  lock written before this still parses and loses only that field; editing a
  file under `pack/` and syncing leaves the `[[packs]]` block byte-identical,
  which is the churn this closes; a git pack still verifies and still fails the
  run on a mismatch.
- Cases: new.

## Not scheduled

- [I003](../issues/I003-a-local-pack-cannot-remove-what-it-dropped.md) is
  closed `wontfix`: a path pack keeps layering, and the rebuild a pack
  developer needs anyway is the answer.

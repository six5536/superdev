---
type: InterfaceContract
id: contract-009-interface-run-state
title: Run State Interface Contract
description: The interface between the unattended loop's skill and its Stop hook — the run-state file, the verbs that write it, the hook's decision table, and the managed hook entry that arms it.
lifecycle: active
resource: /crates/app/superdev/src
links:
  - rel: references
    to: adr-018-loop-in-the-skill-enforcement-in-the-hook
    note: The skill/hook split this interface carries.
  - rel: references
    to: adr-019-run-state-is-a-session-owned-file-behind-cli-verbs
    note: The state file, its verbs, and the hook-owned counter.
---

# Interface contract: run state

The seam between the unattended loop's driver (a knowledge-carried
skill) and the managed Stop hook that enforces it. The skill decides
what runs next and records it through CLI verbs; the hook reads that
record and decides only whether a turn may end. The split is
[ADR-018][sokf:adr-018-loop-in-the-skill-enforcement-in-the-hook], the
state's ownership rules are
[ADR-019][sokf:adr-019-run-state-is-a-session-owned-file-behind-cli-verbs],
and a blocked run's lifecycle is
[ADR-020][sokf:adr-020-a-blocked-run-ends]. The verbs' user-facing
surface is in [contract-002][sokf:contract-002-cli-superdev].

## Data model & API

```rust
/// `.superdev/cache/run.toml` — present exactly while a run is active.
/// An absent file means no run: the hook lets every turn end.
pub struct RunState {
    /// The session that owns the run. The hook ignores Stop payloads
    /// from any other session. Recorded by `begin` from `--session` or
    /// the `CLAUDE_SESSION_ID` environment variable; when neither
    /// exists, empty — and the hook adopts the first Stop payload's
    /// `session_id` as owner, after which ownership binds. `advance`
    /// refreshes the owner the same way, so a resumed session does not
    /// orphan its own run.
    pub session_id: String,
    /// What the driver does next, in prose. The hook's exit-2 message
    /// names it. Empty means nothing to continue: the turn ends.
    pub next: String,
    /// Turn boundaries crossed since the last `advance`. Hook-owned:
    /// only `superdev hook run` increments it; only `advance` resets it.
    pub continues: u32,
    /// When the run began, ISO 8601. Informational.
    pub started: String,
    /// Pid of the `begin` process. Informational, for diagnosing a
    /// stale state by hand.
    pub pid: u32,
}

/// The watchdog cap (ADR-019): at this many continues without an
/// `advance`, the hook stops continuing the run.
pub const CONTINUE_CAP: u32 = 10;
```

## Module boundaries

- The `superdev` binary owns every read and write of `run.toml` — a
  `run` module beside the other verb modules; `superdev-core` never
  touches run state.
- `superdev hook run` is the one reader that also writes: it increments
  `continues` (and may adopt an empty owner); it changes nothing else
  and never opens a plan.
- `components/sokf.rs` declares the `hooks.Stop` entry in
  `.claude/settings.json` as a `ManagedItem::JsonEntry` with marker
  `superdev hook run`, beside the PostToolUse entry; it ships with the
  knowledge capability and is claimed in the lock the same way.
- The driver skill calls the verbs and never writes the file itself:
  exclusive creation is what makes a second run a refusal instead of a
  race.

## Key flows

- A run: `superdev run begin` → per slice, build → integrate, with
  `superdev run advance --next <TEXT>` at every real step forward →
  `superdev run end` when no slice is ready.
- A turn ends: Claude Code fires Stop; `superdev hook run` reads the
  payload and the state, and exits `0` when the state is absent, the
  payload's session is not the owner, `next` is empty, or `continues`
  has reached `CONTINUE_CAP` — otherwise it increments `continues` and
  exits `2` naming `next`, which Claude Code feeds back as the
  instruction to keep going. A payload without a session id matches
  nothing — it neither adopts nor drives a run — so an unclaimed run is
  driven only after adoption. The payload's `stop_hook_active` never
  gates the decision: exit `2` stays effective while it is true, and
  the counter is the guard
  ([research-001][sokf:research-001-claude-code-stop-hook-behaviour]).
- A second `begin` while state exists: refused, naming the owning
  session and `superdev run end` as the way to clear a stale run.

## Cross-cutting concerns

- Security: the state lives under the gitignored `.superdev/cache/`;
  the hook executes nothing and parses only its own TOML and the
  payload JSON. An unreadable payload is a loud exit `2`, matching
  `hook validate`; an unreadable `run.toml` is reported to stderr and
  exits `0` — failing open, because a hook that fails closed holds
  every session in the repo open.
- Performance: one small file read per Stop event; no network, no plan
  parse, no index.
- Migration/rollout: additive. Without a `run.toml` every path exits
  `0`, so existing repos see no behaviour change; the Stop entry
  arrives with the knowledge capability's sync, and `--no-knowledge`
  never gets it.
- Observability: `begin`, `advance` and `end` print the transition;
  the refusal names the owner; the exit-2 message names `next`; `end`
  with no state is harmless and says so.
- Platform interaction: any tool use in a continued turn resets Claude
  Code's eight-consecutive-block override, so the override never ends a
  productive run, and a text-only stall dies at eight before
  `CONTINUE_CAP` fires; no `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP` entry
  ships ([research-001][sokf:research-001-claude-code-stop-hook-behaviour]).

<!-- sokf:links -->
[sokf:adr-018-loop-in-the-skill-enforcement-in-the-hook]: /knowledge/adrs/active/adr-018-loop-in-the-skill-enforcement-in-the-hook.md
[sokf:adr-019-run-state-is-a-session-owned-file-behind-cli-verbs]: /knowledge/adrs/active/adr-019-run-state-is-a-session-owned-file-behind-cli-verbs.md
[sokf:adr-020-a-blocked-run-ends]: /knowledge/adrs/active/adr-020-a-blocked-run-ends.md
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:research-001-claude-code-stop-hook-behaviour]: /knowledge/research/research-001-claude-code-stop-hook-behaviour.md

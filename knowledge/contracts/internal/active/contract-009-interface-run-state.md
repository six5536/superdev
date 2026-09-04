---
type: Contract
id: contract-009-interface-run-state
kind: interface
title: Interface contract for run state
description: The interface between the unattended loop's skill and its Stop hook — the run-state file, the verbs that write it, the hook's decision table, and the managed hook entry that arms it.
lifecycle: active
resource: /crates/app/superdev/src/run.rs
links:
  - rel: references
    to: adr-018-loop-in-the-skill-enforcement-in-the-hook
    note: The skill/hook split this interface carries.
  - rel: references
    to: adr-019-run-state-is-a-session-owned-file-behind-cli-verbs
    note: The state file, its verbs, and the hook-owned counter.
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: The definition is materialised from the `run-state` region and bound by the include; the hook's decision table is bound by `run.rs`'s tests.
---

# Interface contract: run state

The seam between the unattended loop's driver (a knowledge-carried
skill) and the managed Stop hook that enforces it. The skill decides
what runs next and records it through CLI verbs; the hook reads that
record and decides only whether a turn may end. The Definition is the
two state files as `run.rs` declares them — where each lives, the two
caps, and the structs the verbs write and the hook reads. The split is
[ADR-018][sokf:adr-018-loop-in-the-skill-enforcement-in-the-hook], the
state's ownership rules are
[ADR-019][sokf:adr-019-run-state-is-a-session-owned-file-behind-cli-verbs],
a blocked run's lifecycle is
[ADR-020][sokf:adr-020-a-blocked-run-ends], and the materialised
definition is
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source].
The verbs' user-facing surface is in
[contract-002][sokf:contract-002-cli-superdev].

## Definition

<!-- sokf:include /crates/app/superdev/src/run.rs#run-state -->
```rust
/// Where the state lives, relative to the repo root. It is machine state:
/// `.superdev/cache/` is gitignored by `init`.
const RUN_STATE_PATH: &str = ".superdev/cache/run.toml";

/// The watchdog cap (ADR-019): at this many continues without an `advance`,
/// the hook stops continuing the run.
pub const CONTINUE_CAP: u32 = 10;

/// The hold cap (ADR-039): at this many turns held for the same unresolved
/// knowledge, the hook reports and lets the turn end, so a finding the agent
/// cannot settle stalls nothing.
pub const HOLD_CAP: u32 = 3;

/// The run state, present exactly while a run is active. An absent file
/// means no run: the hook lets every turn end.
#[derive(Debug, Serialize, Deserialize)]
pub struct RunState {
    /// The session that owns the run; the hook ignores Stop payloads from
    /// any other session. Recorded by `begin` from `--session` or the
    /// `CLAUDE_SESSION_ID` environment variable; when neither exists, empty,
    /// and the hook adopts the first Stop payload's `session_id` as owner,
    /// after which ownership binds. `advance` refreshes the owner the same
    /// way, so a resumed session does not orphan its own run.
    pub session_id: String,
    /// What the driver does next, in prose; the Stop hook's exit-2 message
    /// names it. Empty means nothing to continue: the turn ends.
    pub next: String,
    /// Turn boundaries crossed since the last `advance`. Hook-owned: only
    /// `superdev hook run` increments it; only `advance` resets it.
    pub continues: u32,
    /// When the run began (ISO 8601). Informational.
    pub started: String,
    /// Pid of the `begin` process. Informational, for diagnosing a stale
    /// state by hand.
    pub pid: u32,
}

/// Where the hold count lives, relative to the repo root. Separate from the
/// run state because a hold happens whether or not a run is armed, and
/// `run.toml`'s presence means a run is active: a hook that created one to
/// count holds would make the next `superdev run begin` refuse (ADR-039).
const HOLD_STATE_PATH: &str = ".superdev/cache/hold.toml";

/// What the Stop hook has held open for one session. An absent file means
/// nothing is being held.
#[derive(Debug, Serialize, Deserialize)]
pub struct HoldState {
    /// The session the count belongs to. A payload from another session
    /// starts the count again.
    pub session_id: String,
    /// Turns held open because the knowledge carried an error. Hook-owned:
    /// `superdev hook run` increments it while it holds and removes the file
    /// once the knowledge is clean.
    pub holds: u32,
}
```
<!-- /sokf:include -->

## Behaviour

### Module boundaries

The `superdev` binary owns every read and write of `run.toml` — a `run`
module beside the other verb modules. `superdev hook run` is the one
reader that also writes: it increments `continues` and adopts an empty
owner. The binary owns `hold.toml` the same way, and `superdev hook
run` is its only writer: it increments `holds` while it holds the turn
and removes the file once the knowledge is clean. `components/sokf.rs`
declares the `hooks.Stop` entry in `.claude/settings.json` as a
`ManagedItem::JsonEntry` with marker `superdev hook run`, beside the
PostToolUse entry; it ships with the knowledge capability and is
claimed in the lock the same way. The driver skill calls the verbs
([ADR-019][sokf:adr-019-run-state-is-a-session-owned-file-behind-cli-verbs]).

- `P_core-touches-no-run-state` [ubiquitous] `superdev-core` SHALL NOT
  touch run state.
- `P_hook-writes-continues-only` [ubiquitous] `superdev hook run` SHALL
  NOT change anything beyond `continues` and an empty owner, or open a
  plan.
- `P_hold-leaves-run-toml` [ubiquitous] A hold SHALL NOT create, alter
  or remove `run.toml` (ADR-039).
- `P_skill-writes-no-state` [ubiquitous] The driver skill SHALL NOT
  write `run.toml` itself (ADR-019).

### Key flows

1. A run: `superdev run begin` → `/build` per work block, with
   `superdev run advance --next <TEXT>` at every real step forward →
   `superdev run end` when no block is ready.
2. A turn ends: Claude Code fires Stop; `superdev hook run` reads the
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
3. A second `begin` while state exists: refused, naming the owning
   session and `superdev run end` as the way to clear a stale run.

### Cross-cutting concerns

Security: the state lives under the gitignored `.superdev/cache/`.

- `P_hook-executes-nothing` [ubiquitous] `superdev hook run` SHALL
  execute nothing and parse only its own TOML and the payload JSON.
- `P_unreadable-payload-exits-2` [event] WHEN the Stop payload is
  unreadable, `superdev hook run` SHALL exit `2` loudly, matching
  `hook validate`.
- `P_unreadable-state-fails-open` [event] WHEN `run.toml` is
  unreadable, `superdev hook run` SHALL report to stderr and exit `0`,
  failing open
  ([ADR-019][sokf:adr-019-run-state-is-a-session-owned-file-behind-cli-verbs]).

Performance: one small file read per Stop event; no network, no plan
parse, no index.

Migration/rollout: additive. Without a `run.toml` every path exits `0`,
so existing repos see no behaviour change; the Stop entry arrives with
the knowledge capability's sync, and `--no-knowledge` never gets it.

Observability: `begin`, `advance` and `end` print the transition; the
refusal names the owner; the exit-2 message names `next`; `end` with
no state is harmless and says so.

Platform interaction: any tool use in a continued turn resets Claude
Code's eight-consecutive-block override, so the override never ends a
productive run, and a text-only stall dies at eight before
`CONTINUE_CAP` fires; no `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP` entry ships
([research-001][sokf:research-001-claude-code-stop-hook-behaviour]).

## Stability

Internal.

- `P_internal` [ubiquitous] Every item above MAY change with the crate.

<!-- sokf:links -->
[sokf:adr-018-loop-in-the-skill-enforcement-in-the-hook]: /knowledge/adrs/active/adr-018-loop-in-the-skill-enforcement-in-the-hook.md
[sokf:adr-019-run-state-is-a-session-owned-file-behind-cli-verbs]: /knowledge/adrs/active/adr-019-run-state-is-a-session-owned-file-behind-cli-verbs.md
[sokf:adr-020-a-blocked-run-ends]: /knowledge/adrs/active/adr-020-a-blocked-run-ends.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:research-001-claude-code-stop-hook-behaviour]: /knowledge/research/research-001-claude-code-stop-hook-behaviour.md

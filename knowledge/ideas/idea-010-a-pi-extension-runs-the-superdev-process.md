---
type: Idea
id: idea-010-a-pi-extension-runs-the-superdev-process
title: A Pi extension runs the superdev process
description: A thin Pi orchestration extension uses Pi's lifecycle, tools, sessions, interaction and child agents to run superdev's canonical FILE → SCOPE → BUILD → ACCEPT workflow without reimplementing it.
status: draft
---

# Idea: a Pi extension runs the superdev process

Build a thin Pi orchestration extension for the complete superdev process. The
canonical skills continue to own FILE → SCOPE → BUILD → ACCEPT; the extension
adapts Pi's lifecycle, tools, sessions and UI to those skills. `superdev`
continues to own canonical knowledge, validation, document formats and durable
unattended-run state.

## Motivation

Pi already gives superdev SOKF-aware `read`, `edit`, `write`, `sokf_search` and
`sokf_graph` tools, but it does not run the development workflow. The current
workflow materialisation targets Claude Code: its skills live under
`.claude/skills/`, its validation and continuation behavior use Claude hooks,
and several skills name Claude-facing tools or slash commands. Loading that
prose in Pi without an adapter leaves phase entry, sub-skill calls, real user
interaction, unattended continuation, context isolation and hard safety gates
to model discipline.

Pi exposes the missing harness mechanisms directly: extension commands and
tools, resource discovery, lifecycle events, session entries, context hooks,
user dialogs, active-tool control, child Pi processes, progress rendering and
compaction hooks. Use those mechanisms to improve process reliability without
moving workflow semantics out of the knowledge-carried skills.

## Sketch

**Control graph.** First resolve
[issue-057][sokf:issue-057-the-skills-disagree-on-who-loops-and-who-returns],
because the adapter must not encode a contradictory graph. Follow
[ADR-050][sokf:adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept]:
`/contract-design` returns to its `/scope` caller; `/build` alone owns the
complete work-block loop, retries, final verification, merge and plan closure;
and `/execute-plan` invokes one complete `/build` under unattended
continuation. Make FILE appear consistently in the standing workflow text.
When no interactive user exists, a user-only question becomes a Deferred
decision rather than an answer supplied by the model.

**Extension shell.** Prove the design as a project extension under
`.pi/extensions/superdev/`, split into an entry point, commands, workflow state,
continuation, interaction and rendering modules. Register `/file`, `/scope`,
`/build`, `/execute-plan`, `/accept` and `/superdev-status` with
`pi.registerCommand()`. Register an optional unattended-start flag with
`pi.registerFlag()`. Use `resources_discover` to expose the canonical workflow
skill directory to Pi instead of copying the workflow prose. Command handlers
enter a skill through `sendUserMessage()` with prompt expansion enabled. On
`session_start`, find the repository root, verify that `superdev` is available,
restore presentation state and reconcile the UI. On `session_shutdown`, close
resources and warn when the session still owns an unattended run.

**Nested skills.** Register a `superdev_skill` tool that loads a named superdev
skill on demand, supplies its input and records its call stack in tool-result
details. The tool maps harness vocabulary such as `read_file`, `sokf_read` and
symbolic `skill_call` instructions to Pi's available tools. It rejects a
recursive call instead of allowing a cycle. Root phase commands use Pi's skill
expansion; nested workflow calls use this explicit tool, so the model receives
the next skill at the point where it is needed.

**Interaction.** Register a sequential `superdev_question` tool around
`ctx.ui.select()`, `ctx.ui.input()` and `ctx.ui.editor()`. The tool works in TUI
and RPC modes when `ctx.hasUI` is true. In print and JSON modes it reports that
interaction is unavailable, which directs `/scope` or `/contract-design` to
record a Deferred decision. It asks one decision at a time and carries the
recommended answer, matching `/grill-me`, rather than presenting a batch that
hides dependencies between decisions.

**Unattended continuation.** Use Pi's `agent_settled` event as the point
corresponding to Claude Code's Stop event. Invoke `superdev hook run` without a
shell and send a synthetic JSON payload containing Pi's session id on stdin.
Exit 2 means the run or knowledge hold continues: truncate the stderr report
and queue it with `pi.sendMessage()` as a triggered follow-up. Exit 0 leaves Pi
idle. Never parse or write `.superdev/cache/run.toml` in TypeScript; the CLI
remains the authority over exclusive ownership, the next step, knowledge holds
and watchdog counters. Pass `PI_SESSION_ID` explicitly to `run begin` and `run
advance`, because Pi exposes that variable to shell tools while the existing
verbs otherwise default to Claude's session variable.

Suppress continuation inside child Pi processes so a build worker cannot start
a second driver. Use `session_before_switch` and `session_before_fork` to warn
or require confirmation while the current session owns a run. Restore state
from the active session branch rather than every historical entry, and let the
binary reject another session that tries to advance the run.

**Isolated agents.** Add a constrained delegation tool based on Pi's subagent
pattern. Use one isolated worker for a complete `/build`, one read-oriented
researcher for `/research`, and one read-only reviewer for acceptance. Spawn
children with JSON output, print mode and no child session; inherit the active
model and thinking level; use the repository as the working directory; pass
the parent run id through the environment; stream concise progress; propagate
cancellation; truncate model-visible output; and include nested usage in the
parent tool result. Never run modifying workers in parallel against one
working tree. The parent retains only the plan, run progress and deferred
queue, while implementation details stay in the worker context.

**Phase context and enforcement.** Use `before_agent_start` to inject only the
current phase, plan, next required action, legal transitions, deferred
decisions and unattended branch rule. Keep format-sensitive instructions in
the skills. Use `tool_call` gates for boundaries that must survive model drift:
refuse unattended mutations on the default branch; prevent acceptance from
editing product code; keep file's writes to issue or idea records; and require
explicit approval before scope commits contract changes. Detect required
capabilities from `pi.getAllTools()` and `pi.getCommands()`. Name a missing
codegraph, prototype, rendered-UI, review or security capability instead of
silently omitting its step. Change active tools conservatively with
`getActiveTools()` and `setActiveTools()`, preserving tools registered by other
extensions.

**Compaction.** Handle `session_before_compact` so a long build's summary keeps
the phase, plan id, branch, completed and ready blocks, current retry,
deferred decisions and unattended ownership. Do not persist these facts only
in assistant prose. A resumed or compacted session must be able to continue
from the plan and CLI run state without replaying the whole conversation.

**Observability.** Persist presentation state with `pi.appendEntry()` and
reconstruct the latest state from `sessionManager.getBranch()` on startup. The
state contains the phase, plan id, work branch, current block, deferred count
and attended or unattended mode; it does not duplicate the CLI's run state.
Use `pi.setSessionName()` for the issue or plan, `pi.setLabel()` at contract
approval and completed blocks, `ctx.ui.setStatus()` for a compact phase marker,
`ctx.ui.setWidget()` for block progress, and `registerEntryRenderer()` for
transitions that belong in the transcript but not in model context.

**Materialisation.** Once the project extension survives a real multi-block
run, add a content-pack position for Pi harness assets and make `superdev sync`
materialise `.pi/extensions/superdev/`, any Pi-specific child-agent definitions,
and the existing Pi SOKF authoring skill. Continue to expose one canonical set
of workflow skills through resource discovery rather than maintaining a Pi
copy and a Claude copy. Add ownership, adoption, orphan, lock and drift behavior
for every materialised Pi file. Document project trust, extension permissions,
disabling the adapter and non-interactive degradation.

**Unit evidence.** Test the workflow-state reducer, branch-aware restoration,
command-to-skill forwarding, repository discovery, synthetic hook payload,
output truncation, recursive-child suppression and every phase-and-branch tool
gate. Keep state and command parsing in pure functions where possible.

**Integration evidence.** Drive a fake `ExtensionAPI` to prove the registered
commands, tools and handlers; hook exit 2 queues exactly one follow-up; hook
exit 0 queues none; another session's run is ignored; state restores from the
active branch; and a question without UI becomes deferred. Extend the existing
real-Pi adapter test pattern to start from a repository subdirectory, load each
phase, cross multiple turns in a two-block unattended fixture, stop a stalled
run at the watchdog cap, refresh ownership on resume, suppress child
continuation and leave the default branch unchanged.

**Behavior evidence.** Add versioned evaluations for duplicate issue
detection, scope interviews, codebase exploration instead of user questions,
contract approval before commit, test-first build order, contract-change return
to scope, retries and deferral, knowledge repair, optional acceptance,
acceptance filing rather than fixing gaps, missing UI, compaction during build
and a prompt that asks for an unattended default-branch mutation. Compare the
extension with plain skill loading. Require a high aggregate threshold and
zero failures for invented user decisions, unapproved contract commits or
unattended default-branch writes.

## Trade-offs

- A thin adapter keeps skills and `superdev` authoritative, but the adapter and
  the skill vocabulary still need a tested compatibility seam.
- Reusing one canonical skill set avoids semantic drift, but nested skill calls
  must become explicit Pi tool calls.
- Pi lifecycle and tool gates enforce more than prose alone, but a trusted
  project extension runs with the user's full system permissions.
- Child agents protect the driver's context and support specialist roles, but
  they add model cost and need recursion and working-tree concurrency guards.
- Calling `superdev hook run` preserves one continuation policy and watchdog,
  but the Pi adapter must synthesize a safe Claude-shaped payload unless the
  CLI gains a harness-neutral check verb.
- Session entries make progress visible and branch-aware, but storing run
  ownership there would create a second authority, so their state must remain
  presentational.
- Phase-specific tool activation reduces distraction, but replacing the whole
  active set would break composition with other extensions.
- Always materialising Pi files has no runtime effect outside Pi, but it adds
  files to repositories whose teams do not use Pi; harness selection avoids
  that at the cost of a new configuration surface.
- Automatic model or thinking-level profiles could improve difficult phases,
  but changing a user's model settings is intrusive and can increase cost.

## Open questions

- Should `superdev sync` always materialise every supported harness adapter, or
  should the manifest select `claude`, `pi`, or both?
- Should Pi discover `.claude/skills/` as the one canonical materialised skill
  set, or should the pack gain a harness-neutral destination?
- Should `superdev hook run` remain the cross-harness continuation interface,
  or should the CLI add a check verb that accepts a session id directly?
- Should phase model and thinking-level profiles remain advisory, or may the
  extension apply them with `setModel()` and `setThinkingLevel()`?
- Which unavailable capabilities need Pi-native fallbacks, and which should
  stop the phase with a named dependency?
- Should the extension register short `/file` and `/scope` aliases when another
  extension already owns those command names, or expose only namespaced forms?
- What measured behavioral-evaluation threshold is high enough to call the Pi
  adapter better than plain skill loading?

## Next step

Resolve issue-057, then spike the smallest adapter in
`.pi/extensions/superdev/`: expose the canonical skills, register the five phase
commands, provide one real question interaction and continue a two-step
synthetic run through `agent_settled` without parsing run state. Use the spike's
behavior evaluations to decide the continuation interface, pack layout and
harness-selection policy before promoting this idea to an issue.

<!-- sokf:links -->
[sokf:adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept]: /knowledge/adrs/deprecated/adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept.md
[sokf:issue-057-the-skills-disagree-on-who-loops-and-who-returns]: /knowledge/issues/done/issue-057-the-skills-disagree-on-who-loops-and-who-returns.md

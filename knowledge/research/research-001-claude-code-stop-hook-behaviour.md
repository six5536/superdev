---
type: Research
id: research-001-claude-code-stop-hook-behaviour
title: Claude Code Stop-hook Behaviour
description: >-
  The Stop hook's payload, the consecutive-block cap and what resets it,
  the CLAUDE_CODE_STOP_HOOK_BLOCK_CAP variable, exit 2 under
  stop_hook_active, CLAUDE_PROJECT_DIR, and session_id across resume.
sources:
  - id: hooks-reference
    resource: https://code.claude.com/docs/en/hooks
    title: Claude Code hooks reference
  - id: hooks-guide
    resource: https://code.claude.com/docs/en/hooks-guide
    title: Claude Code hooks guide
  - id: env-vars
    resource: https://code.claude.com/docs/en/env-vars
    title: Claude Code environment variables reference
  - id: settings-reference
    resource: https://code.claude.com/docs/en/settings-reference
    title: Claude Code settings reference
  - id: sessions
    resource: https://code.claude.com/docs/en/sessions
    title: Claude Code sessions guide
---

# Answers

Docs fetched 2026-08-31; experiments run against Claude Code v2.1.251.
Claims marked *observed* rest on the experiments described in
[Experiments](#experiments); the rest carry doc footnotes.

| # | Question | Answer | Basis |
| - | -------- | ------ | ----- |
| 1 | Stop payload | Common fields plus `stop_hook_active`, `last_assistant_message`, `background_tasks`, `session_crons`; see [Stop payload](#stop-payload) | Documented, confirmed observed |
| 2 | Block cap | 8 consecutive blocks without progress, then the block is overridden and the turn ends. Progress = any tool use in the continued turn, read-only included; each turn starts with a fresh count | Cap documented; progress semantics observed |
| 3 | `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP` | Max consecutive blocks before override; default 8; `0` disables. Settable in the shell or in any settings file's `env` block, `.claude/settings.json` included; the settings value overrides the shell | Documented, `env` application observed |
| 4 | Exit 2 while `stop_hook_active` is true | Effective: the block is honoured and counts toward the cap. No penalty beyond the cap | Observed; consistent with the docs |
| 5 | `CLAUDE_PROJECT_DIR` for Stop hooks | Set, as for every hook event | Documented, confirmed observed |
| 6 | `session_id` across `claude --resume` | Stable: the resumed session reports the original id. Only `/branch` and `--fork-session` create new ids | Observed; documented for the branch/fork side |

# Stop payload

Every hook receives the common fields; `permission_mode` and `effort`
are per-event and `effort` appears only when the current model supports
the effort parameter.[^hooks-reference]

| Field | Meaning |
| ----- | ------- |
| `session_id` | Current session identifier[^hooks-reference] |
| `prompt_id` | UUID of the user prompt being processed; absent until the first user input; requires v2.1.196+[^hooks-reference] |
| `transcript_path` | Path to conversation JSON; written asynchronously, so it may lag the in-memory conversation[^hooks-reference] |
| `cwd` | Working directory when the hook is invoked[^hooks-reference] |
| `permission_mode` | Current permission mode[^hooks-reference] |
| `effort` | Object with a `level` field; present when the model supports the effort parameter[^hooks-reference] |
| `hook_event_name` | `"Stop"`[^hooks-reference] |
| `agent_id`, `agent_type` | Only with `--agent` or inside a subagent[^hooks-reference] |

Stop adds four fields: "Stop hooks receive `stop_hook_active`,
`last_assistant_message`, `background_tasks`, and
`session_crons`."[^hooks-reference]

- `stop_hook_active`: "The `stop_hook_active` field is `true` when
  Claude Code is already continuing as a result of a stop hook. Check
  this value or process the transcript to avoid blocking on a condition
  that will never resolve."[^hooks-reference]
- `last_assistant_message`: "The `last_assistant_message` field
  contains the text content of Claude's final response, so hooks can
  access it without parsing the transcript file. For hooks that act on
  the just-completed turn, such as read-aloud or notification hooks,
  use this field rather than reading `transcript_path`: the transcript
  file isn't guaranteed to include the final message at Stop time on
  all versions."[^hooks-reference]
- `background_tasks`, `session_crons`: arrays that "let hooks
  distinguish 'session is done' from 'session is paused waiting for
  background work to wake it back up'"; present when the task registry
  is reachable, empty when nothing is in flight or
  scheduled.[^hooks-reference]

Observed payload of a `claude -p` run on Haiku (experiment 1): exactly
`background_tasks`, `cwd`, `hook_event_name`, `last_assistant_message`,
`permission_mode`, `prompt_id`, `session_crons`, `session_id`,
`stop_hook_active`, `transcript_path`. `effort` was absent.

# Block cap

## Documented

- Hooks guide, troubleshooting "Stop hook hits the block cap": "Claude
  keeps working instead of stopping, then ends the turn with a warning
  that the Stop hook blocked too many consecutive times. Claude Code
  overrides a Stop hook after it blocks eight times in a row without
  progress. Your hook script needs to check whether it already
  triggered a continuation. Parse the `stop_hook_active` field from the
  JSON input and exit early if it's `true` [...] If your hook
  legitimately needs more than eight iterations to converge, raise the
  cap with `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`."[^hooks-guide]
- Hooks reference: "Claude Code overrides the hook and ends the turn
  after 8 consecutive blocks."[^hooks-reference]
- The `additionalContext` output "keeps the conversation going through
  the same loop protections as `decision: "block"`, namely the
  `stop_hook_active` input and the 8-consecutive-continuation
  cap".[^hooks-reference]

The guide qualifies the cap with "without progress"; neither page
defines progress. The docs are silent on what resets the count.

## Observed

- Progress means tool use: with a cap of 3, continuations that each
  performed a Bash write sustained 14 consecutive honoured blocks
  (experiment 2b), and a read-only Read tool call reset the count the
  same way (experiment 2c). A continuation that only produced text
  counts toward the cap.
- The count is per turn. In one process handling two user prompts, each
  prompt's stop attempts ran a full fresh cycle, and `stop_hook_active`
  returned to `false` at the start of the second (experiment 3). The
  end of a turn — whether the hook allowed the stop or the cap
  overrode it — and the next user message arrive together on this
  path, so their individual contributions are not separable; jointly
  they reset the count.

# CLAUDE_CODE_STOP_HOOK_BLOCK_CAP

Environment variables reference: "Maximum number of consecutive times a
Stop or SubagentStop hook may block the turn from ending before Claude
Code overrides it and ends the turn anyway (default: 8). Set to `0` to
disable the cap. Raise this if your hook legitimately needs more
iterations to resolve".[^env-vars]

Where it can be set:

- In the shell that launches `claude`.[^env-vars]
- In the `env` key of any settings file: the `env` setting "Set[s]
  environment variables for every session and for the subprocesses
  Claude Code starts from it. Any variable in the environment variables
  reference can go here."[^settings-reference] The variable is not on
  the settings reference's ignore lists.[^settings-reference]
- "When the same variable is set in both your shell and a settings file
  `env` block, the settings file value applies."[^env-vars]
- Project and local `env` values apply "after you trust the workspace,
  or at startup in `-p` mode, which never shows the trust dialog", and
  variables Claude Code classifies as safe — "timeouts and limits"
  among them — apply "at startup from every settings
  file".[^settings-reference]

Observed: `"env": {"CLAUDE_CODE_STOP_HOOK_BLOCK_CAP": "3"}` in the test
repository's `.claude/settings.json` capped the Stop-hook loop of a
`claude -p` run at 3 honoured blocks, in a workspace that was never
trusted (experiment 2a).

# Exit 2 while stop_hook_active is true

Exit 2 from a Stop hook "Prevents Claude from stopping, continues the
conversation", with stderr routed to Claude as the reason to
continue.[^hooks-reference] The docs attach no penalty to blocking while
`stop_hook_active` is true; the field is guidance for the hook
author.[^hooks-reference]

Observed: in experiment 1 the hook exited 2 on all nine invocations;
invocations 2 through 8 carried `stop_hook_active: true` and each block
was honoured. Only the ninth consecutive block — past the cap of 8 —
was overridden.

# CLAUDE_PROJECT_DIR

"`${CLAUDE_PROJECT_DIR}`: the project root where the session
started."[^hooks-reference] Hook commands in both shell and exec form
"export them as the environment variables `CLAUDE_PROJECT_DIR`,
`CLAUDE_PLUGIN_ROOT`, and `CLAUDE_PLUGIN_DATA` on the spawned
process".[^hooks-reference] This is hook-wide, Stop included; the page
draws no distinction from PostToolUse. After Claude enters a worktree,
`${CLAUDE_PROJECT_DIR}` stays at the original project root and the
payload's `cwd` follows Claude.[^hooks-reference]

Observed: every Stop-hook invocation in every experiment saw
`CLAUDE_PROJECT_DIR` set to the test repository root.

# session_id across resume

Observed: a session run with `claude -p`, then resumed with
`claude -p --resume <session-id>`, reported the same `session_id` in
the resumed run's Stop payload (experiment 4).

The sessions guide documents the complementary fact: "Sessions created
with `/branch` or `--fork-session` get their own session IDs", and a
plain resume addresses the session by the id it keeps.[^sessions]

# Experiments

Environment: Claude Code v2.1.251 on Linux, 2026-08-31, model Haiku,
non-interactive (`claude -p`). Test bed: a fresh git repository outside
this one, whose `.claude/settings.json` registered a Stop hook script
that appends its stdin JSON plus its `CLAUDE_PROJECT_DIR` to a log,
then exits 2 with a stderr instruction; as a runaway guard the script
exits 0 from its fifteenth invocation.

| # | Setup | Result |
| - | ----- | ------ |
| 1 | Default cap; stderr asks for a plain text reply | 9 invocations: `stop_hook_active` false then true; blocks 1-8 honoured, the 9th overridden |
| 2a | Cap 3 via project settings `env`; plain text reply | 4 invocations: 3 honoured, the 4th overridden |
| 2b | Cap 3; stderr demands a Bash append to a file before stopping | 15 invocations, 14 files lines written; every block honoured until the script's own exit 0 ended the run |
| 2c | Cap 3; stderr demands a read-only Read before stopping | 10 invocations. The transcript shows the model complied on four continuations; modelling the counter as "increment per tool-less blocked stop, reset on tool use" predicts the override at exactly invocation 10 |
| 3 | Cap 3; two user messages in one process via `--input-format stream-json` | 8 invocations as two independent cycles of 4; `stop_hook_active` false at each cycle's start |
| 4 | `claude -p --resume <session-id>` of run 3's session | The resumed run's payload carries run 3's `session_id` |

[^hooks-reference]: Claude Code hooks reference
[^hooks-guide]: Claude Code hooks guide
[^env-vars]: Claude Code environment variables reference
[^settings-reference]: Claude Code settings reference
[^sessions]: Claude Code sessions guide

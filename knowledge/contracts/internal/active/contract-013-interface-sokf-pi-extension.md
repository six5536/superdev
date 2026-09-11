---
type: Contract
id: contract-013-interface-sokf-pi-extension
kind: interface
title: Interface contract for the SOKF Pi extension
description: The turn-end repair-and-validate pass that makes canonical knowledge consistent whatever wrote it, its bounded session-scoped reporting, and the repository root every SOKF operation is confined to.
lifecycle: active
resource: /.pi/extensions/sokf.ts
links:
  - rel: references
    to: adr-055-sokf-does-not-intercept-file-tools
    note: The decision this contract binds — knowledge is made consistent once per turn rather than at each mutation.
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: The Definition is materialised from the `turn-end` region rather than authored.
  - rel: references
    to: contract-003-api-sokf
    note: The MCP server the extension calls; this contract binds the extension, not the server.
---

# Interface contract: the SOKF Pi extension

The extension that gives a Pi session its SOKF tools, and makes the canonical
knowledge consistent when a turn ends. The Definition is the `turn_end` handler
as the source declares it.

[ADR-055][sokf:adr-055-sokf-does-not-intercept-file-tools] decided that
consistency is established once per turn rather than at each mutation, because
an agent writes knowledge through `bash`, a heredoc, `sed`, a patch, or
`git checkout` as readily as through a file tool. This contract binds that
mechanism. [contract-003][sokf:contract-003-api-sokf] binds the MCP server the
extension calls; the two are separate surfaces. The Definition is materialised
from the `turn-end` region rather than authored, under
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source].

## Definition

<!-- sokf:include /.pi/extensions/sokf.ts#turn-end -->
```typescript
	pi.on("turn_end", async (_event, ctx) => {
		const repository = findRepository(ctx.cwd);
		if (!repository) return;

		// Unconditional: no precondition, and no flag deciding whether the
		// knowledge changed. A write through `bash`, a heredoc, `sed`, a patch,
		// or `git checkout` reaches no tool this extension registers, so any
		// signal it could keep would be wrong on exactly the cases this check
		// exists to cover. `--fix` repairs first, so the findings that survive
		// are the ones repair could not settle.
		const result = await runSuperdev(ctx.cwd, ["validate", "--fix"]);
		if (result.code === 0) {
			reported.delete(repository);
			return;
		}

		// Validation ran against the tree as it was; repair, a later write, or a
		// merge can settle a finding between that run and this send. Read the
		// tree again and report only what it still carries, so a resolved
		// finding never costs a turn.
		const surviving = await stillFailing(ctx.cwd);
		if (!surviving) {
			reported.delete(repository);
			return;
		}

		const first = !reported.has(repository);
		reported.add(repository);
		pi.sendMessage(
			{
				customType: "sokf-validation",
				content: `${
					first
						? "SOKF validation fails after automatic repair. Fix these findings."
						: "SOKF validation still fails after automatic repair."
				}\n\n${surviving}`,
				display: true,
			},
			// The first report of a sequence gets the agent one prompted chance to
			// correct. A later one is visible and leaves the decision to it: an
			// agent that reads a repeated report can check the tree itself.
			first ? { deliverAs: "followUp", triggerTurn: true } : { deliverAs: "nextTurn" },
		);
	});
```
<!-- /sokf:include -->

## Behaviour

### Module boundaries

The extension owns tool registration, the MCP client per repository root, and
the turn-end pass. It owns no knowledge semantics: resolution, search, graph
traversal, repair, and validation are the `superdev` binary's, reached over MCP
or by spawning the command. Follow-up state lives in the extension and nowhere
else.

- `P_repository-discovery` [ubiquitous] The extension SHALL select the nearest
  ancestor carrying a `.git` directory or worktree pointer file as the
  repository root.
  - `AC_common-directory-is-not-the-root` [ubiquitous] The extension SHALL NOT
    select Git's shared common directory or main checkout as the root of a
    linked worktree.
  - `AC_configured-fallback` [event] WHEN the upward walk finds no `.git`
    marker, the extension SHALL select the nearest ancestor carrying
    `.superdev/config.toml`.
  - `AC_nested-working-directory` [event] WHEN Pi starts in a descendant of the
    repository root, the extension SHALL select the same root it selects from
    the root itself.
- `P_active-checkout-only` [ubiquitous] Every MCP client, spawned command, and
  follow-up entry SHALL be keyed by the canonical active checkout root.
  - `AC_roots-do-not-share-state` [event] WHEN one session serves two checkout
    roots, each root SHALL carry its own client and its own follow-up entry.
- `P_no-knowledge-semantics` [ubiquitous] The extension SHALL delegate
  resolution, retrieval, search, graph traversal, repair, and validation to the
  `superdev` binary.

### Key flows

1. A turn ends. The extension selects the repository root; without one it does
   nothing.
2. It runs `superdev validate --fix` once, unconditionally, without first
   deciding whether the knowledge changed.
3. A passing run clears the repository's follow-up entry and sends nothing.
4. A failing run is re-checked against the working tree, because repair, a
   later write, or a merge can settle a finding between the run and the send.
5. A surviving report is sent once: triggering for the first report of a
   sequence, visible and non-triggering afterwards.

- `P_turn-end-runs-unconditionally` [event] WHEN a turn ends in a repository,
  the extension SHALL run `superdev validate --fix` once.
  - `AC_no-change-precondition` [ubiquitous] The extension SHALL hold no flag
    or signal deciding whether the knowledge changed.
  - `AC_covers-every-writer` [event] WHEN a turn changed knowledge through no
    tool the extension registers, the run SHALL still happen.
  - `AC_valid-tree-unchanged` [event] WHEN the knowledge is already valid, the
    run SHALL leave every file byte-identical.
- `P_findings-are-fresh` [event] WHEN the extension prepares a report, it SHALL
  re-read the working tree and report only what the tree still carries.
  - `AC_resolved-finding-not-sent` [event] WHEN every finding is settled
    between the run and the send, the extension SHALL send no message.
- `P_one-triggering-report` [event] WHEN validation first reports for a pending
  sequence, the extension SHALL send one visible triggering `sokf-validation`
  follow-up.
  - `AC_later-report-does-not-trigger` [event] WHEN validation reports again in
    the same sequence, the extension SHALL send one visible non-triggering
    message.
  - `AC_clean-run-resets` [event] WHEN a later run passes, the extension SHALL
    clear the sequence so a subsequent failure triggers once again.

### Cross-cutting concerns

Follow-up state is session memory alone: nothing persists, so reload, resume,
fork, and compaction need no recovery rule, and no append can fail. Reports are
bounded because an unbounded one costs the agent the context it needs to act on
them.

- `P_state-is-session-memory` [ubiquitous] Follow-up state SHALL live in
  process memory alone.
  - `AC_nothing-persists` [ubiquitous] The extension SHALL NOT write follow-up
    state to any file.
- `P_report-bounded` [ubiquitous] Each visible report SHALL stop at 200 lines
  or 8 KiB and direct truncated output to `superdev validate`.
- `P_report-is-actionable` [ubiquitous] Each visible report SHALL carry paths,
  locations where available, messages, and corrective actions.
  - `AC_no-patches-in-a-report` [ubiquitous] A visible report SHALL NOT carry a
    patch or a mutation envelope.
- `P_mcp-process-reused` [ubiquitous] The extension SHALL reuse one MCP process
  per repository root for the session.
  - `AC_closed-at-shutdown` [event] WHEN the session shuts down, the extension
    SHALL close every MCP process it started.

## Stability

Project-local.

- `P_project-local` [ubiquitous] Every item above MAY change with this
  repository.

<!-- sokf:links -->
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:adr-055-sokf-does-not-intercept-file-tools]: /knowledge/adrs/active/adr-055-sokf-does-not-intercept-file-tools.md
[sokf:contract-003-api-sokf]: /knowledge/contracts/public/active/contract-003-api-sokf.md

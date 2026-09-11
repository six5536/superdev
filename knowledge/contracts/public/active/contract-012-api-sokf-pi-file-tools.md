---
type: Contract
id: contract-012-api-sokf-pi-file-tools
kind: api
title: API contract for SOKF-routed Pi file tools
description: Pi 0.85.1 read, edit, and write behavior for canonical knowledge identities, plus SOKF safety and bounded validation follow-ups.
lifecycle: active
resource: /.pi/extensions/sokf.ts
links:
  - rel: references
    to: adr-053-sokf-file-tools-delegate-to-pi
    note: Pi owns file semantics and presentation while SOKF owns routing, safety, persistence outcomes, and validation follow-ups.
---

# API contract: SOKF-routed Pi file tools

The adapter preserves Pi 0.85.1 file-tool behavior under [ADR-053][sokf:adr-053-sokf-file-tools-delegate-to-pi]. Pending promises describe the proposed implementation target; BUILD owns source declarations and Definition materialization.

## Definition

<!-- sokf:include /.pi/extensions/sokf.ts#tools -->
```typescript
	pi.registerTool({
		// Pi owns the schema, argument preparation, metadata, result
		// construction, and rendering; SOKF replaces only the execution target.
		...createReadToolDefinition(process.cwd()),
		description:
			"Read files and SOKF project knowledge. Use path sokf:<id> to read a concept's source exactly as it is written.",
		promptSnippet: "Read file contents or canonical project knowledge via sokf:<id>",
		promptGuidelines: [
			"Use read with a sokf:<id> path when a known SOKF concept can answer the project question; do not search before reading an ID already named.",
			"Use read for ordinary files exactly as usual.",
		],
		async execute(toolCallId, params, signal, onUpdate, ctx) {
			const asked = stripAt(params.path);
			// An already-cancelled call is Pi's to refuse, in Pi's own words,
			// before any target is resolved or opened.
			if (signal?.aborted || (!isSokfAddress(params.path) && !(await routesRead(params.path, ctx.cwd)))) {
				return createReadTool(ctx.cwd).execute(toolCallId, params, signal, onUpdate);
			}
			if (asked === "sokf:" || (asked.startsWith("sokf:") && asked.includes("#"))) {
				throw new Error(
					`read serves file source; \`${asked}\` addresses rendered knowledge. Use sokf_search to find it, or read sokf:<id> for the source.`,
				);
			}
			// SOKF resolves identity and containment; Pi owns everything after.
			const repository = requireRepository(ctx.cwd);
			const request = asked.startsWith("sokf:") ? asked : await prepareReadPath(params.path, ctx.cwd);
			const resolved = sourceResolution(await invokeMcp(ctx.cwd, "sokf_resolve_source", { path: request }, signal));
			const target = resolve(repository, resolved.canonicalPath);
			const result = await createReadTool(ctx.cwd).execute(
				toolCallId,
				{ ...params, path: target },
				signal,
				onUpdate,
			);
			return restoreSpelling(result, target, params.path);
		},
	});

	pi.registerTool({
		name: "sokf_search",
		label: "SOKF search",
		description: "Search SOKF whenever project knowledge is needed. Returns matching sections and locators.",
		promptSnippet: "Search the canonical SOKF project knowledge semantically",
		promptGuidelines: ["Use sokf_search whenever project knowledge is needed and no concept ID is already known."],
		parameters: searchSchema,
		async execute(_toolCallId, params: SearchInput, signal, _onUpdate, ctx) {
			return boundedResult(
				await invokeMcp(ctx.cwd, "sokf_search", { ...params }, signal),
			);
		},
	});

	pi.registerTool({
		name: "sokf_graph",
		label: "SOKF graph",
		description: "Follow typed relationships in the canonical SOKF project knowledge, or show its complete edge map.",
		promptSnippet: "Traverse relationships in the canonical SOKF project knowledge",
		parameters: graphSchema,
		async execute(_toolCallId, params: GraphInput, signal, _onUpdate, ctx) {
			return boundedResult(await invokeMcp(ctx.cwd, "sokf_graph", { ...params }, signal));
		},
	});

	pi.registerTool({
		name: "sokf_overview",
		label: "SOKF overview",
		description:
			"Show the canonical SOKF project knowledge at a glance: its name, how many concepts it holds, the tree of them, the index state, and anything wrong with it.",
		promptSnippet: "See the canonical SOKF project knowledge at a glance",
		promptGuidelines: [
			"Use sokf_overview to orient in an unfamiliar repository's knowledge before searching it.",
		],
		parameters: overviewSchema,
		async execute(_toolCallId, _params, signal, _onUpdate, ctx) {
			return boundedResult(await invokeMcp(ctx.cwd, "sokf_overview", {}, signal));
		},
	});

	pi.registerTool({
		name: "edit",
		label: "edit",
		description:
			"Edit files using exact replacements. SOKF virtual addresses and physical knowledge paths use agent-safe mutation with automatic repair and validation.",
		promptSnippet: "Make exact file edits; route SOKF knowledge through safe repair and validation",
		promptGuidelines: [
			"Use edit with sokf:<id> for an existing SOKF concept; do not use section-qualified addresses for mutation.",
			"Each edit is matched against the same original content and must be unique and non-overlapping.",
		],
		parameters: createEditTool(process.cwd()).parameters,
		async execute(toolCallId, params, signal, onUpdate, ctx) {
			const route = routesMutation(params.path, ctx.cwd);
			if (!route.route) {
				return createEditTool(ctx.cwd).execute(toolCallId, params, signal, onUpdate);
			}
			return withFileMutationQueue(route.queue, async () => {
				const envelope = await invokeMcp(
					ctx.cwd,
					"sokf_edit",
					{ ...params, path: route.target },
					signal,
				);
				const details = mutationDetails(envelope);
				return {
					content: envelope.content,
					details: editDetails(details),
				};
			});
		},
	});

	pi.registerTool({
		name: "write",
		label: "write",
		description:
			"Write complete files. SOKF virtual addresses and physical knowledge paths use agent-safe mutation with automatic repair and validation.",
		promptSnippet: "Create or overwrite files; route SOKF knowledge through safe repair and validation",
		promptGuidelines: [
			"Use write with a physical knowledge/<path>.md path to create a SOKF concept; IDs do not determine placement.",
			"Use write only for new files or complete rewrites.",
		],
		parameters: createWriteTool(process.cwd()).parameters,
		async execute(toolCallId, params, signal, onUpdate, ctx) {
			const route = routesMutation(params.path, ctx.cwd);
			if (!route.route) {
				return createWriteTool(ctx.cwd).execute(toolCallId, params, signal, onUpdate);
			}
			return withFileMutationQueue(route.queue, async () => {
				const envelope = await invokeMcp(
					ctx.cwd,
					"sokf_write",
					{ ...params, path: route.target },
					signal,
				);
				mutationDetails(envelope);
				return { content: envelope.content, details: undefined };
			});
		},
	});
```
<!-- /sokf:include -->

## Behaviour

### Transport

Validation follow-up state is session memory keyed by canonical repository root. `Finding` has the exact closed shape bound by `contract-003-api-sokf P_routed-schemas`. The adapter persists no follow-up state, so a new session starts with none.

- `P_worktree-isolation` [ubiquitous] The adapter SHALL
  PENDING(issue-083) scope every SOKF operation and validation sequence
  to the canonical active checkout root, including a linked Git worktree.
  - `AC_worktree-discovery` [event] WHEN Pi starts at a linked worktree root or
    its descendant, the adapter SHALL recognize its
    `.git` pointer file and select that worktree rather than the shared Git
    directory or main checkout.
  - `AC_worktree-git-precedence` [event] WHEN the upward walk from the working
    directory finds no `.git` directory or pointer file, the adapter SHALL
    select the nearest ancestor carrying
    `.superdev/config.toml`, so a managed non-Git repository keeps SOKF routing
    while any `.git` marker on the walk always wins.
  - `AC_worktree-all-operations` [state] WHILE a linked worktree is active,
    routed file tools, semantic retrieval, search, graph traversal, MCP process
    and index activity, mutation, repair, refiling, and validation SHALL
    PENDING(issue-083) use that worktree.
  - `AC_worktree-state-isolated` [event] WHEN one process serves multiple
    checkout roots, each checkout SHALL PENDING(issue-083) have a
    distinct MCP client, canonical queue targets, pending flag, findings,
    and validation execution.
  - `AC_worktree-cross-checkout-refused` [event] WHEN a routed path escapes
    into another checkout, the adapter SHALL reject
    it before access or mutation.
- `P_source-routing` [ubiquitous] The adapter SHALL resolve a routed argument to one logical knowledge ingress and one canonical repository-contained target before invoking a Pi file-tool factory.
  - `AC_source-existing` [event] WHEN an identity or physical knowledge path names an existing regular file, the adapter SHALL execute against its canonical target.
  - `AC_source-missing` [event] WHEN a physical knowledge path names a missing destination, the adapter SHALL canonicalize its nearest existing ancestor and append the normalized missing suffix.
  - `AC_source-nested-cwd` [event] WHEN Pi starts at the repository root or a nested working directory, the adapter SHALL resolve equivalent arguments to the same target.
  - `AC_source-pi-preparation` [event] WHEN Pi prepares a path argument, the adapter SHALL pass the byte-equivalent candidate observed at the Pi 0.85.1 operation boundary.
  - `AC_source-contained-symlink` [event] WHEN a knowledge ingress symlink resolves inside the repository, the adapter SHALL accept its canonical target even when that target lies outside `knowledge/`.
  - `AC_source-closed-schema` [ubiquitous] The adapter SHALL PENDING(issue-082) send and validate the exact closed resolver, retrieval, edit, write, and mutation-result schemas in `contract-003-api-sokf P_routed-schemas` without accepting missing or additional fields.
  - `AC_source-escaping-symlink` [event] WHEN an existing or ancestor symlink resolves outside the repository, the adapter SHALL reject the argument before file access or mutation.
  - `AC_source-target-drift` [event] WHEN an ingress resolves to a different canonical target at mutation dispatch, the adapter SHALL PENDING(issue-082) reject the mutation before persistence.
- `P_read-parity` [ubiquitous] Routed `read` SHALL preserve Pi 0.85.1 content, offset, limit, truncation, continuation, details, errors, cancellation, metadata, and rendering.
  - `AC_read-exact-source` [event] WHEN `read` receives an unqualified `sokf:<id>`, the adapter SHALL return exact UTF-8 source through Pi's built-in read operation.
  - `AC_read-semantic-refused` [event] WHEN Pi file `read` receives `sokf:` or a section-qualified address, the adapter SHALL reject it with concise guidance naming `sokf_search`.
  - `AC_read-description-accurate` [ubiquitous] The routed `read` registration SHALL describe only the address forms it accepts and advertise no refused overview or section address.
  - `AC_read-copied-edit` [event] WHEN a caller copies a routed excerpt including frontmatter, routed `edit` SHALL PENDING(issue-082) accept the unchanged excerpt as `oldText`.
- `P_edit-parity` [ubiquitous] Routed `edit` SHALL PENDING(issue-082) preserve Pi 0.85.1 schema, argument preparation, matching, uniqueness, overlap, byte-order mark, line endings, errors, success content, compact details, metadata, and rendering.
  - `AC_edit-pi-diffs` [event] WHEN a routed edit applies, its details SHALL PENDING(issue-082) contain Pi's display diff, unified patch, and first changed line for the requested edit.
- `P_write-parity` [ubiquitous] Routed `write` SHALL PENDING(issue-082) preserve Pi 0.85.1 parameters, recursive parent creation, errors, cancellation, success content, undefined details, metadata, and rendering.
  - `AC_write-identical-applies` [event] WHEN write content equals existing target bytes, the adapter SHALL PENDING(issue-082) persist and acknowledge the write and schedule final validation.
- `P_alias-queue` [ubiquitous] Virtual and physical aliases SHALL PENDING(issue-082) serialize on Pi's mutation queue keyed by canonical physical target through persistence, repair, refiling, and mutation-state capture.
- `P_agent-safe-routing` [ubiquitous] Routed mutations SHALL PENDING(issue-082) preserve stable identity, existing verification bytes, generated ownership, repository containment, repair, refiling, and validation under agent-safe policy.
  - `AC_generated-authority` [event] WHEN a mutation intersects generated content, the adapter SHALL PENDING(issue-082) reject it before persistence and name the authoritative source.
  - `AC_no-whole-file-recovery` [event] WHEN a targeted edit fails, the adapter SHALL NOT PENDING(issue-082) perform or recommend an automatic whole-file fallback.
- `P_mutation-outcome-boundary` [ubiquitous] The adapter SHALL PENDING(issue-082) use authoritative `applied: true` as the boundary between a file-tool failure and Pi's normal successful result.
  - `AC_pre-persistence-abort` [event] WHEN cancellation occurs before persistence dispatch, the adapter SHALL PENDING(issue-082) leave every target unchanged and create no applied state.
  - `AC_pre-persistence-failure` [event] WHEN resolution, preparation, matching, compare-and-swap, policy, ownership, or authoritative persistence fails before apply, the adapter SHALL PENDING(issue-082) return an applicable tool error and schedule no final validation.
  - `AC_edit-persisted-success` [event] WHEN MCP acknowledges an edit with `applied: true`, the adapter SHALL PENDING(issue-082) return Pi's unchanged edit success content and details.
  - `AC_write-persisted-success` [event] WHEN MCP acknowledges a write with `applied: true`, the adapter SHALL PENDING(issue-082) return Pi's unchanged write success content and undefined details.
  - `AC_late-abort-keeps-success` [event] WHEN cancellation occurs after persistence dispatch and MCP acknowledges `applied: true`, the adapter SHALL PENDING(issue-082) retain persisted changes and return success.
  - `AC_successful-repairs-survive` [event] WHEN repair or refiling persists before a later finding, the adapter SHALL PENDING(issue-082) retain every successful persisted change.
  - `AC_indeterminate-outcome-guidance` [event] WHEN local MCP transport fails without an authoritative outcome after bytes may have changed, the adapter SHALL PENDING(issue-082) return a tool error directing target inspection and `superdev validate`.
  - `AC_failed-persistence-not-dirty` [event] WHEN authoritative persistence establishes that apply did not occur, the adapter SHALL PENDING(issue-082) create no applied state or validation schedule.
- `P_validation-follow-up` [ubiquitous] The adapter SHALL PENDING(issue-083) report post-persistence validation findings outside file-tool results.
  - `AC_turn-end-validation` [event] WHEN a turn ends after an applied mutation, the adapter SHALL PENDING(issue-083) run final validation once.
  - `AC_findings-fresh` [event] WHEN the adapter prepares a validation message, it SHALL PENDING(issue-083) report only findings that the current working tree still carries.
  - `AC_valid-clears` [event] WHEN final validation succeeds, the adapter SHALL PENDING(issue-083) clear its pending flag and send no message.
  - `AC_first-report-triggers` [event] WHEN validation first reports findings for a pending sequence, the adapter SHALL PENDING(issue-083) send one visible triggering `sokf-validation` follow-up separate from file-tool content and details.
  - `AC_later-report-does-not-trigger` [event] WHEN validation reports findings again after a correction turn, the adapter SHALL PENDING(issue-083) send one visible non-triggering message.
  - `AC_session-scoped` [ubiquitous] The adapter SHALL PENDING(issue-083) hold follow-up state in session memory alone and persist no snapshot.
  - `AC_follow-up-bounded` [ubiquitous] Each visible follow-up SHALL PENDING(issue-083) stop at 200 lines or 8 KiB, omit patches and mutation envelopes, and direct truncated output to `superdev validate`.
- `P_context-shape` [ubiquitous] An acknowledged mutation result SHALL PENDING(issue-082) keep mutation envelopes, repair reports, validation findings, and duplicate patches outside model-visible file-tool content and details.
- `P_prompt-ownership` [ubiquitous] The extension SHALL PENDING(issue-083) retain Pi's built-in file-tool metadata and add only the specified flat SOKF routing rules.
  - `AC_discovery-wording` [ubiquitous] `sokf_search` and `sokf_graph` SHALL PENDING(issue-083) own canonical-knowledge discovery wording.

### Authentication

No credential crosses this local adapter boundary.

- `P_local-authority` [ubiquitous] Routed mutations SHALL PENDING(issue-082) use mandatory agent-safe authority with no model-facing override.

### Errors

- `P_pi-errors` [event] WHEN SOKF policy does not intervene, routed file-tool failure wording SHALL PENDING(issue-082) match paired Pi 0.85.1 behavior.
- `P_policy-errors` [event] WHEN SOKF policy rejects a mutation before persistence, the adapter SHALL PENDING(issue-082) return a concise actionable tool error.
- `P_post-persistence-findings` [event] WHEN repair, validation, lifecycle, or cancellation reports a problem after acknowledged apply, the adapter SHALL PENDING(issue-083) preserve success and report the problem separately.

### Limits

- `P_validation-diagnostic-limit` [ubiquitous] Each extension-side validation diagnostic SHALL PENDING(issue-083) stop at 200 lines or 8 KiB.

### Versioning

- `P_pinned-pi` [ubiquitous] Paired evidence SHALL load the pinned `@earendil-works/pi-coding-agent` 0.85.1 test dependency rather than an executable on `PATH`, and fail rather than skip when that dependency cannot load.

### Resources and prompts

- `P_file-prompt-rules` [ubiquitous] File-tool prompt metadata SHALL PENDING(issue-083) add only identity-read, identity-edit, and physical-path-create routing guidance.

## Stability

Project-local and pinned to Pi 0.85.1.

- `P_project-local` [ubiquitous] The SOKF adapter MAY change with this repository.
- `P_contract-synchronized` [ubiquitous] The adapter contract SHALL PENDING(issue-083) remain synchronized with paired evidence.

<!-- sokf:links -->
[sokf:adr-053-sokf-file-tools-delegate-to-pi]: /knowledge/adrs/deprecated/adr-053-sokf-file-tools-delegate-to-pi.md

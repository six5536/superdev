---
type: Contract
id: contract-012-api-sokf-pi-file-tools
kind: api
title: API contract for SOKF-routed Pi file tools
description: The record of the SOKF-routed Pi read, edit, and write tools — the repository they were confined to, the routing and read behaviour that was delivered, and the mutation parity that never was.
lifecycle: deprecated
resource: /archive/pi/sokf-file-tools.ts
links:
  - rel: references
    to: adr-053-sokf-file-tools-delegate-to-pi
    note: The decision that created this interface — Pi owned file semantics and presentation while SOKF owned routing, safety, and persistence outcomes.
  - rel: references
    to: adr-055-sokf-does-not-intercept-file-tools
    note: The decision that retired it — SOKF serves no file tool, and knowledge is made consistent once at turn end.
---

# API contract: SOKF-routed Pi file tools

The routed `read`, `edit`, and `write` registrations that SOKF added to a Pi
session under [ADR-053][sokf:adr-053-sokf-file-tools-delegate-to-pi], and that
[ADR-055][sokf:adr-055-sokf-does-not-intercept-file-tools] removed. A Pi session
now gets Pi's own file tools on physical paths, and the knowledge is repaired and
validated once at turn end. Nothing loads the registrations this contract
defines; they survive in `/archive/pi/sokf-file-tools.ts` so the Definition
materialises from source rather than a hand copy. This contract is history, and
the promises below record what the adapter did, not what it was to become.

The live extension is bound by
[contract-013][sokf:contract-013-interface-sokf-pi-extension], which inherits
the substance of the turn-end reporting, worktree confinement, and diagnostic
cap this contract carried.

## Definition

<!-- sokf:include /archive/pi/sokf-file-tools.ts#tools -->
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

The adapter ran inside the Pi process and reached SOKF over a local MCP client.
Routed arguments were resolved against the canonical active checkout root, so a
nested working directory and a linked worktree reached the same target.

- `P_worktree-isolation` [ubiquitous] The adapter SHALL select the canonical
  active checkout root as the repository for every routed argument.
  - `AC_worktree-discovery` [event] WHEN Pi starts at a linked worktree root or
    its descendant, the adapter SHALL recognize its
    `.git` pointer file and select that worktree rather than the shared Git
    directory or main checkout.
  - `AC_worktree-git-precedence` [event] WHEN the upward walk from the working
    directory finds no `.git` directory or pointer file, the adapter SHALL
    select the nearest ancestor carrying
    `.superdev/config.toml`, so a managed non-Git repository keeps SOKF routing
    while any `.git` marker on the walk always wins.
  - `AC_worktree-cross-checkout-refused` [event] WHEN a routed path escapes
    into another checkout, the adapter SHALL reject
    it before access or mutation.
- `P_source-routing` [ubiquitous] The adapter SHALL resolve a routed argument to one logical knowledge ingress and one canonical repository-contained target before invoking a Pi file-tool factory.
  - `AC_source-existing` [event] WHEN an identity or physical knowledge path names an existing regular file, the adapter SHALL execute against its canonical target.
  - `AC_source-missing` [event] WHEN a physical knowledge path names a missing destination, the adapter SHALL canonicalize its nearest existing ancestor and append the normalized missing suffix.
  - `AC_source-nested-cwd` [event] WHEN Pi starts at the repository root or a nested working directory, the adapter SHALL resolve equivalent arguments to the same target.
  - `AC_source-pi-preparation` [event] WHEN Pi prepares a path argument, the adapter SHALL pass the byte-equivalent candidate observed at the Pi 0.85.1 operation boundary.
  - `AC_source-contained-symlink` [event] WHEN a knowledge ingress symlink resolves inside the repository, the adapter SHALL accept its canonical target even when that target lies outside `knowledge/`.
  - `AC_source-escaping-symlink` [event] WHEN an existing or ancestor symlink resolves outside the repository, the adapter SHALL reject the argument before file access or mutation.
- `P_read-parity` [ubiquitous] Routed `read` SHALL preserve Pi 0.85.1 content, offset, limit, truncation, continuation, details, errors, cancellation, metadata, and rendering.
  - `AC_read-exact-source` [event] WHEN `read` receives an unqualified `sokf:<id>`, the adapter SHALL return exact UTF-8 source through Pi's built-in read operation.
  - `AC_read-semantic-refused` [event] WHEN Pi file `read` receives `sokf:` or a section-qualified address, the adapter SHALL reject it with concise guidance naming `sokf_search`.
  - `AC_read-description-accurate` [ubiquitous] The routed `read` registration SHALL describe only the address forms it accepts and advertise no refused overview or section address.

Routed `edit` and `write` reached persistence but never reached parity with
Pi's own mutation behaviour. Their matching, line-ending handling, diff details,
and outcome boundary were the subject of issue-082, which was declined with
ADR-055.

### Authentication

No credential crossed this local adapter boundary. Write authority was bounded
by policy rather than caller identity.

- `P_local-authority` [ubiquitous] Routed mutations SHALL use mandatory
  agent-safe mutation authority with no model-facing override.

### Errors

A routed failure was a Pi tool error. Routed read errors used Pi's wording
except where SOKF policy intervened; mutation errors never reached Pi parity.
Reporting a post-persistence
finding separately from the successful result was the intent of the declined
issue-083 and was never built: an acknowledged mutation carried its repair and
validation findings in model-visible content to the end.

- `P_pi-errors` [event] WHEN SOKF policy did not intervene, routed `read` failure
  wording SHALL match paired Pi 0.85.1 behavior.
- `P_policy-errors` [event] WHEN SOKF policy rejected a routed `read` argument
  before access, the adapter SHALL return a concise actionable tool error.
- `P_post-persistence-findings` [event] WHEN repair or validation reports a
  problem after an acknowledged apply, the adapter SHALL report it beside the
  successful tool result. This was the unbuilt intent of declined issue-083,
  not delivered behaviour.

### Limits

- `P_validation-diagnostic-limit` [ubiquitous] Each extension-side validation diagnostic SHALL stop at 200 lines or 8 KiB.

### Versioning

- `P_pinned-pi` [ubiquitous] Paired evidence SHALL load the pinned `@earendil-works/pi-coding-agent` 0.85.1 test dependency rather than an executable on `PATH`, and fail rather than skip when that dependency cannot load.

## Stability

Withdrawn. No caller may rely on this interface: the registrations are archived
and nothing loads them.

- `P_project-local` [ubiquitous] The SOKF adapter MAY change with this repository.
- `P_withdrawn` [ubiquitous] A session SHALL NOT receive a SOKF-routed `read`, `edit`, or `write` tool.

<!-- sokf:links -->
[sokf:adr-053-sokf-file-tools-delegate-to-pi]: /knowledge/adrs/deprecated/adr-053-sokf-file-tools-delegate-to-pi.md
[sokf:adr-055-sokf-does-not-intercept-file-tools]: /knowledge/adrs/active/adr-055-sokf-does-not-intercept-file-tools.md
[sokf:contract-013-interface-sokf-pi-extension]: /knowledge/contracts/internal/active/contract-013-interface-sokf-pi-extension.md

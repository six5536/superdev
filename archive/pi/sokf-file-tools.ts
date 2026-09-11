// sokf-file-tools.ts — the SOKF-routed Pi file tools, as they were.
//
// These registrations routed Pi's `read`, `edit`, and `write` through SOKF
// identity resolution and agent-safe mutation. issue-095 removed the routing:
// a Pi session now gets Pi's own file tools on physical paths, and the
// knowledge is repaired and validated once at turn end, which covers a write
// through `bash` or a patch as well as one through a tool.
//
// This file is history. Nothing loads it, nothing builds it, and no test
// references it. It exists so `contract-012-api-sokf-pi-file-tools`, now
// deprecated, keeps materialising its Definition from source rather than from
// a hand copy (ADR-042). Only the three routed file tools are preserved here;
// `sokf_search`, `sokf_graph`, and `sokf_overview` remain live in
// `.pi/extensions/sokf.ts` and are bound by contract-013.

	// sokf:begin tools
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
	// sokf:end tools

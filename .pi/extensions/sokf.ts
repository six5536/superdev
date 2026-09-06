import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, isAbsolute, relative, resolve } from "node:path";
import {
	createEditTool,
	createReadTool,
	createWriteTool,
	DEFAULT_MAX_BYTES,
	DEFAULT_MAX_LINES,
	formatSize,
	truncateHead,
	type EditToolDetails,
	type ExtensionAPI,
	withFileMutationQueue,
} from "@earendil-works/pi-coding-agent";
import { type Static, Type } from "typebox";

const PROTOCOL = "sokf-tools/v1";

const searchSchema = Type.Object({
	query: Type.String({ description: "What to find in the canonical project knowledge" }),
	limit: Type.Optional(Type.Integer({ minimum: 1, description: "Most sections to return" })),
	types: Type.Optional(Type.Array(Type.String({ description: "Concept type" }))),
	tags: Type.Optional(Type.Array(Type.String({ description: "Concept tag" }))),
	lifecycle: Type.Optional(Type.Array(Type.String({ description: "Lifecycle value" }))),
});

const graphSchema = Type.Object({
	id: Type.Optional(Type.String({ description: "Concept ID; omit for the complete edge map" })),
});

type SearchInput = Static<typeof searchSchema>;
type GraphInput = Static<typeof graphSchema>;

type TextContent = { type: "text"; text: string };
type ToolEnvelope = {
	protocol: string;
	content: TextContent[];
	details: unknown;
};
type MutationChange = {
	path: string;
	source: "requested" | "repair";
	diff: string;
};
type MutationDetails = {
	applied: boolean;
	validation: "valid" | "invalid" | "unknown";
	resolvedPath: string;
	finalPath: string;
	changes: MutationChange[];
	findings: Array<{ severity: string; path?: string; message: string }>;
};

function stripAt(path: string): string {
	return path.startsWith("@") ? path.slice(1) : path;
}

function isSokfAddress(path: string): boolean {
	return stripAt(path).startsWith("sokf:");
}

function readInvocation(address: string): string[] {
	const target = stripAt(address).slice("sokf:".length);
	if (target.length === 0) return ["sokf", "overview", "--json"];
	const separator = target.indexOf("#");
	if (separator < 0) return ["sokf", "read", target, "--json"];
	const id = target.slice(0, separator);
	const heading = target.slice(separator + 1);
	if (!id || !heading) throw new Error(`Invalid SOKF address: ${address}`);
	return ["sokf", "read", id, "--heading", heading, "--json"];
}

function findRepository(start: string): string | undefined {
	let current = resolve(start);
	while (true) {
		if (existsSync(resolve(current, ".git")) || existsSync(resolve(current, ".superdev", "config.toml"))) {
			return current;
		}
		const parent = dirname(current);
		if (parent === current) return undefined;
		current = parent;
	}
}

function knowledgeRoot(cwd: string): string | undefined {
	const repository = findRepository(cwd);
	if (!repository) return undefined;
	const knowledge = resolve(repository, "knowledge");
	return existsSync(knowledge) ? knowledge : undefined;
}

function physicalTarget(path: string, cwd: string, repository: string): string {
	const clean = stripAt(path);
	if (isAbsolute(clean)) return resolve(clean);
	if (clean === "knowledge" || clean.startsWith("knowledge/")) {
		return resolve(repository, clean);
	}
	return resolve(cwd, clean);
}

function isWithin(root: string, target: string): boolean {
	const child = relative(root, target);
	return child === "" || (!child.startsWith("..") && !isAbsolute(child));
}

function routesMutation(path: string, cwd: string): { route: boolean; queue: string; target: string } {
	const repository = findRepository(cwd);
	const knowledge = repository ? knowledgeRoot(repository) : undefined;
	if (isSokfAddress(path)) {
		if (!knowledge) throw new Error("No SOKF knowledge found from the current directory");
		return { route: true, queue: knowledge, target: stripAt(path) };
	}
	if (!repository || !knowledge) return { route: false, queue: "", target: path };
	const target = physicalTarget(path, cwd, repository);
	return {
		route: isWithin(knowledge, target),
		queue: knowledge,
		target,
	};
}

function parseEnvelope(stdout: string): ToolEnvelope {
	let envelope: ToolEnvelope;
	try {
		envelope = JSON.parse(stdout) as ToolEnvelope;
	} catch (error) {
		throw new Error(`superdev returned malformed JSON: ${String(error)}`);
	}
	if (envelope.protocol !== PROTOCOL) {
		throw new Error(`Incompatible superdev SOKF protocol: expected ${PROTOCOL}, got ${String(envelope.protocol)}`);
	}
	if (!Array.isArray(envelope.content) || !envelope.content.every((item) => item.type === "text")) {
		throw new Error("superdev returned an invalid SOKF content shape");
	}
	return envelope;
}

async function invoke(
	cwd: string,
	args: string[],
	signal?: AbortSignal,
	request?: unknown,
): Promise<ToolEnvelope> {
	const result = await new Promise<{ stdout: string; stderr: string; code: number | null }>((accept, reject) => {
		const child = spawn("superdev", args, {
			cwd: findRepository(cwd) ?? cwd,
			signal,
			stdio: ["pipe", "pipe", "pipe"],
		});
		let stdout = "";
		let stderr = "";
		child.stdout.setEncoding("utf8");
		child.stderr.setEncoding("utf8");
		child.stdout.on("data", (chunk: string) => {
			stdout += chunk;
		});
		child.stderr.on("data", (chunk: string) => {
			stderr += chunk;
		});
		child.on("error", reject);
		child.on("close", (code) => accept({ stdout, stderr, code }));
		child.stdin.end(request === undefined ? undefined : JSON.stringify(request));
	});
	if (result.code !== 0) {
		throw new Error(result.stderr.trim() || result.stdout.trim() || `superdev exited ${String(result.code)}`);
	}
	return parseEnvelope(result.stdout);
}

function textOf(envelope: ToolEnvelope): string {
	return envelope.content.map((item) => item.text).join("\n");
}

function boundedResult(envelope: ToolEnvelope) {
	const text = textOf(envelope);
	const truncation = truncateHead(text, {
		maxLines: DEFAULT_MAX_LINES,
		maxBytes: DEFAULT_MAX_BYTES,
	});
	let output = truncation.content;
	if (truncation.truncated) {
		output += `\n\n[Output truncated: ${truncation.outputLines} of ${truncation.totalLines} lines (${formatSize(truncation.outputBytes)} of ${formatSize(truncation.totalBytes)}). Narrow the query or graph target.]`;
	}
	return {
		content: [{ type: "text" as const, text: output }],
		details: envelope.details,
	};
}

function mutationDetails(envelope: ToolEnvelope): MutationDetails {
	const details = envelope.details as Partial<MutationDetails>;
	if (
		details.applied !== true ||
		!Array.isArray(details.changes) ||
		!Array.isArray(details.findings) ||
		typeof details.resolvedPath !== "string" ||
		typeof details.finalPath !== "string"
	) {
		throw new Error("superdev returned an invalid SOKF mutation result");
	}
	return details as MutationDetails;
}

function displayDiff(patch: string): string {
	const output: string[] = [];
	let oldLine = 1;
	let newLine = 1;
	for (const line of patch.split("\n")) {
		const hunk = line.match(/^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
		if (hunk) {
			oldLine = Number(hunk[1]);
			newLine = Number(hunk[2]);
			continue;
		}
		if (line.startsWith("--- ") || line.startsWith("+++ ") || line.length === 0) continue;
		if (line.startsWith("-")) {
			output.push(`-${oldLine} ${line.slice(1)}`);
			oldLine += 1;
		} else if (line.startsWith("+")) {
			output.push(`+${newLine} ${line.slice(1)}`);
			newLine += 1;
		} else if (line.startsWith(" ")) {
			output.push(` ${newLine} ${line.slice(1)}`);
			oldLine += 1;
			newLine += 1;
		}
	}
	return output.join("\n");
}

function editDetails(details: MutationDetails): EditToolDetails {
	const patch = details.changes.map((change) => change.diff).join("\n");
	const first = patch.match(/^@@ -\d+(?:,\d+)? \+(\d+)/m);
	return {
		diff: displayDiff(patch),
		patch,
		firstChangedLine: first ? Number(first[1]) : undefined,
	};
}

export default function (pi: ExtensionAPI) {
	pi.registerTool({
		name: "read",
		label: "read",
		description:
			"Read files and SOKF project knowledge. Use path sokf: for an overview, sokf:<id> for a concept, or sokf:<id>#<heading> for a section.",
		promptSnippet: "Read file contents or canonical project knowledge via sokf:<id>",
		promptGuidelines: [
			"Use read with a sokf:<id> path when a known SOKF concept can answer the project question.",
			"Use read for ordinary files exactly as usual.",
		],
		parameters: createReadTool(process.cwd()).parameters,
		async execute(toolCallId, params, signal, onUpdate, ctx) {
			if (!isSokfAddress(params.path)) {
				return createReadTool(ctx.cwd).execute(toolCallId, params, signal, onUpdate);
			}
			const envelope = await invoke(ctx.cwd, readInvocation(params.path), signal);
			const virtualRead = createReadTool(ctx.cwd, {
				operations: {
					access: async () => {},
					readFile: async () => Buffer.from(textOf(envelope), "utf8"),
					detectImageMimeType: async () => undefined,
				},
			});
			return virtualRead.execute(toolCallId, params, signal, onUpdate);
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
			const args = ["sokf", "search", params.query, "--json"];
			if (params.limit !== undefined) args.push("--limit", String(params.limit));
			for (const type of params.types ?? []) args.push("--type", type);
			for (const tag of params.tags ?? []) args.push("--tag", tag);
			for (const lifecycle of params.lifecycle ?? []) args.push("--lifecycle", lifecycle);
			return boundedResult(await invoke(ctx.cwd, args, signal));
		},
	});

	pi.registerTool({
		name: "sokf_graph",
		label: "SOKF graph",
		description: "Follow typed relationships in the canonical SOKF project knowledge, or show its complete edge map.",
		promptSnippet: "Traverse relationships in the canonical SOKF project knowledge",
		parameters: graphSchema,
		async execute(_toolCallId, params: GraphInput, signal, _onUpdate, ctx) {
			const args = ["sokf", "graph"];
			if (params.id) args.push(params.id);
			args.push("--json");
			return boundedResult(await invoke(ctx.cwd, args, signal));
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
				const envelope = await invoke(
					ctx.cwd,
					["sokf", "edit", "--request-json", "-", "--json"],
					signal,
					{ ...params, path: route.target },
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
				const envelope = await invoke(
					ctx.cwd,
					["sokf", "write", "--request-json", "-", "--json"],
					signal,
					{ ...params, path: route.target },
				);
				mutationDetails(envelope);
				return { content: envelope.content, details: undefined };
			});
		},
	});
}

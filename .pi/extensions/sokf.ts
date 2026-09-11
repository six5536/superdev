import { spawn } from "node:child_process";
import { constants, existsSync } from "node:fs";
import { access } from "node:fs/promises";
import { homedir } from "node:os";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
	createEditTool,
	createReadTool,
	createReadToolDefinition,
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

import { SokfMcpClient, type McpToolResult, type SourceResolution } from "./sokf-mcp.ts";

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

/**
 * The canonical active checkout root above `start`.
 *
 * The markers are ranked, not interchangeable. A `.git` entry — a directory
 * in a normal checkout, a pointer file in a linked worktree — anywhere on the
 * upward walk always wins, so Pi's working-directory scope and SOKF's
 * containment boundary are the same checkout. Only when the walk finds no
 * `.git` marker at all does `.superdev/config.toml` select the root, which
 * keeps a managed non-Git repository routing as it does today.
 */
export function findRepository(start: string): string | undefined {
	let current = resolve(start);
	let configured: string | undefined;
	while (true) {
		if (existsSync(resolve(current, ".git"))) return current;
		if (configured === undefined && existsSync(resolve(current, ".superdev", "config.toml"))) {
			configured = current;
		}
		const parent = dirname(current);
		if (parent === current) return configured;
		current = parent;
	}
}

// Pi prepares a path argument before its read operations ever see it, and
// exports no seam that performs that preparation alone. These four functions
// are a narrow adapter for the values observed at the Pi 0.85.1 factory
// operation boundary, and the paired harness proves them byte-for-byte
// against it rather than trusting the copy.
const UNICODE_SPACES = /[\u00A0\u2000-\u200A\u202F\u205F\u3000]/g;
const NARROW_NO_BREAK_SPACE = "\u202F";

type NormalizeOptions = { normalizeUnicodeSpaces?: boolean; stripAtPrefix?: boolean };

function normalizeWindowsShellPath(filePath: string): string {
	if (!filePath.startsWith("/") || filePath.startsWith("//") || filePath.includes("\\")) return filePath;
	const match = filePath.match(/^\/(?:mnt\/|cygdrive\/)?([a-z])(?:\/(.*))?$/i);
	if (!match) return filePath;
	const suffix = match[2]?.replaceAll("/", "\\");
	return `${match[1].toUpperCase()}:\\${suffix ?? ""}`;
}

function normalizePath(input: string, options: NormalizeOptions = {}): string {
	let normalized = input;
	if (options.normalizeUnicodeSpaces) normalized = normalized.replace(UNICODE_SPACES, " ");
	if (options.stripAtPrefix && normalized.startsWith("@")) normalized = normalized.slice(1);
	if (process.platform === "win32") normalized = normalizeWindowsShellPath(normalized);
	const home = homedir();
	if (normalized === "~") return home;
	if (normalized.startsWith("~/") || (process.platform === "win32" && normalized.startsWith("~\\"))) {
		return join(home, normalized.slice(2));
	}
	if (/^file:\/\//.test(normalized)) return fileURLToPath(normalized);
	return normalized;
}

function resolveToCwd(filePath: string, cwd: string): string {
	const normalized = normalizePath(filePath, { normalizeUnicodeSpaces: true, stripAtPrefix: true });
	const base = normalizePath(cwd);
	return isAbsolute(normalized) ? resolve(normalized) : resolve(base, normalized);
}

async function pathExists(filePath: string): Promise<boolean> {
	try {
		await access(filePath, constants.F_OK);
		return true;
	} catch {
		return false;
	}
}

/** The absolute path Pi's read factory would prepare from `filePath`. */
export async function prepareReadPath(filePath: string, cwd: string): Promise<string> {
	const resolved = resolveToCwd(filePath, cwd);
	if (await pathExists(resolved)) return resolved;
	const amPm = resolved.replace(/ (AM|PM)\./gi, `${NARROW_NO_BREAK_SPACE}$1.`);
	if (amPm !== resolved && (await pathExists(amPm))) return amPm;
	const nfd = resolved.normalize("NFD");
	if (nfd !== resolved && (await pathExists(nfd))) return nfd;
	const curly = resolved.replace(/'/g, "\u2019");
	if (curly !== resolved && (await pathExists(curly))) return curly;
	const nfdCurly = nfd.replace(/'/g, "\u2019");
	if (nfdCurly !== resolved && (await pathExists(nfdCurly))) return nfdCurly;
	return resolved;
}

function knowledgeRoot(cwd: string): string | undefined {
	const repository = findRepository(cwd);
	if (!repository) return undefined;
	const knowledge = resolve(repository, "knowledge");
	return existsSync(knowledge) ? knowledge : undefined;
}

/** The machine result of `sokf_resolve_source`, checked before it is trusted. */
function sourceResolution(envelope: ToolEnvelope): SourceResolution {
	const details = envelope.details as Partial<SourceResolution>;
	if (
		typeof details.ingressPath !== "string" ||
		typeof details.canonicalPath !== "string" ||
		typeof details.exists !== "boolean" ||
		!Array.isArray(details.generatedRegions)
	) {
		throw new Error("superdev returned an invalid SOKF source resolution");
	}
	return details as SourceResolution;
}

/**
 * Pi's model-visible read notices quote the path they were given. Exactly one
 * does — the oversized-first-line notice — and Pi sets
 * `truncation.firstLineExceedsLimit` when it emits it, so that flag gates the
 * rewrite rather than the notice's wording. Without the flag no byte is
 * touched, so file content that merely looks like a notice is never rewritten.
 */
function restoreSpelling(
	result: { content: Array<{ type: string; text?: string }>; details?: unknown },
	target: string,
	asked: string,
) {
	const truncation = (result.details as { truncation?: { firstLineExceedsLimit?: boolean } } | undefined)
		?.truncation;
	if (!truncation?.firstLineExceedsLimit) return result;
	const content = result.content.map((item) =>
		item.type === "text" && typeof item.text === "string" && item.text.includes(target)
			? { ...item, text: item.text.replaceAll(target, asked) }
			: item,
	);
	return { ...result, content };
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

/** The active checkout root, or the error a caller can act on. */
function requireRepository(cwd: string): string {
	const repository = findRepository(cwd);
	if (!repository) throw new Error("No Superdev repository found from the current directory");
	return repository;
}

/**
 * Whether a physical read argument enters the knowledge tree.
 *
 * The candidate is prepared exactly as Pi would prepare it, so an argument
 * Pi resolves into `knowledge/` routes and one it resolves elsewhere is
 * Pi's alone. Containment of the canonical target is the resolver's answer,
 * not this one.
 */
async function routesRead(path: string, cwd: string): Promise<boolean> {
	const knowledge = knowledgeRoot(cwd);
	if (!knowledge) return false;
	return isWithin(knowledge, await prepareReadPath(path, cwd));
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

type ProcessResult = { stdout: string; stderr: string; code: number | null };

async function runSuperdev(
	cwd: string,
	args: string[],
	signal?: AbortSignal,
	request?: unknown,
): Promise<ProcessResult> {
	return new Promise<ProcessResult>((accept, reject) => {
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
}

function toolEnvelope(result: McpToolResult): ToolEnvelope {
	const content = result.content ?? [];
	if (!Array.isArray(content) || !content.every((item) => item.type === "text" && typeof item.text === "string")) {
		throw new Error("superdev returned an invalid SOKF MCP content shape");
	}
	const textContent = content as TextContent[];
	if (result.isError) {
		throw new Error(textContent.map((item) => item.text).join("\n") || "SOKF MCP tool failed");
	}
	return { content: textContent, details: result.structuredContent ?? {} };
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

function validationFeedback(result: ProcessResult): string {
	const raw = [result.stdout.trim(), result.stderr.trim()].filter(Boolean).join("\n");
	const report = raw || `superdev validate exited ${String(result.code)}`;
	const truncation = truncateHead(report, { maxLines: 200, maxBytes: 8_000 });
	return truncation.truncated
		? `${truncation.content}\n[Validation output truncated; run superdev validate for the complete report.]`
		: truncation.content;
}

export default function (pi: ExtensionAPI) {
	let knowledgeMutated = false;
	let validationFollowUps = 0;
	const maxValidationFollowUps = 2;
	const clients = new Map<string, SokfMcpClient>();

	async function invokeMcp(
		cwd: string,
		name: string,
		args: Record<string, unknown>,
		signal?: AbortSignal,
	): Promise<ToolEnvelope> {
		const repository = findRepository(cwd);
		if (!repository) throw new Error("No Superdev repository found from the current directory");
		let client = clients.get(repository);
		if (!client) {
			client = new SokfMcpClient(repository);
			clients.set(repository, client);
		}
		return toolEnvelope(await client.callTool(name, args, signal));
	}

	pi.on("session_shutdown", async () => {
		const closing = [...clients.values()].map((client) => client.close());
		clients.clear();
		await Promise.allSettled(closing);
	});

	pi.on("turn_end", async (_event, ctx) => {
		if (!knowledgeMutated) return;
		const result = await runSuperdev(ctx.cwd, ["validate"]);
		if (result.code === 0) {
			knowledgeMutated = false;
			validationFollowUps = 0;
			return;
		}

		const report = validationFeedback(result);
		if (validationFollowUps < maxValidationFollowUps) {
			validationFollowUps += 1;
			pi.sendMessage(
				{
					customType: "sokf-validation",
					content: `Final SOKF validation failed after a knowledge mutation. Fix these findings without retrying mutations that already report applied: true.\n\n${report}`,
					display: true,
					details: { attempt: validationFollowUps, maximum: maxValidationFollowUps },
				},
				{ deliverAs: "followUp", triggerTurn: true },
			);
			return;
		}

		knowledgeMutated = false;
		validationFollowUps = 0;
		pi.sendMessage(
			{
				customType: "sokf-validation",
				content: `SOKF validation still fails after ${maxValidationFollowUps} automatic repair turns. Manual continuation is required.\n\n${report}`,
				display: true,
			},
			{ deliverAs: "nextTurn" },
		);
		ctx.ui.notify("SOKF validation still fails; automatic repair feedback stopped", "error");
	});

	// The adapter contract's definition (contract-012): the five tool
	// registrations whose interfaces its promises govern.
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
				knowledgeMutated = true;
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
				knowledgeMutated = true;
				return { content: envelope.content, details: undefined };
			});
		},
	});
	// sokf:end tools
}

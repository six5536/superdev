import { spawn } from "node:child_process";
import { existsSync, realpathSync } from "node:fs";
import { dirname, resolve } from "node:path";
import {
	DEFAULT_MAX_BYTES,
	DEFAULT_MAX_LINES,
	formatSize,
	truncateHead,
	type ExtensionAPI,
} from "@earendil-works/pi-coding-agent";
import { type Static, Type } from "typebox";

import { SokfMcpClient, type McpToolResult } from "./sokf-mcp.ts";

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

// The knowledge is the whole subject, so the request narrows by nothing.
const overviewSchema = Type.Object({}, { additionalProperties: false });

type SearchInput = Static<typeof searchSchema>;
type GraphInput = Static<typeof graphSchema>;

type TextContent = { type: "text"; text: string };
type ToolEnvelope = {
	content: TextContent[];
	details: unknown;
};

/**
 * The canonical active checkout root above `start`.
 *
 * The markers are ranked, not interchangeable. A `.git` entry — a directory
 * in a normal checkout, a pointer file in a linked worktree — anywhere on the
 * upward walk always wins, so Pi's working-directory scope and SOKF's
 * containment boundary are the same checkout. Only when the walk finds no
 * `.git` marker at all does `.superdev/config.toml` select the root, which
 * keeps a managed non-Git repository discoverable.
 */
export function findRepository(start: string): string | undefined {
	let current = realpathSync(start);
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


function validationFeedback(report: string, first: boolean): string {
	const heading = first
		? "SOKF validation fails after automatic repair. Fix these findings."
		: "SOKF validation still fails after automatic repair.";
	const content = `${heading}\n\n${report}`;
	const notice = "\n[Validation output truncated; run superdev validate for the complete report.]";
	const truncation = truncateHead(content, { maxLines: 200, maxBytes: 8 * 1024 });
	if (!truncation.truncated) return content;
	// Reserve room for the notice inside the cap, not after it.
	return truncateHead(content, {
		maxLines: 199,
		maxBytes: 8 * 1024 - Buffer.byteLength(notice),
	}).content + notice;
}

/**
 * The current report of what the working tree still fails on, or `undefined`
 * when it now passes.
 *
 * This is a second read of the tree rather than a filter over the first
 * report. Repair, a later write, or a merge can settle a finding between the
 * run that produced it and the moment a message is sent, and reporting a
 * finding the tree no longer carries costs the agent a turn and teaches it to
 * distrust the channel.
 */
async function stillFailing(cwd: string): Promise<string | undefined> {
	const result = await runSuperdev(cwd, ["validate"]);
	if (result.code === 0) return undefined;
	return [result.stdout.trim(), result.stderr.trim()].filter(Boolean).join("\n")
		|| `superdev validate exited ${String(result.code)}`;
}

export default function (pi: ExtensionAPI) {
	// Whether a repository's pending sequence has already had its one
	// triggering report. Session memory keyed by canonical repository root:
	// nothing persists, so a new session starts with none and no lifecycle
	// event needs a recovery rule.
	const reported = new Set<string>();
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

	// sokf:begin turn-end
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
				content: validationFeedback(surviving, first),
				display: true,
			},
			// The first report of a sequence gets the agent one prompted chance to
			// correct. A later one is visible and leaves the decision to it: an
			// agent that reads a repeated report can check the tree itself.
			first ? { deliverAs: "followUp", triggerTurn: true } : { deliverAs: "nextTurn" },
		);
	});
	// sokf:end turn-end

	// The extension contract's definition (contract-013): the three tools a
	// session gets beyond Pi's own. `read`, `edit`, and `write` are Pi's,
	// unregistered here, so a knowledge path behaves as any other path.
	// sokf:begin tools
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

	// sokf:end tools
}

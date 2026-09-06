import { spawn } from "node:child_process";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type, type Static } from "typebox";

const here = dirname(fileURLToPath(import.meta.url));
const schema = Type.Object({
	role: Type.Union([
		Type.Literal("scope"),
		Type.Literal("requirements-review"),
		Type.Literal("build"),
		Type.Literal("code-review"),
		Type.Literal("accept"),
		Type.Literal("file"),
	]),
	task: Type.String({ description: "Bounded task and all input the isolated role needs" }),
});
type Input = Static<typeof schema>;

const readOnly = new Set(["requirements-review", "code-review"]);

async function isolated(
	role: Input["role"],
	task: string,
	cwd: string,
	model?: { provider: string; id: string },
	signal?: AbortSignal,
): Promise<string> {
	const promptPath = resolve(here, "prompts", `${role}.md`);
	await readFile(promptPath, "utf8");
	const args = ["--mode", "json", "-p", "--no-session", "--append-system-prompt", promptPath];
	if (model) args.push("--provider", model.provider, "--model", model.id);
	args.push("--tools", readOnly.has(role) ? "read,sokf_search,sokf_graph" : "read,bash,edit,write,sokf_search,sokf_graph");
	args.push(`Task: ${task}`);
	return new Promise((accept, reject) => {
		const child = spawn("pi", args, { cwd, shell: false, stdio: ["ignore", "pipe", "pipe"] });
		let stdout = "";
		let stderr = "";
		child.stdout.setEncoding("utf8");
		child.stderr.setEncoding("utf8");
		child.stdout.on("data", (chunk: string) => (stdout += chunk));
		child.stderr.on("data", (chunk: string) => (stderr += chunk));
		child.on("error", reject);
		child.on("close", (code) => {
			if (code !== 0) return reject(new Error(stderr.trim() || `${role} exited ${code}`));
			let answer = "";
			for (const line of stdout.split("\n")) {
				try {
					const event = JSON.parse(line);
					if (event.type === "message_end" && event.message?.role === "assistant") {
						for (const part of event.message.content ?? []) if (part.type === "text") answer = part.text;
					}
				} catch { /* ignore non-events */ }
			}
			accept(answer || "Isolated role completed without text output.");
		});
		if (signal) {
			const stop = () => child.kill("SIGTERM");
			if (signal.aborted) stop(); else signal.addEventListener("abort", stop, { once: true });
		}
	});
}

export default function superdev(pi: ExtensionAPI) {
	pi.registerTool({
		name: "superdev_isolated_role",
		label: "Superdev isolated role",
		description: "Run one extension-private workflow role in a fresh isolated Pi process",
		parameters: schema,
		execute: async (_id, input, signal, _update, ctx) => ({
			content: [{ type: "text", text: await isolated(input.role, input.task, ctx.cwd, ctx.model, signal) }],
			details: { role: input.role, isolated: true, readOnly: readOnly.has(input.role) },
		}),
	});

	const send = (text: string, ctx: { isIdle(): boolean; ui: { notify(message: string, level: "warning"): void } }) => {
		if (!ctx.isIdle()) return ctx.ui.notify("Superdev commands require an idle session", "warning");
		pi.sendUserMessage(text);
	};
	pi.registerCommand("superdev", {
		description: "Start or continue SCOPE → BUILD → ACCEPT",
		handler: async (args, ctx) => send(`${await readFile(resolve(here, "prompts/orchestrator.md"), "utf8")}\n\nUser request: ${args || "inspect status and resume the canonical workflow"}`, ctx),
	});
	pi.registerCommand("superdev-resume", {
		description: "Resume the canonical workflow after reconstructing Rust-owned state",
		handler: async (_args, ctx) => send("Run `superdev workflow status --json`, bind this Pi session if unowned, then resume the canonical phase. Do not infer completion from an absent cache.", ctx),
	});
	for (const [name, instruction] of [
		["scope", "Run SCOPE through the private scope role, isolated requirements review, and explicit human approval."],
		["build", "Run BUILD through the one modifying build child, complete verification, and a fresh read-only code review."],
		["accept", "Run ACCEPT under project-configured human policy and local Rust-owned integration."],
	] as const) {
		pi.registerCommand(name, {
			description: `${name.toUpperCase()} phase of the canonical workflow`,
			handler: async (args, ctx) => send(`${instruction} ${args}`.trim(), ctx),
		});
	}
	pi.registerCommand("superdev-cancel", {
		description: "Pause the workflow and release transient ownership",
		handler: async (_args, ctx) => {
			const session = ctx.sessionManager.getSessionId();
			const result = await pi.exec("superdev", ["workflow", "cancel", "--session", session], { cwd: ctx.cwd });
			ctx.ui.notify(result.code === 0 ? "Workflow paused; canonical phase unchanged" : result.stderr, result.code === 0 ? "info" : "error");
		},
	});
	pi.registerCommand("superdev-abandon", {
		description: "Human-only abandonment with knowledge disposition",
		handler: async (args, ctx) => {
			if (!ctx.hasUI || !(await ctx.ui.confirm("Abandon workflow?", "Partial product work will not be merged. Approved knowledge disposition must be recorded."))) return;
			send(`Human explicitly approved abandonment. Inspect workflow status, record the approved knowledge-only disposition, then invoke superdev workflow abandon with this session and expected revision. Reason: ${args || "not supplied"}`, ctx);
		},
	});
	pi.registerCommand("file", {
		description: "Capture an issue or idea outside the workflow",
		handler: async (args, ctx) => send(`Use superdev_isolated_role with role=file to capture, without scoping or implementation: ${args}`, ctx),
	});
}

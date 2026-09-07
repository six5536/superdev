import { spawn, type ChildProcess } from "node:child_process";
import { randomBytes } from "node:crypto";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { StringEnum } from "@earendil-works/pi-ai";
import { Type, type Static } from "typebox";

const here = dirname(fileURLToPath(import.meta.url));
const schema = Type.Object({
	role: StringEnum(["scope", "requirements-review", "build", "code-review", "accept", "file"] as const),
	task: Type.String({ description: "Bounded task and all input the isolated role needs" }),
	base: Type.Optional(Type.String({ description: "Immutable review base" })),
	candidate: Type.Optional(Type.String({ description: "Immutable review candidate" })),
});
type Input = Static<typeof schema>;

const readOnly = new Set(["requirements-review", "code-review"]);

type RoleResult = {
	status: "complete" | "clean" | "findings" | "rescope" | "rejected" | "duplicate" | "blocked";
	summary: string;
	findings?: string[];
};

export function parseRoleResult(role: Input["role"], answer: string): RoleResult {
	const marker = "SUPERDEV_RESULT ";
	const line = answer.split("\n").reverse().find((candidate) => candidate.startsWith(marker));
	if (!line) throw new Error(`${role} omitted its structured terminal result`);
	let result: RoleResult;
	try {
		result = JSON.parse(line.slice(marker.length)) as RoleResult;
	} catch {
		throw new Error(`${role} returned malformed terminal JSON`);
	}
	const allowed: Record<Input["role"], RoleResult["status"][]> = {
		scope: ["complete", "blocked"],
		"requirements-review": ["clean", "findings"],
		build: ["complete", "rescope", "blocked"],
		"code-review": ["clean", "findings"],
		accept: ["complete", "rejected", "blocked"],
		file: ["complete", "duplicate", "blocked"],
	};
	if (!allowed[role].includes(result.status) || typeof result.summary !== "string" || !result.summary.trim()) {
		throw new Error(`${role} returned an invalid terminal result`);
	}
	if (result.status === "findings" && (!Array.isArray(result.findings) || result.findings.length === 0)) {
		throw new Error(`${role} reported findings without structured findings`);
	}
	if (result.status === "clean" && result.findings?.length) {
		throw new Error(`${role} reported clean with findings`);
	}
	return result;
}

function stopProcess(child: ChildProcess) {
	const signal = (name: NodeJS.Signals) => {
		try {
			if (child.pid && process.platform !== "win32") process.kill(-child.pid, name);
			else child.kill(name);
		} catch { /* The child already exited. */ }
	};
	signal("SIGTERM");
	setTimeout(() => { if (child.exitCode === null) signal("SIGKILL"); }, 2_000).unref();
}

async function runSuperdev(args: string[], cwd: string, authority: string): Promise<unknown> {
	return new Promise((accept, reject) => {
		const child = spawn("superdev", args, {
			cwd,
			shell: false,
			stdio: ["ignore", "pipe", "pipe"],
			env: { ...process.env, SUPERDEV_UI_AUTHORITY: authority },
		});
		let stdout = "";
		let stderr = "";
		child.stdout.setEncoding("utf8");
		child.stderr.setEncoding("utf8");
		child.stdout.on("data", (chunk: string) => { stdout = (stdout + chunk).slice(-1_000_000); });
		child.stderr.on("data", (chunk: string) => { stderr = (stderr + chunk).slice(-100_000); });
		child.on("error", reject);
		child.on("close", (code) => {
			if (code !== 0) return reject(new Error(stderr.trim() || `superdev exited ${code}`));
			try { accept(JSON.parse(stdout)); } catch { reject(new Error("superdev returned invalid workflow JSON")); }
		});
	});
}

async function isolated(
	role: Input["role"],
	task: string,
	cwd: string,
	model?: { provider: string; id: string },
	signal?: AbortSignal,
	onSpawn?: (child: ChildProcess) => void,
	onClose?: (child: ChildProcess) => void,
	base?: string,
	candidate?: string,
): Promise<RoleResult> {
	const promptPath = resolve(here, "prompts", `${role}.md`);
	const rolePrompt = await readFile(promptPath, "utf8");
	if (readOnly.has(role) && (!base || !candidate || !/^[0-9a-f]{7,64}$/.test(base) || !/^[0-9a-f]{7,64}$/.test(candidate))) {
		throw new Error(`${role} requires immutable hexadecimal base and candidate revisions`);
	}
	const args = ["--mode", "json", "-p", "--no-session", "--approve", "--append-system-prompt", rolePrompt];
	if (model) args.push("--provider", model.provider, "--model", model.id);
	args.push("--tools", readOnly.has(role) ? "read,superdev_review_diff,sokf_search,sokf_graph" : role === "file" ? "read,sokf_search,sokf_graph" : "read,bash,edit,write,sokf_search,sokf_graph");
	args.push(`Task: ${task}`);
	return new Promise((accept, reject) => {
		const child = spawn("pi", args, {
			cwd,
			shell: false,
			detached: process.platform !== "win32",
			stdio: ["ignore", "pipe", "pipe"],
			env: {
				...process.env,
				SUPERDEV_CHILD_ROLE: role,
				...(base ? { SUPERDEV_REVIEW_BASE: base } : {}),
				...(candidate ? { SUPERDEV_REVIEW_CANDIDATE: candidate } : {}),
			},
		});
		onSpawn?.(child);
		let stdout = "";
		let stderr = "";
		const append = (current: string, chunk: string) => {
			const next = current + chunk;
			if (next.length > 1_000_000) {
				stopProcess(child);
				throw new Error(`${role} output exceeded 1 MB`);
			}
			return next;
		};
		child.stdout.setEncoding("utf8");
		child.stderr.setEncoding("utf8");
		child.stdout.on("data", (chunk: string) => {
			try { stdout = append(stdout, chunk); } catch (error) { reject(error); }
		});
		child.stderr.on("data", (chunk: string) => {
			try { stderr = append(stderr, chunk); } catch (error) { reject(error); }
		});
		child.on("error", reject);
		child.on("close", (code) => {
			onClose?.(child);
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
			try {
				accept(parseRoleResult(role, answer));
			} catch (error) {
				reject(error);
			}
		});
		if (signal) {
			const stop = () => stopProcess(child);
			if (signal.aborted) stop(); else signal.addEventListener("abort", stop, { once: true });
		}
	});
}

export default function superdev(pi: ExtensionAPI) {
	const childRole = process.env.SUPERDEV_CHILD_ROLE;
	pi.on("tool_call", (event) => {
		if (!childRole) return;
		if (readOnly.has(childRole) && ["bash", "edit", "write"].includes(event.toolName)) {
			return { block: true, reason: `${childRole} is an isolated read-only role`, terminate: true };
		}
		if (event.toolName === "bash") {
			const command = String((event.input as { command?: unknown }).command ?? "");
			if (/\bsuperdev\s+workflow\s+(?:record-evidence|transition|integrate|abandon)\b/.test(command)) {
				return { block: true, reason: "isolated roles cannot perform authoritative workflow transitions", terminate: true };
			}
		}
	});

	const authority = randomBytes(32).toString("hex");
	const reviewRuns = new Map<string, { role: "requirements-review" | "code-review"; result: RoleResult; base: string; candidate: string }>();
	const children = new Set<ChildProcess>();
	let modifyingChild: ChildProcess | undefined;
	let modifyingBusy = false;
	const stopChild = stopProcess;

	pi.registerTool({
		name: "superdev_review_diff",
		label: "Superdev review diff",
		description: "Read an immutable Git diff for an isolated reviewer",
		parameters: Type.Object({
			base: Type.String(),
			candidate: Type.String(),
		}),
		execute: async (_id, input: { base: string; candidate: string }, _signal, _update, ctx) => {
			if (childRole && readOnly.has(childRole) && (
				input.base !== process.env.SUPERDEV_REVIEW_BASE
				|| input.candidate !== process.env.SUPERDEV_REVIEW_CANDIDATE
			)) throw new Error("review diff revisions differ from the orchestrator-bound candidate");
			for (const revision of [input.base, input.candidate]) {
				if (!/^[0-9a-f]{7,64}$/.test(revision)) throw new Error("review revisions must be hexadecimal object IDs");
			}
			const result = await pi.exec("git", ["diff", "--no-ext-diff", "--unified=80", input.base, input.candidate, "--"], { cwd: ctx.cwd });
			if (result.code !== 0) throw new Error(result.stderr.trim() || "git diff failed");
			if (result.stdout.length > 900_000) throw new Error("review diff exceeds the 900 KB review bound");
			return { content: [{ type: "text", text: result.stdout || "No changes." }], details: { readOnly: true } };
		},
	});
	if (childRole) return;

	type WorkflowStatus = {
		owner?: { session_id?: string; identity?: { plan?: string } };
		phase?: "scope" | "build" | "accept";
	};
	const workflowStatus = async (cwd: string): Promise<WorkflowStatus> => {
		const result = await pi.exec("superdev", ["workflow", "status", "--json"], { cwd });
		if (result.code !== 0) return {};
		try {
			return JSON.parse(result.stdout).result as WorkflowStatus;
		} catch {
			return {};
		}
	};
	const workflowOwner = async (cwd: string) => (await workflowStatus(cwd)).owner;
	const protectOwnedWorkflow = async (ctx: { cwd: string; ui: { notify(message: string, level: "warning"): void } }) => {
		const owner = await workflowOwner(ctx.cwd);
		if (!owner) return;
		ctx.ui.notify(`Cancel or finish ${owner.identity?.plan ?? "the owned workflow"} before changing sessions`, "warning");
		return { cancel: true };
	};
	pi.on("session_before_switch", async (_event, ctx) => protectOwnedWorkflow(ctx));
	pi.on("session_before_fork", async (_event, ctx) => protectOwnedWorkflow(ctx));
	pi.on("session_start", async (_event, ctx) => {
		const status = await workflowStatus(ctx.cwd);
		ctx.ui.setStatus(
			"superdev-workflow",
			status.owner ? `${status.phase?.toUpperCase() ?? "WORKFLOW"}: ${status.owner.identity?.plan ?? "owned"}` : undefined,
		);
	});

	pi.registerTool({
		name: "superdev_workflow_control",
		label: "Superdev workflow control",
		description: "Invoke one typed Rust workflow operation; human-gated actions confirm in trusted Pi UI",
		parameters: Type.Object({
			action: StringEnum(["start", "resume", "record-scope-review", "record-final-evidence", "approve-scope", "return-to-scope", "finish-build", "reject-acceptance", "accept", "abandon"] as const),
			session: Type.String(),
			issue: Type.Optional(Type.String()),
			plan: Type.Optional(Type.String()),
			workBranch: Type.Optional(Type.String()),
			defaultBranch: Type.Optional(Type.String()),
			expectedRevision: Type.Optional(Type.String()),
			reviewRun: Type.Optional(Type.String()),
			candidate: Type.Optional(Type.String()),
			phase: Type.Optional(StringEnum(["scope", "build", "accept"] as const)),
			feedback: Type.Optional(Type.String()),
			reason: Type.Optional(Type.String()),
		}),
		execute: async (_id, input, _signal, _update, ctx) => {
			const humanAction = input.action === "approve-scope" || input.action === "reject-acceptance" || input.action === "abandon";
			let acceptanceRequiresHuman = false;
			if (input.action === "accept") {
				const config = await readFile(resolve(ctx.cwd, ".superdev/config.toml"), "utf8");
				acceptanceRequiresHuman = !/^human_acceptance_required\s*=\s*false\s*$/m.test(config);
			}
			if ((humanAction || acceptanceRequiresHuman) && (!ctx.hasUI || !(await ctx.ui.confirm(
				`${input.action}?`,
				input.action === "abandon" ? "Partial product work will not be integrated." : "This records an authoritative human workflow decision.",
			)))) throw new Error("human workflow decision was not approved");
			const args = ["workflow"];
			if (input.action === "start" || input.action === "resume") {
				if (!input.issue || !input.plan || !input.workBranch) throw new Error("workflow identity is incomplete");
				args.push(input.action, "--session", input.session, "--issue", input.issue, "--plan", input.plan, "--work-branch", input.workBranch);
				if (input.defaultBranch) args.push("--default-branch", input.defaultBranch);
			} else if (input.action === "record-scope-review" || input.action === "record-final-evidence") {
				if (!input.expectedRevision || !input.reviewRun) throw new Error("evidence compare-and-swap fields are incomplete");
				const review = reviewRuns.get(input.reviewRun);
				const expectedRole = input.action === "record-scope-review" ? "requirements-review" : "code-review";
				if (!review || review.role !== expectedRole || review.result.status !== "clean") throw new Error("evidence does not name a clean bound review run");
				if (input.action === "record-final-evidence" && (!input.candidate || input.candidate !== review.candidate)) throw new Error("final evidence candidate differs from the reviewed candidate");
				args.push("evidence", "--session", input.session, "--expected-revision", input.expectedRevision, "--kind", input.action === "record-scope-review" ? "scope-review" : "final", "--review-session", input.reviewRun);
				if (input.candidate) args.push("--candidate", input.candidate);
			} else if (input.action === "abandon") {
				if (!input.expectedRevision || !input.phase || !input.reason?.trim()) throw new Error("abandonment disposition is incomplete");
				args.push("abandon", "--session", input.session, "--expected-revision", input.expectedRevision, "--phase", input.phase, "--reason", input.reason);
			} else {
				if (!input.expectedRevision || !input.phase) throw new Error("transition compare-and-swap fields are incomplete");
				args.push("transition", "--session", input.session, "--expected-revision", input.expectedRevision, "--phase", input.phase, "--transition", input.action);
				if (input.action === "reject-acceptance") {
					if (!input.feedback?.trim()) throw new Error("rejection feedback is required");
					args.push("--feedback", input.feedback);
				}
			}
			const result = await runSuperdev(args, ctx.cwd, authority);
			if ((input.action === "record-scope-review" || input.action === "record-final-evidence") && input.reviewRun) reviewRuns.delete(input.reviewRun);
			return { content: [{ type: "text", text: JSON.stringify(result) }], details: { result, humanGated: humanAction || acceptanceRequiresHuman } };
		},
	});

	pi.registerTool({
		name: "superdev_isolated_role",
		label: "Superdev isolated role",
		description: "Run one extension-private workflow role in a fresh isolated Pi process",
		parameters: schema,
		execute: async (_id, input, signal, _update, ctx) => {
			const modifying = !readOnly.has(input.role) && input.role !== "file";
			if (modifying && (modifyingBusy || modifyingChild)) throw new Error("one modifying workflow child is already active");
			if (modifying) modifyingBusy = true;
			try {
				const result = await isolated(
					input.role,
					input.task,
					ctx.cwd,
					ctx.model,
					signal,
					(child) => {
						children.add(child);
						if (modifying) modifyingChild = child;
					},
					(child) => {
						children.delete(child);
						if (modifyingChild === child) modifyingChild = undefined;
					},
					input.base,
					input.candidate,
				);
				let reviewRun: string | undefined;
				if (input.role === "requirements-review" || input.role === "code-review") {
					reviewRun = randomBytes(24).toString("hex");
					reviewRuns.set(reviewRun, { role: input.role, result, base: input.base!, candidate: input.candidate! });
				}
				const terminal = { ...result, ...(reviewRun ? { reviewRun } : {}) };
				return {
					content: [{ type: "text", text: JSON.stringify(terminal) }],
					details: { role: input.role, result, reviewRun, isolated: true, readOnly: readOnly.has(input.role) },
				};
			} finally {
				if (modifying) modifyingBusy = false;
			}
		},
	});

	const send = (text: string, ctx: { isIdle(): boolean; ui: { notify(message: string, level: "warning"): void } }) => {
		if (!ctx.isIdle()) return ctx.ui.notify("Superdev commands require an idle session", "warning");
		pi.sendUserMessage(text);
	};
	pi.registerCommand("superdev", {
		description: "Start or continue SCOPE → BUILD → ACCEPT",
		handler: async (args, ctx) => send(`${await readFile(resolve(here, "prompts/orchestrator.md"), "utf8")}\n\nUser request: ${args || "inspect status and resume the canonical workflow"}`, ctx),
	});
	pi.registerCommand("superdev-status", {
		description: "Show canonical workflow phase and transient ownership",
		handler: async (_args, ctx) => {
			const result = await pi.exec("superdev", ["workflow", "status", "--json"], { cwd: ctx.cwd });
			ctx.ui.notify(result.code === 0 ? result.stdout.trim() : result.stderr.trim(), result.code === 0 ? "info" : "error");
		},
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
			const stopping = [...children];
			for (const child of stopping) stopChild(child);
			await Promise.race([
				Promise.all(stopping.map((child) => new Promise<void>((done) => child.once("close", () => done())))),
				new Promise<void>((done) => setTimeout(done, 2_500)),
			]);
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
		handler: async (args, ctx) => send(
			`Use superdev_isolated_role with role=file to search for duplicates and prepare one bounded issue/idea filing. Present the title, description, kind, and target default branch. Ask the human to confirm. Only after confirmation invoke \`superdev file --human-approved\` with separate argument-array values; never interpolate text into a shell command. Request: ${args}`,
			ctx,
		),
	});
}

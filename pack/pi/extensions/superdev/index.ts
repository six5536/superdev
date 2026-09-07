import { spawn, type ChildProcess } from "node:child_process";
import { createHash, randomBytes } from "node:crypto";
import { constants } from "node:fs";
import { access, chmod, mkdtemp, open, readFile, rm, type FileHandle } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, dirname, isAbsolute, join, resolve } from "node:path";
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

let parentServicePath: string | undefined;
let parentServiceDigest: string | undefined;
let parentServicePin: Promise<{ path: string; digest: string }> | undefined;

async function resolveServicePath(): Promise<string> {
	for (const directory of (process.env.PATH ?? "").split(delimiter)) {
		if (!directory) continue;
		const candidate = resolve(directory, "superdev");
		try { await access(candidate, constants.X_OK); return candidate; } catch { /* Try the next PATH entry. */ }
	}
	throw new Error("superdev executable was not found on PATH");
}

async function ensureParentService(cwd: string): Promise<{ path: string; digest: string }> {
	if (!parentServicePin) {
		parentServicePin = (async () => {
			const path = await resolveServicePath();
			const probe = await runPinnedSuperdev(path, undefined, ["workflow", "status", "--json"], cwd);
			if (probe.code !== 0) throw new Error(probe.stderr.trim() || "superdev status probe failed");
			parentServicePath = path;
			parentServiceDigest = probe.digest;
			return { path, digest: probe.digest };
		})().catch((error) => { parentServicePin = undefined; throw error; });
	}
	return parentServicePin;
}

async function runSuperdev(args: string[], cwd: string, authority: string): Promise<unknown> {
	const service = await ensureParentService(cwd);
	const result = await runPinnedSuperdev(service.path, service.digest, args, cwd, undefined, { SUPERDEV_UI_AUTHORITY: authority });
	if (result.code !== 0) throw new Error(result.stderr.trim() || `superdev exited ${result.code}`);
	try { return JSON.parse(result.stdout); } catch { throw new Error("superdev returned invalid workflow JSON"); }
}

export function isolatedTools(role: Input["role"]): string {
	if (readOnly.has(role)) return "read,superdev_review_diff,sokf_search,sokf_graph";
	if (role === "file") return "read,sokf_search,sokf_graph";
	if (role === "build") return "read,edit,write,sokf_search,sokf_graph,superdev_build_exec";
	return "read,edit,write,sokf_search,sokf_graph";
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
	trustedExecutable?: string,
	trustedExecutableSha256?: string,
): Promise<RoleResult> {
	const promptPath = resolve(here, "prompts", `${role}.md`);
	const rolePrompt = await readFile(promptPath, "utf8");
	if (readOnly.has(role) && (!base || !candidate || !/^[0-9a-f]{7,64}$/.test(base) || !/^[0-9a-f]{7,64}$/.test(candidate))) {
		throw new Error(`${role} requires immutable hexadecimal base and candidate revisions`);
	}
	if (trustedExecutable && !trustedExecutableSha256) throw new Error("trusted executable digest was not supplied");
	const args = ["--mode", "json", "-p", "--no-session", "--approve", "--append-system-prompt", rolePrompt];
	if (model) args.push("--provider", model.provider, "--model", model.id);
	args.push("--tools", isolatedTools(role));
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
				...(trustedExecutable ? { SUPERDEV_TRUSTED_EXECUTABLE: trustedExecutable } : {}),
				...(trustedExecutableSha256 ? { SUPERDEV_TRUSTED_EXECUTABLE_SHA256: trustedExecutableSha256 } : {}),
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

export function isolatedRoleMayNotRun(command: string): boolean {
	return /\bsuperdev\s+workflow\s+(?:start|resume|cancel|record-evidence|evidence|scope-checkpoint|sync|correction(?!-checkpoint)|transition|integrate|abandon)\b/.test(command)
		|| /\bgit\s+(?:add|commit|update-ref|reset|switch|checkout|merge|rebase|cherry-pick|branch|tag|stash|clean|restore|rm|mv)\b/.test(command);
}

export function buildCommandAllowed(command: string, args: string[]): boolean {
	return command === "superdev"
		&& args[0] === "workflow"
		&& ["block", "attempt", "correction-checkpoint", "status"].includes(args[1] ?? "");
}

type ExecResult = { code: number; stdout: string; stderr: string };
type BuildExec = (command: string, args: string[], cwd: string) => Promise<ExecResult>;

export async function runPinnedSuperdev(path: string, digest: string | undefined, args: string[], cwd: string, signal?: AbortSignal, environment?: Record<string, string>): Promise<ExecResult & { digest: string }> {
	if (process.platform !== "linux") throw new Error("pinned BUILD service execution currently requires Linux /proc file descriptors");
	const source = await open(path, "r");
	let bytes: Buffer;
	try { bytes = await source.readFile(); } finally { await source.close(); }
	const observed = createHash("sha256").update(bytes).digest("hex");
	if (digest && observed !== digest) throw new Error("BUILD service executable changed after the parent pinned it");

	const directory = await mkdtemp(join(tmpdir(), "superdev-exec-"));
	const privatePath = join(directory, "service");
	let executable: FileHandle | undefined;
	try {
		const writer = await open(privatePath, "wx", 0o600);
		try { await writer.writeFile(bytes); await writer.sync(); } finally { await writer.close(); }
		await chmod(privatePath, 0o500);
		executable = await open(privatePath, "r");
		await rm(privatePath);
		const result = await new Promise<ExecResult>((accept, reject) => {
			const child = spawn("/proc/self/fd/3", args, {
				cwd,
				shell: false,
				stdio: ["ignore", "pipe", "pipe", executable!.fd],
				env: environment ? { ...process.env, ...environment } : process.env,
			});
			let stdout = "";
			let stderr = "";
			child.stdout.setEncoding("utf8");
			child.stderr.setEncoding("utf8");
			child.stdout.on("data", (chunk: string) => { stdout = (stdout + chunk).slice(-1_000_000); });
			child.stderr.on("data", (chunk: string) => { stderr = (stderr + chunk).slice(-100_000); });
			let killTimer: NodeJS.Timeout | undefined;
			const stop = () => {
				child.kill("SIGTERM");
				killTimer = setTimeout(() => { if (child.exitCode === null) child.kill("SIGKILL"); }, 2_000);
				killTimer.unref();
			};
			const cleanup = () => {
				if (killTimer) clearTimeout(killTimer);
				signal?.removeEventListener("abort", stop);
			};
			child.on("error", (error) => { cleanup(); reject(error); });
			child.on("close", (code) => { cleanup(); accept({ code: code ?? 2, stdout, stderr }); });
			if (signal) {
				if (signal.aborted) stop(); else signal.addEventListener("abort", stop, { once: true });
			}
		});
		return { ...result, digest: observed };
	} finally {
		await executable?.close();
		await rm(directory, { recursive: true, force: true });
	}
}

export async function runGuardedBuildCommand(exec: BuildExec, command: string, args: string[], cwd: string): Promise<ExecResult> {
	if (!buildCommandAllowed(command, args)) throw new Error(`BUILD executable or operation is not permitted: ${command}`);
	return exec(command, args, cwd);
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
			if (isolatedRoleMayNotRun(command)) {
				return { block: true, reason: "isolated roles cannot perform authoritative workflow transitions", terminate: true };
			}
		}
	});

	const authority = randomBytes(32).toString("hex");
	const reviewRuns = new Map<string, { role: "requirements-review" | "code-review"; result: RoleResult; base: string; candidate: string }>();
	const children = new Set<ChildProcess>();
	let modifyingChild: ChildProcess | undefined;
	let modifyingBusy = false;
	let modifyingCommandAbort: AbortController | undefined;
	let cancelling = false;
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
	pi.registerTool({
		name: "superdev_build_exec",
		label: "Superdev BUILD command",
		description: "Run one BUILD-owned Rust checkpoint, attempt, correction checkpoint, or status command",
		parameters: Type.Object({
			command: Type.String(),
			args: Type.Array(Type.String()),
		}),
		execute: async (_id, input: { command: string; args: string[] }, signal, _update, ctx) => {
			if (childRole !== "build") throw new Error("BUILD command execution is available only to the BUILD role");
			const trustedExecutable = process.env.SUPERDEV_TRUSTED_EXECUTABLE;
			const trustedDigest = process.env.SUPERDEV_TRUSTED_EXECUTABLE_SHA256;
			if (!trustedExecutable || !isAbsolute(trustedExecutable) || !trustedDigest) throw new Error("BUILD service executable is not pinned by the parent");
			const result = await runGuardedBuildCommand(
				async (_command, args, cwd) => runPinnedSuperdev(trustedExecutable, trustedDigest, args, cwd, signal),
				input.command,
				input.args,
				ctx.cwd,
			);
			if (result.code !== 0) throw new Error(result.stderr.trim() || `${input.command} exited ${result.code}`);
			return { content: [{ type: "text", text: result.stdout || "Command completed." }], details: { shellFree: true, serviceOwned: true } };
		},
	});
	if (childRole) return;

	type WorkflowStatus = {
		owner?: {
			session_id: string;
			last_plan_revision: string;
			candidate_revision?: string;
			verified_default_revision?: string;
			identity: { issue: string; plan: string; work_branch: string; default_branch: string };
		};
		phase?: "scope" | "build" | "accept";
		canonicalPlanRevision?: string;
		openWorkflows?: Array<{ issue: string; plan: string; work_branch: string; default_branch: string }>;
		buildState?: { currentBlock: number; attempts: number; finalCorrections: number; fingerprint?: string; blocker: string };
		maxStalledBlockAttempts?: number;
		maxFinalCorrectionCycles?: number;
		humanAcceptanceRequired?: boolean;
		executable?: string;
	};
	const workflowStatus = async (cwd: string): Promise<WorkflowStatus> => {
		try {
			const service = await ensureParentService(cwd);
			const result = await runPinnedSuperdev(service.path, service.digest, ["workflow", "status", "--json"], cwd);
			if (result.code !== 0) return {};
			const status = JSON.parse(result.stdout).result as WorkflowStatus;
			status.executable = service.path;
			return status;
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
		if (!status.owner && status.openWorkflows?.length) {
			ctx.ui.notify(`Canonical workflows are available to resume: ${status.openWorkflows.map((workflow) => workflow.plan).join(", ")}`, "info");
		}
	});
	pi.on("before_agent_start", async (event, ctx) => {
		const status = await workflowStatus(ctx.cwd);
		if (!status.owner || !status.phase) return;
		const durable = {
			phase: status.phase,
			identity: status.owner.identity,
			planRevision: status.owner.last_plan_revision,
			buildState: status.buildState,
		};
		return {
			systemPrompt: `${event.systemPrompt}\n\nCanonical Superdev workflow state (reloaded from Rust for this turn; never infer completion from conversation history):\n${JSON.stringify(durable)}`,
		};
	});

	pi.registerTool({
		name: "superdev_workflow_control",
		label: "Superdev workflow control",
		description: "Invoke one typed Rust workflow operation; human-gated actions confirm in trusted Pi UI",
		parameters: Type.Object({
			action: StringEnum(["start", "resume", "record-scope-review", "record-final-evidence", "approve-scope", "return-to-scope", "reject-acceptance", "accept", "abandon"] as const),
			session: Type.String(),
			issue: Type.Optional(Type.String()),
			plan: Type.Optional(Type.String()),
			workBranch: Type.Optional(Type.String()),
			defaultBranch: Type.Optional(Type.String()),
			expectedRevision: Type.Optional(Type.String()),
			revision: Type.Optional(Type.String()),
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
				if (input.action === "record-scope-review") {
					if (!input.revision) throw new Error("scope evidence requires the reviewed canonical revision");
					args.push("--revision", input.revision);
				}
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
			if (modifying && (cancelling || modifyingBusy || modifyingChild)) throw new Error("one modifying workflow child is already active");
			if (modifying) modifyingBusy = true;
			try {
				const trustedExecutable = input.role === "build" ? (await workflowStatus(ctx.cwd)).executable : undefined;
				if (input.role === "build" && (!trustedExecutable || !isAbsolute(trustedExecutable))) throw new Error("Rust status omitted its trusted executable path");
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
					trustedExecutable,
					input.role === "build" ? parentServiceDigest : undefined,
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
			const status = await workflowStatus(ctx.cwd);
			ctx.ui.notify(JSON.stringify(status), "info");
		},
	});
	pi.registerCommand("superdev-resume", {
		description: "Resume one canonical workflow after Rust reconstructs its durable state",
		handler: async (args, ctx) => {
			const status = await workflowStatus(ctx.cwd);
			if (status.owner) {
				ctx.ui.setStatus("superdev-workflow", `${status.phase?.toUpperCase() ?? "WORKFLOW"}: ${status.owner.identity.plan}`);
				return ctx.ui.notify(`${status.owner.identity.plan} is already owned and ready to continue`, "info");
			}
			const requested = args.trim();
			const candidates = (status.openWorkflows ?? []).filter((workflow) => !requested || workflow.plan === requested);
			if (candidates.length !== 1) {
				const plans = (status.openWorkflows ?? []).map((workflow) => workflow.plan).join(", ") || "none";
				return ctx.ui.notify(`Specify exactly one plan with /superdev-resume <plan-id>. Open workflows: ${plans}`, "warning");
			}
			const workflow = candidates[0];
			await runSuperdev([
				"workflow", "resume", "--session", ctx.sessionManager.getSessionId(),
				"--issue", workflow.issue, "--plan", workflow.plan,
				"--work-branch", workflow.work_branch, "--default-branch", workflow.default_branch,
			], ctx.cwd, authority);
			const resumed = await workflowStatus(ctx.cwd);
			ctx.ui.setStatus("superdev-workflow", `${resumed.phase?.toUpperCase() ?? "WORKFLOW"}: ${workflow.plan}`);
			ctx.ui.notify(`Resumed ${workflow.plan} from canonical ${resumed.phase?.toUpperCase() ?? "workflow"} state`, "info");
		},
	});
	pi.registerCommand("scope", {
		description: "Deterministically run SCOPE, isolated review, and human approval",
		handler: async (args, ctx) => {
			if (cancelling || modifyingBusy || modifyingChild) return ctx.ui.notify("A modifying workflow role is already active", "error");
			modifyingBusy = true;
			const commandAbort = new AbortController();
			modifyingCommandAbort = commandAbort;
			try {
			const initial = await workflowStatus(ctx.cwd);
			if (commandAbort.signal.aborted) return ctx.ui.notify("SCOPE cancelled during initialization", "warning");
			if (!initial.owner || initial.phase !== "scope") return ctx.ui.notify("An owned SCOPE workflow is required", "error");
			const scopeBaseResult = await pi.exec("git", ["rev-parse", "--verify", initial.owner.identity.work_branch], { cwd: ctx.cwd });
			if (scopeBaseResult.code !== 0) return ctx.ui.notify("Could not resolve the pre-SCOPE revision", "error");
			const scopeBase = scopeBaseResult.stdout.trim();
			const scoped = await isolated("scope", args || `Complete ${initial.owner.identity.plan} from canonical state.`, ctx.cwd, ctx.model, commandAbort.signal,
				(child) => { children.add(child); modifyingChild = child; },
				(child) => { children.delete(child); if (modifyingChild === child) modifyingChild = undefined; });
			if (commandAbort.signal.aborted) return ctx.ui.notify("SCOPE cancelled before publication", "warning");
			if (scoped.status !== "complete") return ctx.ui.notify(scoped.summary, "warning");
			await runSuperdev([
				"workflow", "scope-checkpoint", "--session", initial.owner.session_id,
				"--expected-revision", initial.owner.last_plan_revision,
			], ctx.cwd, authority);
			const status = await workflowStatus(ctx.cwd);
			const owner = status.owner;
			if (!owner || status.phase !== "scope" || !status.canonicalPlanRevision
				|| owner.session_id !== initial.owner.session_id
				|| owner.identity.issue !== initial.owner.identity.issue
				|| owner.identity.plan !== initial.owner.identity.plan
				|| owner.identity.work_branch !== initial.owner.identity.work_branch
				|| owner.identity.default_branch !== initial.owner.identity.default_branch) {
				throw new Error("SCOPE ownership changed during isolated work");
			}
			if (owner.last_plan_revision === initial.owner.last_plan_revision) throw new Error("SCOPE child did not publish a reviewable checkpoint");
			const candidateResult = await pi.exec("git", ["rev-parse", "--verify", owner.identity.work_branch], { cwd: ctx.cwd });
			if (candidateResult.code !== 0) return ctx.ui.notify("Could not resolve immutable review revisions", "error");
			const base = scopeBase;
			const candidate = candidateResult.stdout.trim();
			const reviewed = await isolated("requirements-review", `Review immutable SCOPE diff ${base}..${candidate} and return the required structured result.`, ctx.cwd, ctx.model, undefined,
				(child) => children.add(child), (child) => children.delete(child), base, candidate);
			if (reviewed.status !== "clean") return ctx.ui.notify(reviewed.summary, "warning");
			const reviewRun = randomBytes(24).toString("hex");
			const evidence = await runSuperdev([
				"workflow", "evidence", "--session", owner.session_id,
				"--expected-revision", owner.last_plan_revision, "--revision", status.canonicalPlanRevision,
				"--kind", "scope-review", "--review-session", reviewRun, "--candidate", candidate,
			], ctx.cwd, authority) as { result?: { last_plan_revision?: string } };
			const revision = evidence.result?.last_plan_revision;
			if (!revision) throw new Error("scope evidence response omitted the new plan revision");
			if (!await ctx.ui.confirm("Approve SCOPE?", `Approve the reviewed SCOPE for ${owner.identity.plan} and enter BUILD?`)) return;
			await runSuperdev([
				"workflow", "transition", "--session", owner.session_id,
				"--expected-revision", revision, "--phase", "scope", "--transition", "approve-scope",
			], ctx.cwd, authority);
			ctx.ui.setStatus("superdev-workflow", `BUILD: ${owner.identity.plan}`);
			ctx.ui.notify("SCOPE approved; workflow advanced to BUILD", "info");
			} finally {
				if (modifyingCommandAbort === commandAbort) modifyingCommandAbort = undefined;
				modifyingBusy = false;
			}
		},
	});
	pi.registerCommand("build", {
		description: "Deterministically run BUILD, verification, and immutable review",
		handler: async (args, ctx) => {
			if (cancelling || modifyingBusy || modifyingChild) return ctx.ui.notify("A modifying workflow role is already active", "error");
			modifyingBusy = true;
			const commandAbort = new AbortController();
			modifyingCommandAbort = commandAbort;
			try {
			let instruction = args;
			while (true) {
				const status = await workflowStatus(ctx.cwd);
				if (commandAbort.signal.aborted) return ctx.ui.notify("BUILD cancelled during initialization", "warning");
				const owner = status.owner;
				if (!owner || status.phase !== "build") return ctx.ui.notify("An owned BUILD workflow is required", "error");
				if (status.buildState && status.maxFinalCorrectionCycles !== undefined
					&& status.buildState.finalCorrections >= status.maxFinalCorrectionCycles) {
					return ctx.ui.notify("Final correction limit is exhausted; human guidance or re-scope is required", "error");
				}
				if (!status.executable || !isAbsolute(status.executable)) throw new Error("Rust status omitted its trusted executable path");
				const built = await isolated("build", instruction || `Complete ${owner.identity.plan} from canonical state.`, ctx.cwd, ctx.model, commandAbort.signal,
				(child) => { children.add(child); modifyingChild = child; },
				(child) => { children.delete(child); if (modifyingChild === child) modifyingChild = undefined; }, undefined, undefined, status.executable, parentServiceDigest);
			if (commandAbort.signal.aborted) return ctx.ui.notify("BUILD cancelled before publication", "warning");
			if (built.status === "rescope") {
				const latest = await workflowStatus(ctx.cwd);
				if (!latest.owner || latest.phase !== "build") throw new Error("BUILD ownership changed before re-scope");
				await runSuperdev([
					"workflow", "transition", "--session", latest.owner.session_id,
					"--expected-revision", latest.owner.last_plan_revision, "--phase", "build",
					"--transition", "return-to-scope", "--feedback", built.summary,
				], ctx.cwd, authority);
				ctx.ui.setStatus("superdev-workflow", `SCOPE: ${latest.owner.identity.plan}`);
				return ctx.ui.notify("BUILD discovery was preserved on the primary issue; the same plan returned to SCOPE", "warning");
			}
			if (built.status !== "complete") return ctx.ui.notify(built.summary, "error");
			const ready = await workflowStatus(ctx.cwd);
			const currentOwner = ready.owner;
			if (!currentOwner || ready.phase !== "build") throw new Error("BUILD ownership changed before synchronization");
			const baseResult = await pi.exec("git", ["rev-parse", "--verify", currentOwner.identity.default_branch], { cwd: ctx.cwd });
			const candidateResult = await pi.exec("git", ["rev-parse", "--verify", currentOwner.identity.work_branch], { cwd: ctx.cwd });
			if (baseResult.code !== 0 || candidateResult.code !== 0) return ctx.ui.notify("Could not resolve synchronization revisions", "error");
			await runSuperdev([
				"workflow", "sync", "--session", currentOwner.session_id,
				"--expected-revision", currentOwner.last_plan_revision,
				"--expected-default", baseResult.stdout.trim(),
				"--expected-work", candidateResult.stdout.trim(),
			], ctx.cwd, authority);
			const synchronized = await workflowStatus(ctx.cwd);
			const synchronizedOwner = synchronized.owner;
			if (!synchronizedOwner || synchronized.phase !== "build") throw new Error("BUILD ownership changed after synchronization");
			const synchronizedBase = await pi.exec("git", ["rev-parse", "--verify", synchronizedOwner.identity.default_branch], { cwd: ctx.cwd });
			const synchronizedCandidate = await pi.exec("git", ["rev-parse", "--verify", synchronizedOwner.identity.work_branch], { cwd: ctx.cwd });
			if (synchronizedBase.code !== 0 || synchronizedCandidate.code !== 0) return ctx.ui.notify("Could not resolve immutable review revisions", "error");
			const base = synchronizedBase.stdout.trim();
			const candidate = synchronizedCandidate.stdout.trim();
			const verification = await runSuperdev([
				"workflow", "evidence", "--session", synchronizedOwner.session_id,
				"--expected-revision", synchronizedOwner.last_plan_revision, "--kind", "verification",
				"--candidate", candidate,
			], ctx.cwd, authority) as { result?: { last_plan_revision?: string } };
			const verifiedRevision = verification.result?.last_plan_revision;
			if (!verifiedRevision) throw new Error("verification response omitted the new plan revision");
			const reviewed = await isolated("code-review", `Review immutable diff ${base}..${candidate} and return the required structured result.`, ctx.cwd, ctx.model, undefined,
				(child) => children.add(child), (child) => children.delete(child), base, candidate);
			const reviewRun = randomBytes(24).toString("hex");
			if (reviewed.status !== "clean") {
				const correction = await runSuperdev([
					"workflow", "correction", "--session", synchronizedOwner.session_id,
					"--expected-revision", verifiedRevision, "--candidate", candidate,
					"--review-session", reviewRun,
					"--summary", (reviewed.findings ?? [reviewed.summary]).join("\n"),
				], ctx.cwd, authority) as { result?: { stalled?: boolean } };
				const stalled = correction.result?.stalled === true;
				if (stalled) return ctx.ui.notify(`Final correction limit exhausted: ${reviewed.summary}`, "error");
				ctx.ui.notify(`Final review requires correction: ${reviewed.summary}`, "warning");
				instruction = `Correct the latest immutable final-review findings for ${synchronizedOwner.identity.plan}:\n${(reviewed.findings ?? [reviewed.summary]).join("\n")}`;
				continue;
			}

			reviewRuns.set(reviewRun, { role: "code-review", result: reviewed, base, candidate });
			const evidence = await runSuperdev([
				"workflow", "evidence", "--session", synchronizedOwner.session_id,
				"--expected-revision", verifiedRevision, "--kind", "final",
				"--review-session", reviewRun, "--candidate", candidate,
			], ctx.cwd, authority) as { result?: { last_plan_revision?: string } };
			const revision = evidence.result?.last_plan_revision;
			if (!revision) throw new Error("attestation response omitted the new plan revision");
			reviewRuns.delete(reviewRun);
				ctx.ui.setStatus("superdev-workflow", `ACCEPT: ${synchronizedOwner.identity.plan}`);
				return ctx.ui.notify("BUILD gates passed; workflow advanced to ACCEPT", "info");
			}
			} finally {
				if (modifyingCommandAbort === commandAbort) modifyingCommandAbort = undefined;
				modifyingBusy = false;
			}
		},
	});
	pi.registerCommand("accept", {
		description: "Deterministically decide ACCEPT and integrate locally",
		handler: async (_args, ctx) => {
			const status = await workflowStatus(ctx.cwd);
			const owner = status.owner;
			if (!owner || status.phase !== "accept") return ctx.ui.notify("An owned ACCEPT workflow is required", "error");
			if (!owner.candidate_revision || !owner.verified_default_revision) throw new Error("ACCEPT state is missing candidate-bound BUILD evidence");
			const currentDefault = await pi.exec("git", ["rev-parse", "--verify", owner.identity.default_branch], { cwd: ctx.cwd });
			if (currentDefault.code !== 0) throw new Error("could not resolve the configured default branch");
			if (currentDefault.stdout.trim() !== owner.verified_default_revision) {
				await runSuperdev([
					"workflow", "transition", "--session", owner.session_id,
					"--expected-revision", owner.last_plan_revision, "--phase", "accept",
					"--transition", "recover-stale-default",
				], ctx.cwd, authority);
				ctx.ui.setStatus("superdev-workflow", `BUILD: ${owner.identity.plan}`);
				return ctx.ui.notify("Default branch advanced; final evidence was invalidated and workflow returned to BUILD", "warning");
			}
			let accepted = true;
			if (status.humanAcceptanceRequired) {
				accepted = await ctx.ui.confirm("Accept candidate?", `Accept reviewed candidate ${owner.candidate_revision} and integrate it locally?`);
			}
			if (!accepted) {
				const feedback = await ctx.ui.input("Rejection feedback", "Required feedback for returning this plan to SCOPE");
				if (!feedback?.trim()) return ctx.ui.notify("Rejection requires non-empty feedback", "error");
				await runSuperdev([
					"workflow", "transition", "--session", owner.session_id,
					"--expected-revision", owner.last_plan_revision, "--phase", "accept",
					"--transition", "reject-acceptance", "--feedback", feedback,
				], ctx.cwd, authority);
				ctx.ui.setStatus("superdev-workflow", `SCOPE: ${owner.identity.plan}`);
				return ctx.ui.notify("Candidate rejected; feedback preserved and workflow returned to SCOPE", "warning");
			}
			await runSuperdev([
				"workflow", "transition", "--session", owner.session_id,
				"--expected-revision", owner.last_plan_revision, "--phase", "accept", "--transition", "accept",
			], ctx.cwd, authority);
			const closureResult = await pi.exec("git", ["rev-parse", "--verify", owner.identity.work_branch], { cwd: ctx.cwd });
			if (closureResult.code !== 0) throw new Error("could not resolve the prepared closure commit");
			await runSuperdev([
				"workflow", "integrate", "--session", owner.session_id,
				"--default-branch", owner.identity.default_branch,
				"--expected-default", owner.verified_default_revision,
				"--work-branch", owner.identity.work_branch,
				"--expected-work", closureResult.stdout.trim(),
			], ctx.cwd, authority);
			ctx.ui.setStatus("superdev-workflow", undefined);
			ctx.ui.notify("Candidate accepted and integrated locally; nothing was pushed or deleted", "info");
		},
	});
	pi.registerCommand("superdev-cancel", {
		description: "Pause the workflow and release transient ownership",
		handler: async (_args, ctx) => {
			if (cancelling) return ctx.ui.notify("Workflow cancellation is already in progress", "warning");
			cancelling = true;
			try {
			modifyingCommandAbort?.abort();
			const stopping = [...children];
			for (const child of stopping) stopChild(child);
			await Promise.race([
				Promise.all(stopping.map((child) => new Promise<void>((done) => child.once("close", () => done())))),
				new Promise<void>((done) => setTimeout(done, 2_500)),
			]);
			for (let attempt = 0; modifyingBusy && attempt < 100; attempt += 1) {
				await new Promise((done) => setTimeout(done, 25));
			}
			if (modifyingBusy) return ctx.ui.notify("Could not safely pause while a modifying command is still shutting down", "error");
			const session = ctx.sessionManager.getSessionId();
			try {
				await runSuperdev(["workflow", "cancel", "--session", session], ctx.cwd, authority);
				ctx.ui.notify("Workflow paused; canonical phase unchanged", "info");
			} catch (error) {
				ctx.ui.notify(String(error), "error");
			}
			} finally {
				cancelling = false;
			}
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

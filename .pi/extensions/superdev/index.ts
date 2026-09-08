import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { createHash, randomBytes } from "node:crypto";
import { constants } from "node:fs";
import { access, chmod, mkdtemp, open, readFile, rm, type FileHandle } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { StringEnum } from "@earendil-works/pi-ai";
import { Type, type Static } from "typebox";
import { IsolatedArtifact, boundedText, cleanupArtifacts, type OutputPolicy } from "./lib/output.ts";
import { withProgress } from "./lib/progress.ts";
import { registerWorkflowQuestions, type QuestionState } from "./lib/questions.ts";
import { findingFingerprint, parseLegacyRoleResult, roleResultSchema, validateRoleResult, type ReviewFinding, type Role, type RoleResult } from "./lib/review.ts";

const here = dirname(fileURLToPath(import.meta.url));
const schema = Type.Object({
	role: StringEnum(["scope", "requirements-review", "build", "code-review", "accept", "file"] as const),
	task: Type.String({ description: "Bounded task and all input the isolated role needs" }),
	base: Type.Optional(Type.String({ description: "Immutable review base" })),
	candidate: Type.Optional(Type.String({ description: "Immutable review candidate" })),
});
type Input = Static<typeof schema>;

const readOnly = new Set<Role>(["requirements-review", "code-review", "accept"]);

export function parseRoleResult(role: Input["role"], answer: string): RoleResult {
	return parseLegacyRoleResult(role, answer);
}

function killProcessTree(pid: number) {
	if (process.platform === "win32") {
		spawnSync(`${process.env.SystemRoot ?? "C:\\Windows"}\\System32\\taskkill.exe`, ["/F", "/T", "/PID", String(pid)], { windowsHide: true, stdio: "ignore" });
		return;
	}
	try { process.kill(-pid, "SIGKILL"); }
	catch { try { process.kill(pid, "SIGKILL"); } catch { /* Already exited. */ } }
}

function stopProcess(child: ChildProcess) {
	if (child.pid) killProcessTree(child.pid);
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
	if (result.code !== 0) throw new Error(boundedText(result.stderr.trim() || `superdev exited ${result.code}`, defaultOutputPolicy));
	try {
		const parsed = JSON.parse(result.stdout);
		if (parsed.protocol !== "superdev-workflow/v2") throw new Error(`unsupported workflow protocol ${String(parsed.protocol)}`);
		return parsed;
	} catch (error) {
		if (error instanceof SyntaxError) throw new Error("superdev returned invalid workflow JSON");
		throw error;
	}
}

export function requiresHumanAcceptance(value: boolean | undefined): boolean {
	if (value === undefined) throw new Error("ACCEPT status omitted the configured human-acceptance policy");
	return value;
}

export function isolatedTools(role: Input["role"]): string {
	if (role === "code-review") return "superdev_review_diff,sokf_search,sokf_graph,superdev_submit_result";
	if (readOnly.has(role)) return "read,sokf_search,sokf_graph,superdev_submit_result";
	if (role === "file") return "read,sokf_search,sokf_graph,superdev_submit_result";
	if (role === "build") return "read,edit,write,sokf_search,sokf_graph,superdev_build_exec,superdev_submit_result";
	return "read,edit,write,sokf_search,sokf_graph,superdev_submit_result";
}

const defaultOutputPolicy: OutputPolicy = {
	timeoutSeconds: 1_200,
	maxContextBytes: 8_192,
	maxContextLines: 200,
	maxReviewStateBytes: 262_144,
	maxReviewFindings: 100,
	maxArtifactBytes: 10_485_760,
	maxArtifacts: 20,
	retentionHours: 24,
};

async function isolated(
	role: Input["role"],
	task: string,
	cwd: string,
	model?: { provider: string; id: string },
	signal?: AbortSignal,
	onSpawn?: (child: ChildProcess) => void,
	onClose?: (child: ChildProcess) => void | Promise<void>,
	base?: string,
	candidate?: string,
	trustedExecutable?: string,
	trustedExecutableSha256?: string,
	policy: OutputPolicy = defaultOutputPolicy,
	session = "no-session",
	onActivity?: (activity: string) => void,
): Promise<RoleResult> {
	const promptPath = resolve(here, "prompts", `${role}.md`);
	const rolePrompt = await readFile(promptPath, "utf8");
	if (readOnly.has(role) && (!base || !candidate || !/^[0-9a-f]{7,64}$/.test(base) || !/^[0-9a-f]{7,64}$/.test(candidate))) {
		throw new Error(`${role} requires immutable hexadecimal base and candidate revisions`);
	}
	if (trustedExecutable && !trustedExecutableSha256) throw new Error("trusted executable digest was not supplied");
	const artifact = await IsolatedArtifact.create(session, role, policy.maxArtifactBytes);
	await cleanupArtifacts(session, policy);
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
				SUPERDEV_MAX_REVIEW_STATE_BYTES: String(policy.maxReviewStateBytes ?? 262_144),
				SUPERDEV_MAX_REVIEW_FINDINGS: String(policy.maxReviewFindings ?? 100),
				SUPERDEV_MAX_CONTEXT_BYTES: String(policy.maxContextBytes),
				SUPERDEV_MAX_CONTEXT_LINES: String(policy.maxContextLines),
			},
		});
		onSpawn?.(child);
		let pending = "";
		let stderrTail = "";
		let submission: unknown;
		let submissions = 0;
		let settled = false;
		let timedOut = false;
		const finish = (operation: () => Promise<void>) => {
			if (settled) return;
			settled = true;
			clearTimeout(timeout);
			signal?.removeEventListener("abort", stop);
			void operation().catch((error) => reject(error));
		};
		const consume = (line: string) => {
			if (!line.trim()) return;
			let event: any;
			try { event = JSON.parse(line); } catch { return; }
			if (event.type === "tool_execution_start") {
				const target = typeof event.args?.path === "string" ? ` ${event.args.path}` : "";
				onActivity?.(`${event.toolName}${target}`);
			}
			if (event.type === "tool_execution_end" && event.toolName === "superdev_submit_result" && !event.isError) {
				submissions += 1;
				submission = event.result?.details?.superdevResult;
			}
		};
		const stop = () => stopProcess(child);
		const timeout = setTimeout(() => { timedOut = true; stop(); }, policyTimeoutMs(policy));
		timeout.unref();
		child.stdout.setEncoding("utf8");
		child.stderr.setEncoding("utf8");
		child.stdout.on("data", (chunk: string) => {
			pending += chunk;
			if (Buffer.byteLength(pending) > policy.maxArtifactBytes) {
				stop();
				return finish(async () => {
					await onClose?.(child);
					await artifact.finishStderr();
					reject(new Error(`isolated-output-overflow: ${role} emitted an over-cap event`));
				});
			}
			let newline;
			while ((newline = pending.indexOf("\n")) >= 0) {
				consume(pending.slice(0, newline));
				pending = pending.slice(newline + 1);
			}
		});
		child.stderr.on("data", (chunk: string) => {
			artifact.writeStderr(chunk);
			stderrTail = (stderrTail + chunk).slice(-Math.max(policy.maxContextBytes * 2, 16_384));
		});
		child.on("error", (error) => finish(async () => { await onClose?.(child); await artifact.finishStderr(); reject(error); }));
		child.on("close", (code) => finish(async () => {
			await onClose?.(child);
			if (pending) consume(pending);
			await artifact.finishStderr();
			if (timedOut) return reject(new Error(`isolated role timed out after ${policyTimeoutMs(policy) / 1_000}s; diagnostics: ${artifact.stderrPath}`));
			if (signal?.aborted) return reject(new Error(`isolated ${role} role was cancelled; diagnostics: ${artifact.stderrPath}`));
			if (code !== 0) {
				const diagnostic = boundedText(stderrTail.trim() || `${role} exited ${code}`, policy, "tail", artifact.stderrPath);
				return reject(new Error(diagnostic));
			}
			if (submissions !== 1 || submission === undefined) return reject(new Error(`${role} must submit exactly one typed terminal result`));
			try {
				const result = validateRoleResult(role, submission, {
					maxBytes: policy.maxReviewStateBytes ?? 262_144,
					maxFindings: policy.maxReviewFindings ?? 100,
				});
				await artifact.writeResult(result);
				accept({ ...result, artifactPath: artifact.resultPath });
			} catch (error) { reject(error); }
		}));
		if (signal) {
			if (signal.aborted) stop(); else signal.addEventListener("abort", stop, { once: true });
		}
	});
}

function policyTimeoutMs(policy: OutputPolicy & { timeoutSeconds?: number }): number {
	return (policy.timeoutSeconds ?? 1_200) * 1_000;
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
				detached: true,
				stdio: ["ignore", "pipe", "pipe", executable!.fd],
				env: environment ? { ...process.env, ...environment } : process.env,
			});
			let stdout = "";
			let stderr = "";
			child.stdout.setEncoding("utf8");
			child.stderr.setEncoding("utf8");
			child.stdout.on("data", (chunk: string) => { stdout = (stdout + chunk).slice(-51_200); });
			child.stderr.on("data", (chunk: string) => { stderr = (stderr + chunk).slice(-51_200); });
			const stop = () => { if (child.pid) killProcessTree(child.pid); };
			const cleanup = () => {
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
	const childRoleValue = process.env.SUPERDEV_CHILD_ROLE;
	const childRole = childRoleValue && ["scope", "requirements-review", "build", "code-review", "accept", "file"].includes(childRoleValue)
		? childRoleValue as Role
		: undefined;
	const reviewPaths = new Set<string>();
	const reviewOffsets = new Map<string, number>();
	let inventoryLoaded = false;
	let inventoryExpectedOffset = 1;
	let inventoryComplete = false;
	if (childRole) {
		pi.registerTool({
			name: "superdev_submit_result",
			label: "Submit Superdev role result",
			description: "Submit the one authoritative typed result and terminate this isolated role",
			parameters: roleResultSchema,
			executionMode: "sequential",
			execute: async (_id, input) => {
				const result = validateRoleResult(childRole, input, {
					maxBytes: Number(process.env.SUPERDEV_MAX_REVIEW_STATE_BYTES ?? 262_144),
					maxFindings: Number(process.env.SUPERDEV_MAX_REVIEW_FINDINGS ?? 100),
				});
				if (childRole === "code-review" && result.status !== "blocked") {
					if (!inventoryLoaded || !inventoryComplete) throw new Error("code review did not completely inspect the changed-path inventory");
					const missing = [...reviewPaths].filter((path) => reviewOffsets.get(path) !== -1);
					if (missing.length) throw new Error(`code review omitted changed paths: ${missing.join(", ")}`);
				}
				return {
					content: [{ type: "text", text: `Submitted ${childRole} result: ${result.status}` }],
					details: { superdevResult: result },
					terminate: true,
				};
			},
		});
	}
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

	if (childRole === "code-review") pi.registerTool({
		name: "superdev_review_diff",
		label: "Superdev review diff",
		description: "Inventory or read one bounded page of the parent-bound immutable Git diff",
		parameters: Type.Object({
			path: Type.Optional(Type.String({ description: "Changed repo-relative path; omit for inventory" })),
			offset: Type.Optional(Type.Number({ minimum: 1, description: "One-indexed diff line offset" })),
			limit: Type.Optional(Type.Number({ minimum: 1, maximum: 200, description: "Most diff lines to return" })),
		}),
		execute: async (_id, input: { path?: string; offset?: number; limit?: number }, _signal, _update, ctx) => {
			const base = process.env.SUPERDEV_REVIEW_BASE;
			const candidate = process.env.SUPERDEV_REVIEW_CANDIDATE;
			if (!base || !candidate || !/^[0-9a-f]{7,64}$/.test(base) || !/^[0-9a-f]{7,64}$/.test(candidate)) {
				throw new Error("review revisions were not bound by the parent");
			}
			if (!input.path) {
				const offset = Math.max(1, Math.floor(input.offset ?? 1));
				if (offset !== inventoryExpectedOffset) throw new Error(`inventory pagination must continue at offset ${inventoryExpectedOffset}`);
				if (!inventoryLoaded) {
					const names = await pi.exec("git", ["diff", "--no-ext-diff", "--name-only", base, candidate, "--"], { cwd: ctx.cwd });
					if (names.code !== 0) throw new Error(names.stderr.trim() || "git diff inventory failed");
					for (const path of names.stdout.split("\n").filter(Boolean)) {
						reviewPaths.add(path);
						reviewOffsets.set(path, 1);
					}
					inventoryLoaded = true;
				}
				const paths = [...reviewPaths];
				const requested = Math.min(200, Math.max(1, Math.floor(input.limit ?? 200)));
				const selected: string[] = [];
				for (const path of paths.slice(offset - 1, offset - 1 + requested)) {
					if (Buffer.byteLength([...selected, path].join("\n")) > 8_192) break;
					selected.push(path);
				}
				const nextOffset = offset - 1 + selected.length < paths.length ? offset + selected.length : undefined;
				inventoryExpectedOffset = nextOffset ?? -1;
				inventoryComplete = nextOffset === undefined;
				return {
					content: [{ type: "text", text: selected.length ? `Changed paths:\n${selected.join("\n")}` : "No changed paths." }],
					details: { readOnly: true, inventory: true, offset, paths: selected.length, totalPaths: paths.length, nextOffset },
				};
			}
			if (isAbsolute(input.path) || input.path.split(/[\\/]/).includes("..") || !reviewPaths.has(input.path)) {
				throw new Error("review path must come from the bound diff inventory");
			}
			const result = await pi.exec("git", ["diff", "--no-ext-diff", "--unified=20", base, candidate, "--", input.path], { cwd: ctx.cwd });
			if (result.code !== 0) throw new Error(result.stderr.trim() || "git diff failed");
			if (!inventoryComplete) throw new Error("complete the changed-path inventory before reading diffs");
			const lines = result.stdout.split("\n");
			const offset = Math.max(1, Math.floor(input.offset ?? 1));
			const expectedOffset = reviewOffsets.get(input.path);
			if (expectedOffset === -1) throw new Error("that path is already completely reviewed");
			if (offset !== expectedOffset) throw new Error(`diff pagination for ${input.path} must continue at offset ${expectedOffset}`);
			const requested = Math.min(200, Math.max(1, Math.floor(input.limit ?? 200)));
			const selected: string[] = [];
			for (const line of lines.slice(offset - 1, offset - 1 + requested)) {
				if (Buffer.byteLength([...selected, line].join("\n")) > 8_192) break;
				selected.push(line);
			}
			if (!selected.length && lines.length) throw new Error("diff page byte limit prevented progress");
			const nextOffset = offset - 1 + selected.length < lines.length ? offset + selected.length : undefined;
			reviewOffsets.set(input.path, nextOffset ?? -1);
			return {
				content: [{ type: "text", text: selected.join("\n") || "No changes for that path." }],
				details: { readOnly: true, path: input.path, offset, lines: selected.length, totalLines: lines.length, nextOffset },
			};
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
			const childPolicy = {
				...defaultOutputPolicy,
				maxContextBytes: Number(process.env.SUPERDEV_MAX_CONTEXT_BYTES ?? 8_192),
				maxContextLines: Number(process.env.SUPERDEV_MAX_CONTEXT_LINES ?? 200),
			};
			if (result.code !== 0) throw new Error(boundedText(result.stderr.trim() || `${input.command} exited ${result.code}`, childPolicy));
			return { content: [{ type: "text", text: boundedText(result.stdout || "Command completed.", childPolicy) }], details: { shellFree: true, serviceOwned: true } };
		},
	});
	if (childRole) return;

	type WorkflowStatus = {
		owner?: {
			session_id: string;
			last_plan_revision: string;
			scope_base_revision?: string;
			candidate_revision?: string;
			verified_default_revision?: string;
			owner_pid?: number;
			owner_started?: string;
			child_role?: Role;
			child_pid?: number;
			child_started?: string;
			identity: { issue: string; plan: string; work_branch: string; default_branch: string };
		};
		phase?: "scope" | "build" | "accept";
		canonicalPlanRevision?: string;
		openWorkflows?: Array<{ issue: string; plan: string; work_branch: string; default_branch: string }>;
		buildState?: { currentBlock: number; attempts: number; finalCorrections: number; fingerprint?: string; blocker: string };
		maxStalledBlockAttempts?: number;
		maxFinalCorrectionCycles?: number;
		maxScopeReviewCycles?: number;
		isolatedRoleTimeoutSeconds?: number;
		maxIsolatedContextBytes?: number;
		maxIsolatedContextLines?: number;
		maxReviewStateBytes?: number;
		maxReviewFindings?: number;
		maxIsolatedArtifactBytes?: number;
		maxIsolatedArtifactsPerSession?: number;
		isolatedArtifactRetentionHours?: number;
		humanAcceptanceRequired?: boolean;
		executable?: string;
	};
	let reviewStateLimit = 262_144;
	let reviewFindingLimit = 100;
	const workflowStatus = async (cwd: string): Promise<WorkflowStatus> => {
		if (process.env.SUPERDEV_VERIFICATION_ACTIVE === "1") return {};
		try {
			const service = await ensureParentService(cwd);
			const result = await runPinnedSuperdev(service.path, service.digest, ["workflow", "status", "--json"], cwd);
			if (result.code !== 0) return {};
			const response = JSON.parse(result.stdout);
			if (response.protocol !== "superdev-workflow/v2") throw new Error(`unsupported workflow protocol ${String(response.protocol)}`);
			const status = response.result as WorkflowStatus;
			reviewStateLimit = status.maxReviewStateBytes ?? reviewStateLimit;
			reviewFindingLimit = status.maxReviewFindings ?? reviewFindingLimit;
			status.executable = service.path;
			return status;
		} catch {
			return {};
		}
	};
	const policyFrom = (status: WorkflowStatus): OutputPolicy => ({
		timeoutSeconds: status.isolatedRoleTimeoutSeconds ?? 1_200,
		maxContextBytes: status.maxIsolatedContextBytes ?? 8_192,
		maxContextLines: status.maxIsolatedContextLines ?? 200,
		maxReviewStateBytes: status.maxReviewStateBytes ?? 262_144,
		maxReviewFindings: status.maxReviewFindings ?? 100,
		maxArtifactBytes: status.maxIsolatedArtifactBytes ?? 10_485_760,
		maxArtifacts: status.maxIsolatedArtifactsPerSession ?? 20,
		retentionHours: status.isolatedArtifactRetentionHours ?? 24,
	});
	let continueScopeFromAnswers: ((state: QuestionState, ctx: any) => Promise<void>) | undefined;
	let continueAcceptRequest: ((state: QuestionState, ctx: any) => Promise<void>) | undefined;
	const questions = registerWorkflowQuestions(pi, {
		maxBytes: () => reviewStateLimit,
		maxFindings: () => reviewFindingLimit,
		onPause: async (_state, ctx) => {
			const status = await workflowStatus(ctx.cwd);
			if (status.owner) await runSuperdev(["workflow", "cancel", "--session", status.owner.session_id], ctx.cwd, authority);
		},
		onResume: async (state, ctx) => {
			const status = await workflowStatus(ctx.cwd);
			if (status.owner) return;
			const workflow = status.openWorkflows?.find((candidate) => candidate.plan === state.workflow);
			if (!workflow) throw new Error(`cannot find canonical workflow ${state.workflow} to resume`);
			await runSuperdev([
				"workflow", "resume", "--session", ctx.sessionManager.getSessionId(),
				"--issue", workflow.issue, "--plan", workflow.plan,
				"--work-branch", workflow.work_branch, "--default-branch", workflow.default_branch,
			], ctx.cwd, authority);
		},
		onSubmit: async (state, ctx) => {
			if (state.originPhase === "accept") {
				if (!continueAcceptRequest) throw new Error("ACCEPT continuation is unavailable");
				return await continueAcceptRequest(state, ctx);
			}
			if (!continueScopeFromAnswers) throw new Error("SCOPE continuation is unavailable");
			await continueScopeFromAnswers(state, ctx);
		},
	});
	const processStartIdentity = async (pid: number): Promise<string> => {
		if (process.platform !== "linux") return String(pid);
		const fields = (await readFile(`/proc/${pid}/stat`, "utf8")).trim().split(" ");
		return fields[21] ?? `${pid}:unknown`;
	};
	const processMatches = async (pid: number | undefined, started: string | undefined): Promise<boolean> => {
		if (!pid || !started) return false;
		try {
			if (process.platform !== "linux") { process.kill(pid, 0); return started === String(pid); }
			return await processStartIdentity(pid) === started;
		} catch { return false; }
	};
	const recordChildStart = async (cwd: string, status: WorkflowStatus, role: Role, child: ChildProcess) => {
		if (!status.owner || !child.pid) return;
		await runSuperdev([
			"workflow", "activity-start", "--session", status.owner.session_id,
			"--expected-revision", status.owner.last_plan_revision, "--role", role,
			"--owner-pid", String(process.pid), "--owner-started", await processStartIdentity(process.pid),
			"--child-pid", String(child.pid), "--child-started", await processStartIdentity(child.pid),
		], cwd, authority);
	};
	const recordChildFinish = async (cwd: string, expectedPid: number | undefined) => {
		const latest = await workflowStatus(cwd);
		if (!latest.owner || !expectedPid || latest.owner.child_pid !== expectedPid) return;
		await runSuperdev(["workflow", "activity-finish", "--session", latest.owner.session_id, "--expected-revision", latest.owner.last_plan_revision], cwd, authority);
	};
	const activityRegistrations = new Map<ChildProcess, Promise<void>>();
	const childStarted = (ctx: any, status: WorkflowStatus, role: Role, modifying = false) => (child: ChildProcess) => {
		children.add(child);
		if (modifying) modifyingChild = child;
		const registration = recordChildStart(ctx.cwd, status, role, child)
			.catch((error) => ctx.ui.notify(`Could not record child activity: ${String(error)}`, "warning"));
		activityRegistrations.set(child, registration);
	};
	const childFinished = async (ctx: any, child: ChildProcess) => {
		children.delete(child);
		if (modifyingChild === child) modifyingChild = undefined;
		const registration = activityRegistrations.get(child) ?? Promise.resolve();
		activityRegistrations.delete(child);
		try {
			await registration;
			await recordChildFinish(ctx.cwd, child.pid);
		} catch (error) {
			ctx.ui.notify(`Could not clear child activity: ${String(error)}`, "warning");
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
		if (status.owner?.child_pid && !(await processMatches(status.owner.owner_pid, status.owner.owner_started))) {
			if (await processMatches(status.owner.child_pid, status.owner.child_started)) killProcessTree(status.owner.child_pid);
			await runSuperdev(["workflow", "activity-finish", "--session", status.owner.session_id, "--expected-revision", status.owner.last_plan_revision], ctx.cwd, authority);
			await runSuperdev(["workflow", "cancel", "--session", status.owner.session_id], ctx.cwd, authority);
			ctx.ui.notify("Recovered an orphaned isolated role. Partial work was preserved; resume explicitly when ready.", "warning");
			status.owner = undefined;
			status.phase = undefined;
		}
		const restoredQuestions = questions.restore(ctx.sessionManager.getEntries());
		if (questions.restoreIssue()) ctx.ui.notify(`${questions.restoreIssue()}; restart exhaustive review explicitly`, "warning");
		if (restoredQuestions && status.canonicalPlanRevision && restoredQuestions.candidate !== status.canonicalPlanRevision) {
			questions.supersede("candidate revision changed");
			ctx.ui.notify("Saved workflow questions are stale; restart exhaustive review when ready", "warning");
		} else if (restoredQuestions?.status === "active" || restoredQuestions?.status === "paused") {
			ctx.ui.notify(`Workflow questions restored (${Object.keys(restoredQuestions.answers).length}/${restoredQuestions.findings.length} answered). Say ‘resume workflow questions’ to continue.`, "info");
		}
		ctx.ui.setStatus(
			"superdev-workflow",
			status.owner ? `${status.phase?.toUpperCase() ?? "WORKFLOW"}: ${status.owner.identity?.plan ?? "owned"}` : undefined,
		);
		if (!status.owner && status.openWorkflows?.length) {
			ctx.ui.notify(`Canonical workflows are available to resume: ${status.openWorkflows.map((workflow) => workflow.plan).join(", ")}`, "info");
		}
	});
	pi.on("session_shutdown", async (_event, ctx) => {
		for (const child of children) stopChild(child);
		await Promise.allSettled([...activityRegistrations.values()]);
		const status = await workflowStatus(ctx.cwd);
		if (status.owner?.session_id === ctx.sessionManager.getSessionId()) {
			try {
				if (status.owner.child_pid) await runSuperdev(["workflow", "activity-finish", "--session", status.owner.session_id, "--expected-revision", status.owner.last_plan_revision], ctx.cwd, authority);
				await runSuperdev(["workflow", "cancel", "--session", status.owner.session_id], ctx.cwd, authority);
			} catch { /* Shutdown remains best-effort after child termination. */ }
		}
		await cleanupArtifacts(ctx.sessionManager.getSessionId(), policyFrom(status));
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
			action: StringEnum(["start", "resume", "record-scope-review", "record-verification", "record-final-evidence", "approve-scope", "return-to-scope", "reject-acceptance", "accept", "abandon"] as const),
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
				const status = await workflowStatus(ctx.cwd);
				acceptanceRequiresHuman = requiresHumanAcceptance(status.humanAcceptanceRequired);
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
			} else if (input.action === "record-verification") {
				if (!input.expectedRevision || !input.candidate) throw new Error("verification evidence fields are incomplete");
				args.push("evidence", "--session", input.session, "--expected-revision", input.expectedRevision, "--kind", "verification", "--candidate", input.candidate);
			} else if (input.action === "record-scope-review" || input.action === "record-final-evidence") {
				if (!input.expectedRevision || !input.reviewRun) throw new Error("evidence compare-and-swap fields are incomplete");
				const review = reviewRuns.get(input.reviewRun);
				const expectedRole = input.action === "record-scope-review" ? "requirements-review" : "code-review";
				if (!review || review.role !== expectedRole || review.result.status !== "clean") throw new Error("evidence does not name a clean bound review run");
				if (!input.candidate || input.candidate !== review.candidate) throw new Error("evidence candidate differs from the reviewed candidate");
				const status = await workflowStatus(ctx.cwd);
				const expectedBase = input.action === "record-scope-review"
					? status.owner?.scope_base_revision
					: status.owner?.verified_default_revision;
				if (!expectedBase || review.base !== expectedBase) throw new Error("evidence review base differs from authoritative workflow state");
				args.push("evidence", "--session", input.session, "--expected-revision", input.expectedRevision, "--kind", input.action === "record-scope-review" ? "scope-review" : "final", "--review-session", input.reviewRun);
				if (input.action === "record-scope-review") {
					if (!input.revision || input.revision !== status.canonicalPlanRevision) throw new Error("scope evidence revision differs from canonical reviewed state");
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
			if (cancelling) throw new Error("workflow cancellation is in progress");
			const modifying = !readOnly.has(input.role) && input.role !== "file";
			if (modifying && (cancelling || modifyingBusy || modifyingChild)) throw new Error("one modifying workflow child is already active");
			if (modifying) modifyingBusy = true;
			try {
				const status = await workflowStatus(ctx.cwd);
				const trustedExecutable = input.role === "build" ? status.executable : undefined;
				if (cancelling || signal.aborted) throw new Error("workflow role launch was cancelled during initialization");
				if (input.role === "build" && (!trustedExecutable || !isAbsolute(trustedExecutable))) throw new Error("Rust status omitted its trusted executable path");
				const result = await isolated(
					input.role,
					input.task,
					ctx.cwd,
					ctx.model,
					signal,
					childStarted(ctx, status, input.role, modifying),
					(child) => childFinished(ctx, child),
					input.base,
					input.candidate,
					trustedExecutable,
					input.role === "build" ? parentServiceDigest : undefined,
					policyFrom(status),
					ctx.sessionManager.getSessionId(),
				);
				let reviewRun: string | undefined;
				if (input.role === "requirements-review" || input.role === "code-review") {
					reviewRun = randomBytes(24).toString("hex");
					reviewRuns.set(reviewRun, { role: input.role, result, base: input.base!, candidate: input.candidate! });
				}
				const terminal = { ...result, ...(reviewRun ? { reviewRun } : {}) };
				const terminalText = boundedText(JSON.stringify(terminal), policyFrom(status), "head", result.artifactPath);
				return {
					content: [{ type: "text", text: terminalText }],
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
		description: "Show canonical workflow phase, live activity, and one next action",
		handler: async (_args, ctx) => {
			const status = await workflowStatus(ctx.cwd);
			const pending = questions.current();
			let pausedWorkflows = "";
			if (!status.owner && status.openWorkflows?.length) {
				const states = await Promise.all(status.openWorkflows.map(async (workflow) => {
					const text = await readFile(resolve(ctx.cwd, "knowledge/plans/open", `${workflow.plan}.md`), "utf8");
					return `${workflow.plan} in ${text.match(/^phase:\s*(scope|build|accept)$/m)?.[1]?.toUpperCase() ?? "UNKNOWN"}`;
				}));
				pausedWorkflows = states.join(", ");
			}
			const activity = status.owner?.child_role
				? `running ${status.owner.child_role}${status.owner.child_started ? ` since ${status.owner.child_started}` : ""}`
				: pending && pending.status !== "submitted" && pending.status !== "superseded"
					? `${pending.status} questions (${Object.keys(pending.answers).length}/${pending.findings.length})`
					: status.owner ? "awaiting phase action" : pausedWorkflows ? `paused: ${pausedWorkflows}` : "unowned";
			const nextAction = status.owner?.child_role ? "Esc or /superdev-cancel to interrupt"
				: pending?.status === "paused" ? "Resume workflow questions"
					: pending?.status === "active" ? "Answer or discuss the next workflow question"
						: status.owner ? `Continue ${status.phase?.toUpperCase() ?? "workflow"}`
							: status.openWorkflows?.length ? "Run /superdev-resume and select a workflow" : "Start /superdev";
			ctx.ui.notify(`${status.phase?.toUpperCase() ?? "NO ACTIVE PHASE"} · ${activity}\nNext: ${nextAction}`, "info");
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
			if (!candidates.length) return ctx.ui.notify("No matching canonical workflow is available to resume", "warning");
			let workflow = candidates[0];
			if (candidates.length > 1) {
				if (!ctx.hasUI) return ctx.ui.notify(`human-input-required: choose one workflow from ${candidates.map((candidate) => candidate.plan).join(", ")}`, "warning");
				const selected = await ctx.ui.select("Resume which workflow?", candidates.map((candidate) => candidate.plan));
				workflow = candidates.find((candidate) => candidate.plan === selected)!;
				if (!workflow) return;
			}
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
	const pauseAfterFailure = async (ctx: any, phase: string, error: unknown): Promise<boolean> => {
		const status = await workflowStatus(ctx.cwd);
		const workflow = status.owner?.identity;
		if (status.owner) {
			try { await runSuperdev(["workflow", "cancel", "--session", status.owner.session_id], ctx.cwd, authority); }
			catch { /* Preserve the original failure as the actionable diagnostic. */ }
		}
		ctx.ui.setStatus("superdev-workflow", undefined);
		const diagnostic = boundedText(String(error), policyFrom(status));
		if (!ctx.hasUI) {
			ctx.ui.notify(`human-input-required: ${phase} interrupted; partial work was preserved. Resume explicitly. ${diagnostic}`, "error");
			return false;
		}
		const action = await ctx.ui.select(`${phase} interrupted; partial work was preserved`, ["Retry phase", "Discuss", "Cancel workflow"]);
		if (action !== "Retry phase" || !workflow) {
			ctx.ui.notify(action === "Discuss" ? `${phase} remains paused for discussion. ${diagnostic}` : `${phase} remains paused; automatic retry is disabled. ${diagnostic}`, "error");
			return false;
		}
		await runSuperdev([
			"workflow", "resume", "--session", ctx.sessionManager.getSessionId(),
			"--issue", workflow.issue, "--plan", workflow.plan,
			"--work-branch", workflow.work_branch, "--default-branch", workflow.default_branch,
		], ctx.cwd, authority);
		return true;
	};
	let startBuildPhase: ((args: string, ctx: any) => Promise<void>) | undefined;
	const runScopePhase = async (args: string, ctx: any, submitted?: QuestionState) => {
		if (cancelling || modifyingBusy || modifyingChild) return ctx.ui.notify("A modifying workflow role is already active", "error");
		modifyingBusy = true;
		const commandAbort = new AbortController();
		modifyingCommandAbort = commandAbort;
		let launchBuild = false;
		let retryScope = false;
		try {
			let correction = submitted;
			let cycle = submitted?.cycle ?? 0;
			let firstBase: string | undefined;
			while (true) {
				const initial = await workflowStatus(ctx.cwd);
				if (commandAbort.signal.aborted) return ctx.ui.notify("SCOPE interrupted; partial work was preserved", "warning");
				if (!initial.owner || initial.phase !== "scope") return ctx.ui.notify("An owned SCOPE workflow is required", "error");
				if (correction && initial.owner.last_plan_revision !== correction.candidate) throw new Error("SCOPE candidate changed; saved answers were superseded");
				if (!firstBase) {
					const resolved = await pi.exec("git", ["rev-parse", "--verify", initial.owner.scope_base_revision ?? initial.owner.identity.work_branch], { cwd: ctx.cwd });
					if (resolved.code !== 0) throw new Error("Could not resolve the pre-SCOPE revision");
					firstBase = resolved.stdout.trim();
				}
				const task = correction
					? `Apply this complete SCOPE correction batch for ${initial.owner.identity.plan}. Mechanical findings: ${JSON.stringify(correction.mechanicalFindings ?? [])}. Confirmed human answers: ${JSON.stringify(correction.answers)}.`
					: args || `Complete ${initial.owner.identity.plan} from canonical state.`;
				const scoped = await withProgress(ctx, { key: "superdev-workflow", title: `SCOPE ${initial.owner.identity.plan}`, stage: correction ? "batched correction" : "scope authoring" },
					(signal, update) => isolated("scope", task, ctx.cwd, ctx.model, signal,
						childStarted(ctx, initial, "scope", true),
						(child) => childFinished(ctx, child),
						undefined, undefined, undefined, undefined, policyFrom(initial), initial.owner!.session_id,
						(activity) => update({ stage: correction ? "batched correction" : "scope authoring", activity })));
				if (scoped.status !== "complete") return ctx.ui.notify(scoped.summary, "warning");
				await runSuperdev(["workflow", "scope-checkpoint", "--session", initial.owner.session_id, "--expected-revision", initial.owner.last_plan_revision], ctx.cwd, authority);
				const status = await workflowStatus(ctx.cwd);
				const owner = status.owner;
				if (!owner || status.phase !== "scope" || !status.canonicalPlanRevision || owner.session_id !== initial.owner.session_id) throw new Error("SCOPE ownership changed during isolated work");
				const candidateResult = await pi.exec("git", ["rev-parse", "--verify", owner.identity.work_branch], { cwd: ctx.cwd });
				const headBefore = await pi.exec("git", ["rev-parse", "HEAD"], { cwd: ctx.cwd });
				const cleanBefore = await pi.exec("git", ["status", "--porcelain"], { cwd: ctx.cwd });
				const candidate = candidateResult.stdout.trim();
				if (candidateResult.code !== 0 || headBefore.code !== 0 || cleanBefore.code !== 0 || cleanBefore.stdout.trim() || headBefore.stdout.trim() !== candidate) {
					throw new Error("requirements review requires the clean immutable candidate at HEAD");
				}
				const reviewed = await withProgress(ctx, { key: "superdev-workflow", title: `SCOPE ${owner.identity.plan}`, stage: "requirements review" },
					(signal, update) => isolated("requirements-review", `Review the complete immutable SCOPE candidate ${candidate} against baseline ${firstBase}.`, ctx.cwd, ctx.model, signal,
						childStarted(ctx, status, "requirements-review"), (child) => childFinished(ctx, child), firstBase, candidate,
						undefined, undefined, policyFrom(status), owner.session_id,
						(activity) => update({ stage: "requirements review", activity })));
				const [headAfter, cleanAfter] = await Promise.all([
					pi.exec("git", ["rev-parse", "HEAD"], { cwd: ctx.cwd }),
					pi.exec("git", ["status", "--porcelain"], { cwd: ctx.cwd }),
				]);
				if (headAfter.code !== 0 || cleanAfter.code !== 0 || cleanAfter.stdout.trim() || headAfter.stdout.trim() !== candidate) {
					throw new Error("candidate changed during requirements review; result discarded");
				}
				const completedCorrection = correction;
				if (correction) { cycle += 1; correction = undefined; }
				if (reviewed.status === "findings") {
					const substantive = (reviewed.findings ?? []).filter((finding) => finding.classification === "substantive");
					const mechanical = (reviewed.findings ?? []).filter((finding) => finding.classification === "mechanical");
					if (substantive.length) {
						const reused: QuestionState["answers"] = {};
						for (const finding of substantive) {
							const previous = completedCorrection?.findings.find((candidate) => findingFingerprint(candidate) === findingFingerprint(finding));
							const answer = previous && completedCorrection?.answers[previous.id];
							if (answer) reused[finding.id] = { ...answer, findingIds: [finding.id] };
						}
						if (substantive.every((finding) => reused[finding.id])) {
							if (cycle >= (status.maxScopeReviewCycles ?? 3)) return ctx.ui.notify(`SCOPE correction limit exhausted: ${reviewed.summary}`, "error");
							correction = { version: 1, workflow: owner.identity.plan, candidate: status.canonicalPlanRevision, status: "submitted", findings: substantive, mechanicalFindings: mechanical, cycle, answers: reused };
							continue;
						}
						questions.begin({ version: 1, workflow: owner.identity.plan, candidate: status.canonicalPlanRevision, status: "active", findings: substantive, mechanicalFindings: mechanical, cycle, answers: reused });
						ctx.ui.notify(`Requirements review found ${substantive.length} substantive and ${mechanical.length} mechanical findings. Discuss or answer them one at a time.`, "warning");
						return;
					}
					if (cycle >= (status.maxScopeReviewCycles ?? 3)) return ctx.ui.notify(`SCOPE correction limit exhausted: ${reviewed.summary}`, "error");
					correction = { version: 1, workflow: owner.identity.plan, candidate: status.canonicalPlanRevision, status: "submitted", findings: [], mechanicalFindings: mechanical, cycle, answers: {} };
					continue;
				}
				if (reviewed.status !== "clean") return ctx.ui.notify(reviewed.summary, "error");
				const reviewRun = randomBytes(24).toString("hex");
				reviewRuns.set(reviewRun, { role: "requirements-review", result: reviewed, base: firstBase, candidate });
				const evidence = await runSuperdev(["workflow", "evidence", "--session", owner.session_id, "--expected-revision", owner.last_plan_revision, "--revision", status.canonicalPlanRevision, "--kind", "scope-review", "--review-session", reviewRun, "--candidate", candidate], ctx.cwd, authority) as { result?: { last_plan_revision?: string } };
				const revision = evidence.result?.last_plan_revision;
				if (!revision) throw new Error("scope evidence response omitted the new plan revision");
				reviewRuns.delete(reviewRun);
				if (!ctx.hasUI) {
					await runSuperdev(["workflow", "cancel", "--session", owner.session_id], ctx.cwd, authority);
					return ctx.ui.notify(`human-input-required: ${owner.identity.plan} SCOPE approval is pending; resume in an interactive session`, "warning");
				}
				const action = await ctx.ui.select("Reviewed SCOPE is clean", ["Approve and start BUILD", "Approve only", "Request changes", "Discuss", "Pause", "Abandon workflow"]);
				if (!action || action === "Pause") {
					await runSuperdev(["workflow", "cancel", "--session", owner.session_id], ctx.cwd, authority);
					return ctx.ui.notify("SCOPE paused at the human gate; ownership released", "info");
				}
				if (action === "Discuss" || action === "Request changes") {
					const finding: ReviewFinding = {
						id: "human-scope-change",
						classification: "substantive",
						summary: action === "Request changes" ? "Human requested changes to reviewed SCOPE" : "Human wants to discuss reviewed SCOPE before approval",
						evidence: `The complete SCOPE candidate ${candidate} passed exhaustive requirements review.`,
						impact: "Any confirmed change requires one batched correction and complete requirements re-review before approval.",
						question: "What, if anything, should change in the reviewed scope before approval?",
						recommendation: "Describe the smallest concrete change needed, or confirm that no change is required.",
					};
					questions.begin({ version: 1, workflow: owner.identity.plan, candidate: revision, status: "active", findings: [finding], cycle, answers: {} });
					return ctx.ui.notify("Reviewed SCOPE is ready for discussion and an explicitly confirmed answer", "info");
				}
				if (action === "Abandon workflow") {
					if (!(await ctx.ui.confirm("Abandon workflow?", "Partial product work will not be integrated; approved knowledge disposition will be published."))) return ctx.ui.notify("Abandonment was not confirmed", "info");
					const reason = (await ctx.ui.input("Abandonment reason", "Why should this workflow be abandoned?"))?.trim();
					if (!reason) return ctx.ui.notify("Abandonment requires a reason", "warning");
					await runSuperdev(["workflow", "abandon", "--session", owner.session_id, "--expected-revision", revision, "--phase", "scope", "--reason", reason], ctx.cwd, authority);
					ctx.ui.setStatus("superdev-workflow", undefined);
					return ctx.ui.notify("Workflow abandoned; partial product work was not integrated", "info");
				}
				await runSuperdev(["workflow", "transition", "--session", owner.session_id, "--expected-revision", revision, "--phase", "scope", "--transition", "approve-scope"], ctx.cwd, authority);
				ctx.ui.setStatus("superdev-workflow", `BUILD: ${owner.identity.plan}`);
				launchBuild = action === "Approve and start BUILD";
				if (!launchBuild) ctx.ui.notify("SCOPE approved; say ‘start build’ when ready", "info");
				break;
			}
		} catch (error) {
			retryScope = await pauseAfterFailure(ctx, "SCOPE", error);
		} finally {
			if (modifyingCommandAbort === commandAbort) modifyingCommandAbort = undefined;
			modifyingBusy = false;
		}
		if (retryScope) return runScopePhase(args, ctx, submitted);
		if (launchBuild) {
			if (!startBuildPhase) throw new Error("BUILD driver is unavailable");
			await startBuildPhase("", ctx);
		}
	};
	continueScopeFromAnswers = (state, ctx) => runScopePhase("", ctx, state);
	pi.registerCommand("scope", {
		description: "Run SCOPE through exhaustive review and human approval",
		handler: (args, ctx) => runScopePhase(args, ctx),
	});
	let runAcceptPhase: ((args: string, ctx: any) => Promise<void>) | undefined;
	const runBuildPhase = async (args: string, ctx: any) => {
			if (cancelling || modifyingBusy || modifyingChild) return ctx.ui.notify("A modifying workflow role is already active", "error");
			modifyingBusy = true;
			const commandAbort = new AbortController();
			modifyingCommandAbort = commandAbort;
			let retryBuild = false;
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
				const built = await withProgress(ctx, { key: "superdev-workflow", title: `BUILD ${owner.identity.plan}`, stage: instruction ? "batched correction" : "implementation" },
					(signal, update) => isolated("build", instruction || `Complete ${owner.identity.plan} from canonical state.`, ctx.cwd, ctx.model, signal,
						childStarted(ctx, status, "build", true),
						(child) => childFinished(ctx, child), undefined, undefined, status.executable, parentServiceDigest,
						policyFrom(status), owner.session_id,
						(activity) => update({ stage: instruction ? "batched correction" : "implementation", activity })));
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
			const reviewed = await withProgress(ctx, { key: "superdev-workflow", title: `BUILD ${synchronizedOwner.identity.plan}`, stage: "code review" },
				(signal, update) => isolated("code-review", `Review immutable diff ${base}..${candidate} and return the required structured result.`, ctx.cwd, ctx.model, signal,
					childStarted(ctx, synchronized, "code-review"), (child) => childFinished(ctx, child), base, candidate,
					undefined, undefined, policyFrom(synchronized), synchronizedOwner.session_id,
					(activity) => update({ stage: "code review", activity })));
			const reviewRun = randomBytes(24).toString("hex");
			if (reviewed.status === "findings" && reviewed.findings?.some((finding) => finding.classification === "requires-scope")) {
				const latest = await workflowStatus(ctx.cwd);
				if (!latest.owner || latest.phase !== "build") throw new Error("BUILD ownership changed before review re-scope");
				await runSuperdev([
					"workflow", "transition", "--session", latest.owner.session_id,
					"--expected-revision", latest.owner.last_plan_revision, "--phase", "build",
					"--transition", "return-to-scope", "--feedback", JSON.stringify(reviewed.findings),
				], ctx.cwd, authority);
				const substantive = reviewed.findings.filter((finding) => finding.classification === "requires-scope").map((finding) => ({
					...finding,
					classification: "substantive" as const,
					question: finding.question ?? `How should SCOPE resolve ${finding.summary}?`,
				}));
				const mechanical = reviewed.findings.filter((finding) => finding.classification === "correctable-within-scope");
				questions.begin({ version: 1, workflow: latest.owner.identity.plan, candidate: latest.owner.last_plan_revision, status: "active", findings: substantive, mechanicalFindings: mechanical, answers: {} });
				ctx.ui.notify("Final review returned the workflow to SCOPE with the complete finding set", "warning");
				return;
			}
			if (reviewed.status !== "clean") {
				const correction = await runSuperdev([
					"workflow", "correction", "--session", synchronizedOwner.session_id,
					"--expected-revision", verifiedRevision, "--candidate", candidate,
					"--review-session", reviewRun,
					"--summary", reviewed.findings?.map((finding) => `${finding.id}: ${finding.summary}`).join("\n") ?? reviewed.summary,
				], ctx.cwd, authority) as { result?: { stalled?: boolean } };
				const stalled = correction.result?.stalled === true;
				if (stalled) return ctx.ui.notify(`Final correction limit exhausted: ${reviewed.summary}`, "error");
				ctx.ui.notify(`Final review requires correction: ${reviewed.summary}`, "warning");
				instruction = `Correct the complete immutable final-review finding set for ${synchronizedOwner.identity.plan}:\n${JSON.stringify(reviewed.findings ?? [], null, 2)}`;
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
				ctx.ui.notify("BUILD gates passed; starting ACCEPT assessment", "info");
				if (!runAcceptPhase) throw new Error("ACCEPT driver is unavailable");
				return await runAcceptPhase("", ctx);
			}
			} catch (error) {
				retryBuild = await pauseAfterFailure(ctx, "BUILD", error);
			} finally {
				if (modifyingCommandAbort === commandAbort) modifyingCommandAbort = undefined;
				modifyingBusy = false;
			}
			if (retryBuild) return runBuildPhase(args, ctx);
	};
	startBuildPhase = runBuildPhase;
	pi.registerCommand("build", {
		description: "Run BUILD through automatic ACCEPT assessment",
		handler: runBuildPhase,
	});
	const executeAcceptPhase = async (_args: string, ctx: any) => {
			const status = await workflowStatus(ctx.cwd);
			const owner = status.owner;
			if (!owner || status.phase !== "accept") return ctx.ui.notify("An owned ACCEPT workflow is required", "error");
			if (!owner.candidate_revision || !owner.verified_default_revision) throw new Error("ACCEPT state is missing candidate-bound BUILD evidence");
			const acceptHeadBefore = await pi.exec("git", ["rev-parse", "HEAD"], { cwd: ctx.cwd });
			const acceptCleanBefore = await pi.exec("git", ["status", "--porcelain"], { cwd: ctx.cwd });
			if (acceptHeadBefore.code !== 0 || acceptCleanBefore.code !== 0 || acceptCleanBefore.stdout.trim()
				|| acceptHeadBefore.stdout.trim() !== owner.candidate_revision) throw new Error("ACCEPT assessment requires the clean immutable candidate at HEAD");
			const decision = await withProgress(ctx, { key: "superdev-workflow", title: `ACCEPT ${owner.identity.plan}`, stage: "assessment" },
				(signal, update) => isolated(
					"accept",
					`Assess whether immutable candidate ${owner.candidate_revision} is ready for the parent-owned configured acceptance decision.`,
					ctx.cwd,
					ctx.model,
					signal,
					childStarted(ctx, status, "accept"),
					(child) => childFinished(ctx, child),
					owner.verified_default_revision,
					owner.candidate_revision,
					undefined,
					undefined,
					policyFrom(status),
					owner.session_id,
					(activity) => update({ stage: "assessment", activity }),
				));
			const acceptHeadAfter = await pi.exec("git", ["rev-parse", "HEAD"], { cwd: ctx.cwd });
			const acceptCleanAfter = await pi.exec("git", ["status", "--porcelain"], { cwd: ctx.cwd });
			if (acceptHeadAfter.code !== 0 || acceptCleanAfter.code !== 0 || acceptCleanAfter.stdout.trim() || acceptHeadAfter.stdout.trim() !== acceptHeadBefore.stdout.trim()) {
				throw new Error("candidate changed during ACCEPT assessment; result discarded");
			}
			if (decision.status === "findings") {
				const requiresScope = decision.findings?.some((finding) => finding.classification === "requires-scope");
				const routed = await runSuperdev([
					"workflow", "transition", "--session", owner.session_id,
					"--expected-revision", owner.last_plan_revision, "--phase", "accept",
					"--transition", requiresScope ? "reject-acceptance" : "return-to-build",
					"--feedback", JSON.stringify(decision.findings),
				], ctx.cwd, authority) as { result?: { state?: { last_plan_revision?: string } } };
				ctx.ui.setStatus("superdev-workflow", `${requiresScope ? "SCOPE" : "BUILD"}: ${owner.identity.plan}`);
				ctx.ui.notify(`ACCEPT findings returned the workflow to ${requiresScope ? "SCOPE" : "BUILD"}`, "warning");
				if (requiresScope) {
					const revision = routed.result?.state?.last_plan_revision;
					if (!revision) throw new Error("ACCEPT-to-SCOPE routing omitted the new plan revision");
					const substantive = (decision.findings ?? []).filter((finding) => finding.classification === "requires-scope").map((finding) => ({
						...finding,
						classification: "substantive" as const,
						question: finding.question ?? `How should SCOPE resolve ${finding.summary}?`,
						recommendation: finding.recommendation ?? "Choose the smallest intent change that resolves the acceptance defect.",
					}));
					const mechanical = (decision.findings ?? []).filter((finding) => finding.classification === "correctable-within-scope");
					questions.begin({ version: 1, workflow: owner.identity.plan, candidate: revision, status: "active", findings: substantive, mechanicalFindings: mechanical, answers: {} });
					return;
				}
				if (startBuildPhase) {
					modifyingBusy = false;
					modifyingCommandAbort = undefined;
					return await startBuildPhase(`Correct this complete ACCEPT finding set: ${JSON.stringify(decision.findings)}`, ctx);
				}
				throw new Error("BUILD driver is unavailable for ACCEPT correction");
			}
			if (decision.status !== "complete") return ctx.ui.notify(decision.summary, "error");
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
			const humanAcceptanceRequired = requiresHumanAcceptance(status.humanAcceptanceRequired);
			if (humanAcceptanceRequired) {
				if (!ctx.hasUI) {
					await runSuperdev(["workflow", "cancel", "--session", owner.session_id], ctx.cwd, authority);
					return ctx.ui.notify(`human-input-required: ${owner.identity.plan} ACCEPT decision is pending; resume in an interactive session`, "warning");
				}
				const action = await ctx.ui.select("ACCEPT assessment is clean", ["Accept", "Request changes", "Discuss", "Pause"]);
				if (!action || action === "Pause") {
					await runSuperdev(["workflow", "cancel", "--session", owner.session_id], ctx.cwd, authority);
					ctx.ui.setStatus("superdev-workflow", undefined);
					return ctx.ui.notify("ACCEPT paused at the human decision; ownership released", "info");
				}
				if (action !== "Accept") {
					const finding: ReviewFinding = {
						id: "human-acceptance-change",
						classification: "substantive",
						summary: "Human requested changes at final acceptance",
						evidence: "The immutable candidate reached the configured human acceptance gate.",
						impact: "The request must be clarified and routed to BUILD or SCOPE before more work starts.",
						question: "What change is required? Confirm it as `BUILD correction: …` when intent is unchanged, or `Return to SCOPE: …` when requirements, architecture, API, or acceptance intent changes.",
						recommendation: "Discuss the change, then confirm both its concrete wording and destination.",
					};
					questions.begin({ version: 1, workflow: owner.identity.plan, candidate: owner.last_plan_revision, originPhase: "accept", status: "active", findings: [finding], answers: {} });
					return ctx.ui.notify("Acceptance change request is ready for discussion and explicit routing confirmation", "warning");
				}
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
	};
	runAcceptPhase = async (args, ctx) => {
		while (true) {
			try { return await executeAcceptPhase(args, ctx); }
			catch (error) { if (!(await pauseAfterFailure(ctx, "ACCEPT", error))) return; }
		}
	};
	continueAcceptRequest = async (state, ctx) => {
		const status = await workflowStatus(ctx.cwd);
		const owner = status.owner;
		if (!owner || status.phase !== "accept" || owner.last_plan_revision !== state.candidate) throw new Error("ACCEPT decision candidate changed; restart assessment");
		const answer = state.answers[state.findings[0]?.id]?.answer;
		if (!answer) throw new Error("confirmed acceptance change request is missing");
		const toBuild = /^build correction\s*:/i.test(answer.trim());
		const toScope = /^return to scope\s*:/i.test(answer.trim());
		if (!toBuild && !toScope) throw new Error("confirmed acceptance request must name `BUILD correction:` or `Return to SCOPE:`");
		await runSuperdev([
			"workflow", "transition", "--session", owner.session_id,
			"--expected-revision", owner.last_plan_revision, "--phase", "accept",
			"--transition", toBuild ? "return-to-build" : "reject-acceptance",
			"--feedback", answer,
		], ctx.cwd, authority);
		if (toBuild) {
			if (!startBuildPhase) throw new Error("BUILD driver is unavailable");
			return await startBuildPhase(`Apply confirmed ACCEPT correction: ${answer}`, ctx);
		}
		return await runScopePhase(`Apply confirmed acceptance scope change: ${answer}`, ctx);
	};
	pi.registerCommand("accept", {
		description: "Assess ACCEPT and integrate after configured human authority",
		handler: (args, ctx) => runAcceptPhase!(args, ctx),
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

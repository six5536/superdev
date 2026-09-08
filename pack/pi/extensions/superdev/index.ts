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
import { registerPhaseDrivers, type PhaseRuntime } from "./lib/phases.ts";
import { withProgress } from "./lib/progress.ts";
import { registerWorkflowQuestions, type QuestionState } from "./lib/questions.ts";
import { parseLegacyRoleResult, roleResultSchema, validateRoleResult, type Role, type RoleResult } from "./lib/review.ts";

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

export async function isolated(
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
		let submissionStarts = 0;
		let submissionEnds = 0;
		let submissionErrors = 0;
		let stdoutBytes = 0;
		let parsedEvents = 0;
		let malformedLines = 0;
		let assistantEnds = 0;
		let lastAssistantStopReason: string | undefined;
		let lastEventType: string | undefined;
		const eventCounts: Record<string, number> = {};
		const toolStarts: Record<string, number> = {};
		const submissionDiagnostics: string[] = [];
		const startedAt = Date.now();
		let settled = false;
		let timedOut = false;
		const increment = (counts: Record<string, number>, raw: unknown) => {
			const key = String(raw ?? "unknown").replace(/[^a-zA-Z0-9_.-]/g, "_").slice(0, 80) || "unknown";
			if (Object.hasOwn(counts, key) || Object.keys(counts).length < 40) counts[key] = (counts[key] ?? 0) + 1;
		};
		const writeDiagnostic = async (outcome: string, code: number | null, error?: unknown) => artifact.writeDiagnostic({
			version: 1,
			role,
			outcome,
			exitCode: code,
			timedOut,
			cancelled: Boolean(signal?.aborted),
			startedAt: new Date(startedAt).toISOString(),
			finishedAt: new Date().toISOString(),
			elapsedMs: Date.now() - startedAt,
			stdoutBytes,
			parsedEvents,
			malformedLines,
			eventCounts,
			toolStarts,
			assistantEnds,
			...(lastAssistantStopReason ? { lastAssistantStopReason } : {}),
			lastEventType,
			submission: { starts: submissionStarts, ends: submissionEnds, successful: submissions, errors: submissionErrors, payloadPresent: submission !== undefined, diagnostics: submissionDiagnostics },
			stderr: { ...artifact.stderrSummary(), path: artifact.stderrPath },
			...(error === undefined ? {} : { error: String(error).slice(0, 2_048) }),
		});
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
			try { event = JSON.parse(line); } catch { malformedLines += 1; return; }
			parsedEvents += 1;
			lastEventType = String(event.type ?? "unknown").slice(0, 80);
			increment(eventCounts, event.type);
			if (event.type === "message_end" && event.message?.role === "assistant") {
				assistantEnds += 1;
				if (typeof event.message.stopReason === "string") lastAssistantStopReason = event.message.stopReason.slice(0, 160);
			}
			if (event.type === "tool_execution_start") {
				increment(toolStarts, event.toolName);
				if (event.toolName === "superdev_submit_result") submissionStarts += 1;
				const path = typeof event.args?.path === "string" ? event.args.path : "";
				const target = path && !isAbsolute(path) && !path.split(/[\\/]/).includes("..") ? ` ${path.slice(0, 160)}` : "";
				onActivity?.(`${String(event.toolName ?? "tool").replace(/[^a-zA-Z0-9_-]/g, "").slice(0, 80)}${target}`);
			}
			if (event.type === "tool_execution_end" && event.toolName === "superdev_submit_result") {
				submissionEnds += 1;
				if (event.isError) {
					submissionErrors += 1;
					const diagnostic = JSON.stringify(event.result?.content ?? event.error ?? "submission tool returned an error");
					submissionDiagnostics.push(diagnostic.slice(0, 2_048));
				} else {
					submissions += 1;
					submission = event.result?.details?.superdevResult;
					if (submission === undefined) submissionDiagnostics.push("successful submission event omitted details.superdevResult");
				}
			}
		};
		const stop = () => stopProcess(child);
		const timeout = setTimeout(() => { timedOut = true; stop(); }, policyTimeoutMs(policy));
		timeout.unref();
		child.stdout.setEncoding("utf8");
		child.stderr.setEncoding("utf8");
		child.stdout.on("data", (chunk: string) => {
			stdoutBytes += Buffer.byteLength(chunk);
			pending += chunk;
			if (Buffer.byteLength(pending) > policy.maxArtifactBytes) {
				stop();
				return finish(async () => {
					await onClose?.(child);
					await artifact.finishStderr();
					await writeDiagnostic("stdout-overflow", null, `${role} emitted an over-cap event`);
					reject(new Error(`isolated-output-overflow: ${role} emitted an over-cap event; diagnostics: ${artifact.diagnosticPath}`));
				});
			}
			let newline;
			while ((newline = pending.indexOf("\n")) >= 0) {
				consume(pending.slice(0, newline));
				pending = pending.slice(newline + 1);
			}
		});
		child.stderr.on("data", (chunk: string) => {
			if (!artifact.writeStderr(chunk)) {
				child.stderr.pause();
				artifact.onStderrDrain(() => child.stderr.resume());
			}
			stderrTail = (stderrTail + chunk).slice(-Math.max(policy.maxContextBytes * 2, 16_384));
		});
		child.on("error", (error) => finish(async () => {
			await onClose?.(child);
			await artifact.finishStderr();
			await writeDiagnostic("spawn-error", null, error);
			reject(new Error(`${String(error)}; diagnostics: ${artifact.diagnosticPath}`));
		}));
		child.on("close", (code) => finish(async () => {
			await onClose?.(child);
			if (pending) consume(pending);
			await artifact.finishStderr();
			if (timedOut) {
				await writeDiagnostic("timeout", code, `timed out after ${policyTimeoutMs(policy) / 1_000}s`);
				return reject(new Error(`isolated role timed out after ${policyTimeoutMs(policy) / 1_000}s; diagnostics: ${artifact.diagnosticPath}`));
			}
			if (signal?.aborted) {
				await writeDiagnostic("cancelled", code, `${role} role was cancelled`);
				return reject(new Error(`isolated ${role} role was cancelled; diagnostics: ${artifact.diagnosticPath}`));
			}
			if (code !== 0) {
				const diagnostic = boundedText(stderrTail.trim() || `${role} exited ${code}`, policy, "tail", artifact.stderrPath);
				await writeDiagnostic("nonzero-exit", code, diagnostic);
				return reject(new Error(`${diagnostic}; diagnostics: ${artifact.diagnosticPath}`));
			}
			if (submissions !== 1 || submission === undefined) {
				const reason = submissions !== 1 ? `observed ${submissions} successful terminal submissions` : "successful submission omitted its typed payload";
				await writeDiagnostic("terminal-protocol-failure", code, reason);
				return reject(new Error(`${role} must submit exactly one typed terminal result (${reason}); diagnostics: ${artifact.diagnosticPath}`));
			}
			try {
				const result = validateRoleResult(role, submission, {
					maxBytes: policy.maxReviewStateBytes ?? 262_144,
					maxFindings: policy.maxReviewFindings ?? 100,
				});
				await artifact.writeResult(result);
				await writeDiagnostic("complete", code);
				accept({ ...result, artifactPath: artifact.resultPath });
			} catch (error) {
				await writeDiagnostic("invalid-terminal-result", code, error);
				reject(new Error(`${String(error)}; diagnostics: ${artifact.diagnosticPath}`));
			}
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
	const runtime: PhaseRuntime = { cancelling: false, modifyingBusy: false };
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
		busy?: boolean;
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
		const service = await ensureParentService(cwd);
		const result = await runPinnedSuperdev(service.path, service.digest, ["workflow", "status", "--json"], cwd);
		if (result.code !== 0) throw new Error(boundedText(result.stderr.trim() || "workflow status failed", defaultOutputPolicy));
		let response: any;
		try { response = JSON.parse(result.stdout); }
		catch { throw new Error("superdev returned invalid workflow status JSON"); }
		if (response.protocol !== "superdev-workflow/v2") throw new Error(`unsupported workflow protocol ${String(response.protocol)}`);
		if (!response.result || typeof response.result !== "object") throw new Error("superdev workflow status omitted its result");
		const status = response.result as WorkflowStatus;
		reviewStateLimit = status.maxReviewStateBytes ?? reviewStateLimit;
		reviewFindingLimit = status.maxReviewFindings ?? reviewFindingLimit;
		status.executable = service.path;
		return status;
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
	let phaseContinuations: any;
	const questions = registerWorkflowQuestions(pi, {
		maxBytes: () => reviewStateLimit,
		exposeTool: false,
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
			if (!phaseContinuations) throw new Error("workflow phase continuation is unavailable");
			if (state.originPhase === "accept") return await phaseContinuations.continueAcceptRequest(state, ctx);
			if (state.originPhase === "build") return await phaseContinuations.continueBuildDecision(state, ctx);
			await phaseContinuations.continueScopeFromAnswers(state, ctx);
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
		let latest = await workflowStatus(cwd);
		for (let attempt = 0; latest.busy && attempt < 100; attempt += 1) {
			await new Promise((done) => setTimeout(done, 25));
			latest = await workflowStatus(cwd);
		}
		if (latest.busy) throw new Error("workflow transaction remained busy while clearing child activity");
		if (!latest.owner || !expectedPid || latest.owner.child_pid !== expectedPid) return;
		await runSuperdev(["workflow", "activity-finish", "--session", latest.owner.session_id, "--expected-revision", latest.owner.last_plan_revision], cwd, authority);
	};
	const activityRegistrations = new Map<ChildProcess, Promise<void>>();
	const activityRegistrationErrors = new Map<ChildProcess, unknown>();
	const childStarted = (ctx: any, status: WorkflowStatus, role: Role, modifying = false) => (child: ChildProcess) => {
		children.add(child);
		if (modifying) runtime.modifyingChild = child;
		const registration = recordChildStart(ctx.cwd, status, role, child).catch((error) => {
			activityRegistrationErrors.set(child, error);
			stopChild(child);
		});
		activityRegistrations.set(child, registration);
	};
	const childFinished = async (ctx: any, child: ChildProcess) => {
		children.delete(child);
		if (runtime.modifyingChild === child) runtime.modifyingChild = undefined;
		const registration = activityRegistrations.get(child) ?? Promise.resolve();
		activityRegistrations.delete(child);
		await registration;
		const startError = activityRegistrationErrors.get(child);
		activityRegistrationErrors.delete(child);
		if (startError) throw new Error(`Could not record child activity: ${String(startError)}`);
		try {
			await recordChildFinish(ctx.cwd, child.pid);
		} catch (error) {
			throw new Error(`Could not clear child activity: ${String(error)}`);
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
		const internalTools = new Set(["superdev_workflow_control", "superdev_workflow_questions"]);
		pi.setActiveTools(pi.getActiveTools().filter((tool: string) => !internalTools.has(tool)));
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
			if ((humanAction || acceptanceRequiresHuman) && !ctx.hasUI) {
				throw new Error(`human-input-required: interactive confirmation is required for ${input.action}; resume this workflow in an interactive Pi session`);
			}
			if ((humanAction || acceptanceRequiresHuman) && !(await ctx.ui.confirm(
				`${input.action}?`,
				input.action === "abandon" ? "Partial product work will not be integrated." : "This records an authoritative human workflow decision.",
			))) throw new Error("human workflow decision was not approved");
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
			if (input.action === "resume" && questions.current()?.status === "paused") await questions.resume(ctx);
			if ((input.action === "record-scope-review" || input.action === "record-final-evidence") && input.reviewRun) reviewRuns.delete(input.reviewRun);
			return { content: [{ type: "text", text: JSON.stringify(result) }], details: { result, humanGated: humanAction || acceptanceRequiresHuman } };
		},
	});

	pi.registerTool({
		name: "superdev_isolated_role",
		label: "Superdev isolated filing",
		description: "Prepare one bounded filing in a fresh isolated Pi process",
		parameters: Type.Object({
			role: StringEnum(["file"] as const),
			task: Type.String({ description: "Bounded filing task and all input the isolated role needs" }),
		}),
		execute: async (_id, input, signal, updateTool, ctx) => {
			if (input.role !== "file") throw new Error("Direct phase-role invocation is internal; use an explicit workflow skill.");
			if (runtime.cancelling) throw new Error("workflow cancellation is in progress");
			const modifying = !readOnly.has(input.role) && input.role !== "file";
			if (modifying && (runtime.cancelling || runtime.modifyingBusy || runtime.modifyingChild)) throw new Error("one modifying workflow child is already active");
			if (modifying) runtime.modifyingBusy = true;
			try {
				const status = await workflowStatus(ctx.cwd);
				const trustedExecutable = input.role === "build" ? status.executable : undefined;
				if (runtime.cancelling || signal.aborted) throw new Error("workflow role launch was cancelled during initialization");
				if (input.role === "build" && (!trustedExecutable || !isAbsolute(trustedExecutable))) throw new Error("Rust status omitted its trusted executable path");
				const stage = ({
					scope: "scope authoring",
					"requirements-review": "requirements review",
					build: "implementation",
					"code-review": "code review",
					accept: "acceptance assessment",
					file: "filing",
				} as const)[input.role];
				const plan = status.owner?.identity.plan;
				const startedAt = Date.now();
				let currentActivity = "starting isolated process";
				const emitToolProgress = () => updateTool?.({
					content: [{ type: "text", text: `${stage} · ${currentActivity} · ${Math.floor((Date.now() - startedAt) / 1000)}s elapsed · Esc cancels` }],
					details: { role: input.role, stage, activity: currentActivity, running: true },
				});
				emitToolProgress();
				const progressTimer = setInterval(emitToolProgress, 1_000);
				progressTimer.unref?.();
				const result = await withProgress(
					ctx,
					{ key: "superdev-workflow", title: `${input.role.toUpperCase()}${plan ? ` ${plan}` : ""}`, stage },
					(progressSignal, update) => isolated(
						input.role,
						input.task,
						ctx.cwd,
						ctx.model,
						AbortSignal.any([signal, progressSignal]),
						childStarted(ctx, status, input.role, modifying),
						(child) => childFinished(ctx, child),
						input.base,
						input.candidate,
						trustedExecutable,
						input.role === "build" ? parentServiceDigest : undefined,
						policyFrom(status),
						ctx.sessionManager.getSessionId(),
						(activity) => {
							currentActivity = activity;
							update({ stage, activity });
							emitToolProgress();
						},
					),
				).finally(() => clearInterval(progressTimer));
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
				if (modifying) runtime.modifyingBusy = false;
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
			const activity = status.busy ? "repository workflow transaction is busy"
				: status.owner?.child_role
				? `running ${status.owner.child_role}${status.owner.child_started ? ` since ${status.owner.child_started}` : ""}`
				: pending && pending.status !== "submitted" && pending.status !== "superseded"
					? `${pending.status} questions (${Object.keys(pending.answers).length}/${pending.findings.length})`
					: status.owner ? "awaiting phase action" : pausedWorkflows ? `paused: ${pausedWorkflows}` : "unowned";
			const nextAction = status.busy ? "Wait for the current workflow transaction, then check status again"
				: status.owner?.child_role ? "Esc or /superdev-cancel to interrupt"
				: pending?.status === "paused" ? "Resume workflow questions"
					: pending?.status === "active" ? "Answer or discuss the next workflow question"
						: status.owner ? `Continue ${status.phase?.toUpperCase() ?? "workflow"}`
							: status.openWorkflows?.length ? "Run /superdev-resume and select a workflow" : "Start /superdev";
			ctx.ui.notify(`${status.busy ? "TRANSACTION BUSY" : status.phase?.toUpperCase() ?? "NO ACTIVE PHASE"} · ${activity}\nNext: ${nextAction}`, "info");
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
			if (questions.current()?.status === "paused") await questions.resume(ctx);
			const resumed = await workflowStatus(ctx.cwd);
			ctx.ui.setStatus("superdev-workflow", `${resumed.phase?.toUpperCase() ?? "WORKFLOW"}: ${workflow.plan}`);
			ctx.ui.notify(`Resumed ${workflow.plan} from canonical ${resumed.phase?.toUpperCase() ?? "workflow"} state`, "info");
		},
	});
	phaseContinuations = registerPhaseDrivers({
		pi, runSuperdev, workflowStatus, authority, policyFrom, questions, isolated, childStarted, childFinished,
		reviewRuns, parentServiceDigest, requiresHumanAcceptance, runtime,
	});
	pi.registerTool({
		name: "superdev_run_phase",
		label: "Run Superdev phase",
		description: "Run or continue one workflow phase using human-level operations; ownership and revision mechanics remain internal",
		parameters: Type.Object({
			phase: StringEnum(["scope", "build", "accept"] as const),
			action: StringEnum(["run", "retry", "inspect", "record-answer", "revise-answer", "submit-answers", "approve", "cancel"] as const),
			intent: Type.Optional(Type.String()),
			issue: Type.Optional(Type.String()),
			plan: Type.Optional(Type.String()),
			workBranch: Type.Optional(Type.String()),
			defaultBranch: Type.Optional(Type.String()),
			findingIds: Type.Optional(Type.Array(Type.String())),
			answer: Type.Optional(Type.String()),
			destination: Type.Optional(StringEnum(["scope", "build"] as const)),
			offset: Type.Optional(Type.Number({ minimum: 0 })),
			limit: Type.Optional(Type.Number({ minimum: 1, maximum: 20 })),
		}),
		executionMode: "sequential",
		execute: async (_id, input, _signal, _update, ctx) => {
			const respond = (result: Record<string, unknown>) => ({ content: [{ type: "text", text: JSON.stringify(result) }], details: result });
			let status: WorkflowStatus | undefined;
			const failed = (failedStage: string, error: unknown) => {
				const rawDiagnostic = String(error);
				const diagnosticPath = rawDiagnostic.match(/diagnostics:\s*([^;\n]+)/)?.[1]?.trim();
				return respond({
					status: "failed",
					phase: input.phase,
					failedStage,
					diagnostic: boundedText(rawDiagnostic, policyFrom(status ?? {})),
					...(diagnosticPath ? { diagnosticPath } : {}),
					partialWorkPreserved: true,
					recoveryOperations: ["retry", "cancel"],
					recommendation: "Inspect and discuss the diagnostic before selecting an explicit recovery operation.",
				});
			};
			try {
				status = await workflowStatus(ctx.cwd);
			if (input.action === "inspect") {
				const questionState = questions.snapshot(input.offset, input.limit);
				const publicQuestions = questionState ? { ...questionState, candidate: undefined } : null;
				return respond({
					status: status.owner ? "active" : status.openWorkflows?.length ? "paused" : "idle",
					phase: status.phase ?? null,
					workflow: status.owner?.identity,
					openWorkflows: status.openWorkflows ?? [],
					questions: publicQuestions,
				});
			}
			if (input.action === "cancel") {
				if (questions.current()?.status === "active") {
					await questions.operate({ action: "pause" }, ctx);
					return respond({ status: "paused", phase: input.phase, partialWorkPreserved: true });
				}
				if (!status.owner) return respond({ status: "idle", phase: null });
				await runSuperdev(["workflow", "cancel", "--session", status.owner.session_id], ctx.cwd, authority);
				return respond({ status: "paused", phase: status.phase, partialWorkPreserved: true });
			}
			if (["record-answer", "revise-answer", "submit-answers"].includes(input.action)) {
				let pending = questions.current();
				if (!pending && input.action === "record-answer") {
					if (!status.owner || status.phase !== input.phase || !["scope", "accept"].includes(input.phase)) throw new Error("record-answer requires active findings or a SCOPE/ACCEPT approval gate");
					const scope = input.phase === "scope";
					if (!scope && !input.destination) throw new Error("an ACCEPT change requires destination `build` or `scope`");
					questions.begin({
						version: 1,
						workflow: status.owner.identity.plan,
						candidate: status.owner.last_plan_revision,
						originPhase: input.phase,
						status: "active",
						findings: [{
							id: scope ? "human-scope-change" : "human-acceptance-change",
							classification: "substantive",
							summary: scope ? "Human requested a revision to reviewed SCOPE" : "Human requested a routed acceptance change",
							evidence: `The reviewed ${input.phase.toUpperCase()} candidate reached its human approval gate.`,
							impact: scope ? "The plan requires one correction and complete requirements re-review." : `The confirmed change returns the workflow to ${input.destination?.toUpperCase()}.`,
							question: "What exact change should the workflow apply?",
							recommendation: "Confirm the smallest concrete change that resolves the concern.",
						}],
						answers: {},
					});
					pending = questions.current();
				}
				if (!pending || (pending.originPhase ?? "scope") !== input.phase) throw new Error(`no ${input.phase.toUpperCase()} question queue is active`);
				if (input.action === "revise-answer") {
					const revised = await questions.operate({ action: "revise", findingId: input.findingIds?.[0] }, ctx);
					return respond({ status: "answer-reopened", phase: input.phase, reopened: revised.details?.reopened ?? [] });
				}
				if (input.action === "submit-answers") {
					runtime.lastOutcome = undefined;
					try {
						const submitted = await questions.operate({ action: "submit" }, ctx);
						return respond(runtime.lastOutcome ?? { status: submitted.details?.submitted ? "answers-submitted" : "submission-declined", phase: input.phase });
					} catch (error) {
						return failed(`${input.phase} answer continuation`, error);
					}
				}
				const answer = input.phase === "accept" && pending.findings[0]?.id === "human-acceptance-change"
					? `${input.destination === "build" ? "BUILD correction" : "Return to SCOPE"}: ${input.answer ?? ""}`
					: input.answer;
				const recorded = await questions.operate({ action: "propose-answer", findingIds: input.findingIds?.length ? input.findingIds : [pending.findings[0]?.id], proposedAnswer: answer }, ctx);
				return respond({ status: recorded.details?.confirmed ? "answer-recorded" : recorded.details?.humanInputRequired ? "human-input-required" : "answer-declined", phase: input.phase, answered: recorded.details?.answered ?? [] });
			}
			if (!status.owner) {
				const requested = input.issue && input.plan && input.workBranch ? {
					issue: input.issue,
					plan: input.plan,
					work_branch: input.workBranch,
					default_branch: input.defaultBranch ?? "main",
				} : status.openWorkflows?.length === 1 ? status.openWorkflows[0] : undefined;
				if (!requested) return respond({
					status: "selection-required",
					phase: input.phase,
					openWorkflows: status.openWorkflows ?? [],
					recommendation: "Select one exact issue, plan, and issue-derived work branch semantically.",
				});
				const existing = status.openWorkflows?.find((workflow) => workflow.issue === requested.issue && workflow.plan === requested.plan && workflow.work_branch === requested.work_branch);
				if (!existing && input.phase !== "scope") throw new Error("Only SCOPE can establish a new workflow.");
				const identity = ["--session", ctx.sessionManager.getSessionId(), "--issue", requested.issue, "--plan", requested.plan, "--work-branch", requested.work_branch, "--default-branch", existing?.default_branch ?? requested.default_branch];
				try {
					await runSuperdev(["workflow", existing ? "resume" : "start", ...identity], ctx.cwd, authority);
					status = await workflowStatus(ctx.cwd);
				} catch (error) {
					return failed(existing ? "workflow resume" : "workflow start", error);
				}
			}
			if (status.phase !== input.phase) return respond({
				status: "routed",
				phase: status.phase ?? null,
				workflow: status.owner?.identity,
				recommendation: `Invoke /skill:${status.phase ?? "scope"} for the canonical workflow phase.`,
			});
			if (questions.current()?.status === "paused") {
				if (input.action !== "retry") return respond({ status: "paused", phase: input.phase, partialWorkPreserved: true, recoveryOperations: ["retry", "cancel"] });
				await questions.resume(ctx);
				return respond({ status: "findings", phase: input.phase, questions: questions.current()?.findings.length ?? 0 });
			}
			if (input.action === "approve") {
				if (!status.owner || !["scope", "accept"].includes(input.phase)) throw new Error("approve is available only at a SCOPE or ACCEPT human gate");
				const pending = questions.current();
				if (pending && pending.status === "active" && !pending.findings.every((finding) => finding.id.startsWith("human-"))) {
					throw new Error("Resolve the active review findings before approval.");
				}
				if (input.phase === "accept" && (runtime.lastOutcome?.status !== "ready-for-approval" || runtime.lastOutcome?.expectedRevision !== status.owner.last_plan_revision)) {
					throw new Error("Run the current ACCEPT assessment before approving it.");
				}
				if (!ctx.hasUI) return respond({ status: "human-input-required", phase: input.phase, recommendation: `${input.phase.toUpperCase()} approval requires an interactive Pi session.` });
				if (!(await ctx.ui.confirm(`Approve reviewed ${input.phase.toUpperCase()}?`, input.phase === "scope" ? `Approve ${status.owner.identity.plan} and advance it to BUILD?` : `Accept ${status.owner.identity.plan} and integrate it locally?`))) {
					return respond({ status: "approval-declined", phase: input.phase });
				}
				if (pending?.status === "active") questions.supersede("the human approved the unchanged reviewed candidate");
				try {
					if (input.phase === "scope") {
						await runSuperdev(["workflow", "transition", "--session", status.owner.session_id, "--expected-revision", status.owner.last_plan_revision, "--phase", "scope", "--transition", "approve-scope"], ctx.cwd, authority);
						return respond({ status: "approved", phase: "build", workflow: status.owner.identity });
					}
					await phaseContinuations.approveAccept(status, ctx);
					return respond({ status: "accepted", phase: null, workflow: status.owner.identity });
				} catch (error) {
					return failed(`${input.phase} approval`, error);
				}
			}
			if (runtime.cancelling || runtime.modifyingBusy || runtime.modifyingChild) {
				return respond({ status: "busy", phase: input.phase, recommendation: "Wait for the active operation to finish or cancel it explicitly." });
			}
			runtime.lastOutcome = undefined;
			const runner = input.phase === "scope" ? phaseContinuations.runScopePhase : input.phase === "build" ? phaseContinuations.runBuildPhase : phaseContinuations.runAcceptPhase;
			await runner(input.intent ?? "", ctx);
			if (runtime.lastOutcome) return respond(runtime.lastOutcome);
			const after = await workflowStatus(ctx.cwd);
			const pending = questions.current();
			return respond({ status: pending?.status === "active" ? "findings" : "complete", phase: after.phase, workflow: after.owner?.identity, questions: pending?.status === "active" ? pending.findings.length : 0 });
			} catch (error) {
				return failed(`${input.phase} ${input.action}`, error);
			}
		},
	});
	pi.registerCommand("superdev-cancel", {
		description: "Pause the workflow and release transient ownership",
		handler: async (_args, ctx) => {
			if (runtime.cancelling) return ctx.ui.notify("Workflow cancellation is already in progress", "warning");
			runtime.cancelling = true;
			try {
			runtime.modifyingCommandAbort?.abort();
			const stopping = [...children];
			for (const child of stopping) stopChild(child);
			await Promise.race([
				Promise.all(stopping.map((child) => new Promise<void>((done) => child.once("close", () => done())))),
				new Promise<void>((done) => setTimeout(done, 2_500)),
			]);
			for (let attempt = 0; runtime.modifyingBusy && attempt < 100; attempt += 1) {
				await new Promise((done) => setTimeout(done, 25));
			}
			if (runtime.modifyingBusy) return ctx.ui.notify("Could not safely pause while a modifying command is still shutting down", "error");
			const session = ctx.sessionManager.getSessionId();
			try {
				await runSuperdev(["workflow", "cancel", "--session", session], ctx.cwd, authority);
				ctx.ui.notify("Workflow paused; canonical phase unchanged", "info");
			} catch (error) {
				ctx.ui.notify(String(error), "error");
			}
			} finally {
				runtime.cancelling = false;
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

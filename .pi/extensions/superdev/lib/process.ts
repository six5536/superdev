import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { createHash, randomBytes } from "node:crypto";
import { constants } from "node:fs";
import { access, chmod, mkdtemp, open, readFile, rm, type FileHandle } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { StringEnum } from "@earendil-works/pi-ai";
import { Type, type Static } from "typebox";
import { IsolatedArtifact, boundedText, cleanupArtifacts, type OutputPolicy } from "./output.ts";
import { parseLegacyRoleResult, validateRoleResult, type RoleResult, type Role } from "./review.ts";
import { pinService } from "./service-pin.ts";

const here = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const schema = Type.Object({
	role: StringEnum(["scope", "requirements-review", "build", "code-review", "accept"] as const),
	task: Type.String({ description: "Bounded task and all input the isolated role needs" }),
	base: Type.Optional(Type.String({ description: "Immutable review base" })),
	candidate: Type.Optional(Type.String({ description: "Immutable review candidate" })),
});
type Input = Static<typeof schema>;

export const readOnly = new Set<Role>(["requirements-review", "code-review", "accept"]);

export function parseRoleResult(role: Input["role"], answer: string): RoleResult {
	return parseLegacyRoleResult(role, answer);
}

export function killProcessTree(pid: number) {
	if (process.platform === "win32") {
		spawnSync(`${process.env.SystemRoot ?? "C:\\Windows"}\\System32\\taskkill.exe`, ["/F", "/T", "/PID", String(pid)], { windowsHide: true, stdio: "ignore" });
		return;
	}
	try { process.kill(-pid, "SIGKILL"); }
	catch { try { process.kill(pid, "SIGKILL"); } catch { /* Already exited. */ } }
}

export function stopProcess(child: ChildProcess) {
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

export async function ensureParentService(cwd: string): Promise<{ path: string; digest: string }> {
	if (!parentServicePin) {
		parentServicePin = (async () => {
			const service = await pinService(await resolveServicePath(), cwd);
			parentServicePath = service.path;
			parentServiceDigest = service.digest;
			return { path: service.path, digest: service.digest };
		})().catch((error) => { parentServicePin = undefined; throw error; });
	}
	return parentServicePin;
}

export async function runSuperdev(args: string[], cwd: string, authority: string, signal?: AbortSignal): Promise<unknown> {
	const service = await ensureParentService(cwd);
	const result = await runPinnedSuperdev(service.path, service.digest, args, cwd, signal, { SUPERDEV_UI_AUTHORITY: authority });
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
	if (role === "build") return "read,edit,write,sokf_search,sokf_graph,superdev_build_exec,superdev_submit_result";
	return "read,edit,write,sokf_search,sokf_graph,superdev_submit_result";
}

export const defaultOutputPolicy: OutputPolicy = {
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
		let submissionDuplicates = 0;
		let stdoutBytes = 0;
		let parsedEvents = 0;
		let malformedLines = 0;
		let turns = 0;
		let assistantEnds = 0;
		let lastAssistantStopReason: string | undefined;
		let lastEventType: string | undefined;
		const eventCounts: Record<string, number> = {};
		const toolStarts: Record<string, number> = {};
		const submissionDiagnostics: string[] = [];
		const terminalTimeline: Array<Record<string, unknown>> = [];
		const recordTerminalEvent = (event: Record<string, unknown>) => {
			terminalTimeline.push({ sequence: parsedEvents, turn: turns, assistant: assistantEnds, ...event });
			if (terminalTimeline.length > 128) terminalTimeline.shift();
		};
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
			submission: { starts: submissionStarts, ends: submissionEnds, successful: submissions, errors: submissionErrors, duplicates: submissionDuplicates, payloadPresent: submission !== undefined, diagnostics: submissionDiagnostics },
			terminalTimeline,
			events: artifact.stdoutSummary(),
			stderr: artifact.stderrSummary(),
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
			if (event.type === "turn_start") turns += 1;
			if (event.type === "message_start" && event.message?.customType) {
				recordTerminalEvent({ type: "custom-message", customType: String(event.message.customType).slice(0, 80) });
			}
			if (event.type === "message_end" && event.message?.role === "assistant") {
				assistantEnds += 1;
				if (typeof event.message.stopReason === "string") lastAssistantStopReason = event.message.stopReason.slice(0, 160);
			}
			if (event.type === "tool_execution_start") {
				increment(toolStarts, event.toolName);
				recordTerminalEvent({ type: "tool-start", tool: String(event.toolName ?? "unknown").slice(0, 80) });
				if (event.toolName === "superdev_submit_result") submissionStarts += 1;
				const path = typeof event.args?.path === "string" ? event.args.path : "";
				const target = path && !isAbsolute(path) && !path.split(/[\\/]/).includes("..") ? ` ${path.slice(0, 160)}` : "";
				onActivity?.(`${String(event.toolName ?? "tool").replace(/[^a-zA-Z0-9_-]/g, "").slice(0, 80)}${target}`);
			}
			if (event.type === "tool_execution_end") {
				const duplicate = event.result?.details?.superdevDuplicate === true;
				recordTerminalEvent({ type: "tool-end", tool: String(event.toolName ?? "unknown").slice(0, 80), error: Boolean(event.isError), ...(duplicate ? { duplicate: true } : {}) });
				if (event.toolName !== "superdev_submit_result") return;
				submissionEnds += 1;
				if (duplicate) {
					submissionDuplicates += 1;
				} else if (event.isError) {
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
		const finishArtifactStreams = async () => { await Promise.all([artifact.finishStdout(), artifact.finishStderr()]); };
		const timeout = setTimeout(() => { timedOut = true; stop(); }, policyTimeoutMs(policy));
		timeout.unref();
		child.stdout.setEncoding("utf8");
		child.stderr.setEncoding("utf8");
		child.stdout.on("data", (chunk: string) => {
			stdoutBytes += Buffer.byteLength(chunk);
			if (!artifact.writeStdout(chunk)) {
				child.stdout.pause();
				artifact.onStdoutDrain(() => child.stdout.resume());
			}
			pending += chunk;
			if (Buffer.byteLength(pending) > policy.maxArtifactBytes) {
				stop();
				return finish(async () => {
					await onClose?.(child);
					await finishArtifactStreams();
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
			await finishArtifactStreams();
			await writeDiagnostic("spawn-error", null, error);
			reject(new Error(`${String(error)}; diagnostics: ${artifact.diagnosticPath}`));
		}));
		child.on("close", (code) => finish(async () => {
			await onClose?.(child);
			if (pending) consume(pending);
			await finishArtifactStreams();
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
	return /\bsuperdev\s+workflow\s+(?:start|resume|cancel|record-evidence|evidence|scope-baseline|scope-checkpoint|sync|correction(?!-checkpoint)|transition|integrate|abandon)\b/.test(command)
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

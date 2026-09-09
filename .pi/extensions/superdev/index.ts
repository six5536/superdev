import { registerPhaseTool } from "./lib/phase-tool.ts";
import { registerIntakeTools } from "./lib/intake.ts";
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
import { parseLegacyRoleResult, roleResultSchemaFor, validateRoleResult, type Role, type RoleResult } from "./lib/review.ts";

import { killProcessTree, stopProcess, ensureParentService, runSuperdev, readOnly, defaultOutputPolicy, isolated, runGuardedBuildCommand, runPinnedSuperdev, requiresHumanAcceptance, isolatedRoleMayNotRun } from "./lib/process.ts";
export { isolated, isolatedTools, isolatedRoleMayNotRun, parseRoleResult, runPinnedSuperdev, runGuardedBuildCommand, requiresHumanAcceptance, buildCommandAllowed, buildCommands } from "./lib/process.ts";

const here = dirname(fileURLToPath(import.meta.url));
export default function superdev(pi: ExtensionAPI) {
	const childRoleValue = process.env.SUPERDEV_CHILD_ROLE;
	const childRole = childRoleValue && ["scope", "requirements-review", "build", "code-review", "accept"].includes(childRoleValue)
		? childRoleValue as Role
		: undefined;
	// Pi exposes these extension-owned resources as native /skill:* commands.
	if (!childRole) pi.on("resources_discover", () => ({ skillPaths: [join(here, "skills")] }));
	const reviewPaths = new Set<string>();
	const reviewOffsets = new Map<string, number>();
	let inventoryLoaded = false;
	let inventoryExpectedOffset = 1;
	let inventoryComplete = false;
	let childResultSubmitted = false;
	let terminalRepairQueued = false;
	let lastSubmissionError: string | undefined;
	if (childRole) {
		// Repair only the terminal envelope, once, in the same child and deadline.
		// A cancelled, failed, or terminating tool batch never triggers this turn.
		pi.on("agent_end", (event, ctx) => {
			const last = event.messages.findLast((message) => message.role === "assistant");
			if (childResultSubmitted || terminalRepairQueued || ctx.signal?.aborted || !last || last.stopReason !== "stop") return;
			// Schema validation failures bypass execute/tool_result; finalized messages retain them.
			const rejected = event.messages.findLast((message) => message.role === "toolResult" && message.toolName === "superdev_submit_result" && message.isError);
			if (rejected?.role === "toolResult") lastSubmissionError = rejected.content.filter((part) => part.type === "text").map((part) => part.text).join("\n").slice(0, 2_048);
			terminalRepairQueued = true;
			pi.setActiveTools(["superdev_submit_result"]);
			pi.sendMessage({
				customType: "superdev-terminal-repair",
				content: `The ${childRole} role stopped without an accepted typed result. ${lastSubmissionError ?? "No successful submission was observed."} This is the only terminal repair turn. Use the existing evidence; do not repeat work or invent review evidence. Invoke superdev_submit_result with a valid result. Put completed corrections in summary, not findings. If completion is unsupported, report an allowed non-success status.`,
				display: false,
			}, { deliverAs: "followUp", triggerTurn: true });
		});
		pi.registerTool({
			name: "superdev_submit_result",
			label: "Submit Superdev role result",
			description: "Submit one accepted typed result and terminate this isolated role. Rejected submissions do not count: correct the payload and submit again. Describe resolved corrections in summary, never findings.",
			parameters: roleResultSchemaFor(childRole),
			executionMode: "sequential",
			execute: async (_id, input) => {
				if (childResultSubmitted) {
					return {
						content: [{ type: "text", text: `The ${childRole} result was already submitted; this duplicate was ignored.` }],
						details: { superdevDuplicate: true },
						terminate: true,
					};
				}
				let result: RoleResult;
				try {
					result = validateRoleResult(childRole, input, {
						maxBytes: Number(process.env.SUPERDEV_MAX_REVIEW_STATE_BYTES ?? 262_144),
						maxFindings: Number(process.env.SUPERDEV_MAX_REVIEW_FINDINGS ?? 100),
					});
					if (childRole === "code-review" && result.status !== "blocked") {
						if (!inventoryLoaded || !inventoryComplete) throw new Error("code review did not completely inspect the changed-path inventory");
						const missing = [...reviewPaths].filter((path) => reviewOffsets.get(path) !== -1);
						if (missing.length) throw new Error(`code review omitted changed paths: ${missing.join(", ")}`);
					}
				} catch (error) {
					lastSubmissionError = String(error).slice(0, 2_048);
					throw new Error(`${lastSubmissionError}. No result was accepted. Correct the payload and submit again; rejected submissions do not count. Describe resolved corrections in summary, not findings.`, { cause: error });
				}
				childResultSubmitted = true;
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
		if (terminalRepairQueued && event.toolName !== "superdev_submit_result") {
			return { block: true, reason: "Terminal repair permits only superdev_submit_result; prior work must not run again", terminate: true };
		}
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
    pi.on("tool_call", (event) => {
        if (runtime.modifyingBusy && !["read", "sokf_search", "sokf_graph"].includes(event.toolName)) {
            return { block: true, reason: "A workflow child owns the checkout. Wait or cancel before modifying files." };
        }
    });
    pi.on("user_bash", () => {
        if (runtime.modifyingBusy) return { result: { output: "A workflow child owns the checkout; wait or cancel first.", exitCode: 1, cancelled: false, truncated: false } };
    });


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
		executableSha256?: string;
		defaultBranch?: string;
		defaultRevision?: string;
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
		status.executableSha256 = service.digest;
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
					const snapshot = await pi.exec("git", ["show", `${workflow.work_branch}:knowledge/plans/open/${workflow.plan}.md`], { cwd: ctx.cwd });
                    if (snapshot.code !== 0) throw new Error(snapshot.stderr || "Cannot read paused workflow snapshot");
                    const text = snapshot.stdout;
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
		reviewRuns, requiresHumanAcceptance, runtime,
	});
	registerPhaseTool({ pi, workflowStatus, questions, runSuperdev, authority, policyFrom, runtime, phaseContinuations });

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
            if (runtime.modifyingBusy) return ctx.ui.notify("Cancel the running child before abandoning", "warning");
            const status = await workflowStatus(ctx.cwd);
            if (!status.owner || status.owner.session_id !== ctx.sessionManager.getSessionId()) return ctx.ui.notify("Resume this workflow before abandoning it", "warning");
            if (!ctx.hasUI) return;
            const reason = args.trim() || (await ctx.ui.input("Reason for abandonment"))?.trim();
            if (!reason || !(await ctx.ui.confirm("Abandon workflow?", `${reason}\nPartial product work will not be merged.`))) return;
            await runSuperdev(["workflow", "abandon", "--session", status.owner.session_id, "--expected-revision", status.owner.last_plan_revision, "--phase", status.phase!, "--reason", reason], ctx.cwd, authority);
            questions.supersede("workflow abandoned by human");
            ctx.ui.setStatus("superdev-workflow", undefined);
            ctx.ui.notify("Workflow abandoned; partial product work remains on its branch", "info");
        },
    });
    registerIntakeTools({ pi });
}

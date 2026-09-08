import { createHash } from "node:crypto";
import { chmod, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

import superdev, { buildCommandAllowed, isolated, isolatedRoleMayNotRun, isolatedTools, parseRoleResult, requiresHumanAcceptance, runGuardedBuildCommand, runPinnedSuperdev } from "../../../.pi/extensions/superdev/index.ts";
import { IsolatedArtifact, boundedText } from "../../../.pi/extensions/superdev/lib/output.ts";
import { registerPhaseDrivers, type PhaseRuntime } from "../../../.pi/extensions/superdev/lib/phases.ts";
import { withProgress } from "../../../.pi/extensions/superdev/lib/progress.ts";
import { registerWorkflowQuestions } from "../../../.pi/extensions/superdev/lib/questions.ts";
import { findingFingerprint, validateRoleResult, type ReviewFinding } from "../../../.pi/extensions/superdev/lib/review.ts";

async function smoke() {
	const commands: string[] = [];
	const tools: string[] = [];
	const toolDefinitions = new Map<string, any>();
	const fake = {
		on() {
			// Registration is enough for this load smoke.
		},
		registerCommand(name: string) {
			commands.push(name);
		},
		registerTool(tool: { name: string }) {
			tools.push(tool.name);
			toolDefinitions.set(tool.name, tool);
		},
	};
	superdev(fake as never);
	for (const removed of ["scope", "build", "accept"]) {
		if (commands.includes(removed)) throw new Error(`legacy phase command ${removed} remains public`);
	}
	for (const command of [
		"superdev",
		"superdev-status",
		"superdev-resume",
		"superdev-cancel",
		"superdev-abandon",
		"file",
	]) {
		if (!commands.includes(command)) throw new Error(`missing command ${command}`);
	}
	if (!tools.includes("superdev_run_phase")) throw new Error("missing generic phase tool");
	if (!tools.includes("superdev_isolated_role")) throw new Error("missing isolated filing role tool");
	if (tools.includes("superdev_review_diff")) throw new Error("immutable review diff leaked into the main agent tool set");
	if (!tools.includes("superdev_workflow_control")) throw new Error("missing extension-private workflow control adapter");
	if (tools.includes("superdev_workflow_questions")) throw new Error("internal workflow question tool was exposed");
	const phaseSchema = JSON.stringify(toolDefinitions.get("superdev_run_phase")?.parameters);
	for (const action of ["run", "retry", "inspect", "record-answer", "revise-answer", "submit-answers", "approve", "cancel"]) {
		if (!phaseSchema.includes(`\"${action}\"`)) throw new Error(`generic phase tool omitted ${action}`);
	}
	if (phaseSchema.includes("expectedRevision") || phaseSchema.includes("session")) throw new Error("generic phase tool exposed internal ownership mechanics");
	const isolatedSchema = JSON.stringify(toolDefinitions.get("superdev_isolated_role")?.parameters);
	if (!isolatedSchema.includes("file") || ["scope", "build", "accept", "code-review", "requirements-review"].some((role) => isolatedSchema.includes(role))) {
		throw new Error("direct isolated tool exposed a workflow phase role");
	}
	const priorVerification = process.env.SUPERDEV_VERIFICATION_ACTIVE;
	process.env.SUPERDEV_VERIFICATION_ACTIVE = "1";
	try {
		const phaseTool = toolDefinitions.get("superdev_run_phase");
		const inspect = await phaseTool.execute("inspect", { phase: "scope", action: "inspect" }, undefined, undefined, { cwd: "/repo" });
		if (inspect.details?.status !== "idle" || inspect.details?.questions !== null) throw new Error("generic inspect did not return typed idle state");
		const cancel = await phaseTool.execute("cancel", { phase: "scope", action: "cancel" }, undefined, undefined, { cwd: "/repo" });
		if (cancel.details?.status !== "idle") throw new Error("generic cancel did not return typed idle state");
		try {
			await toolDefinitions.get("superdev_isolated_role").execute("direct", { role: "scope", task: "bypass" }, undefined, undefined, {});
			throw new Error("direct phase-role invocation was accepted");
		} catch (error) {
			if (!String(error).includes("internal")) throw error;
		}
	} finally {
		if (priorVerification === undefined) delete process.env.SUPERDEV_VERIFICATION_ACTIVE;
		else process.env.SUPERDEV_VERIFICATION_ACTIVE = priorVerification;
	}
	if (!requiresHumanAcceptance(true) || requiresHumanAcceptance(false)) throw new Error("configured acceptance policy changed");
	try {
		requiresHumanAcceptance(undefined);
		throw new Error("missing acceptance policy was treated as automatic");
	} catch (error) {
		if (!String(error).includes("omitted")) throw error;
	}
	for (const command of [
		"superdev workflow evidence",
		"superdev workflow scope-checkpoint",
		"superdev workflow correction",
		"superdev workflow sync",
		"superdev workflow transition",
		"superdev workflow integrate",
	]) {
		if (!isolatedRoleMayNotRun(command)) throw new Error(`isolated role may bypass ${command}`);
	}
	if (!isolatedRoleMayNotRun("git commit -am bypass")) throw new Error("isolated role may mutate Git history");
	if (isolatedTools("scope").split(",").includes("bash")) throw new Error("SCOPE child has direct shell access");
	if (isolatedTools("build").split(",").includes("bash")) throw new Error("BUILD child has direct shell access");
	if (isolatedTools("accept").split(",").some((tool) => ["bash", "edit", "write"].includes(tool))) throw new Error("ACCEPT assessor is not read-only");
	if (isolatedTools("code-review").split(",").includes("read")) throw new Error("final reviewer can bypass immutable diff bounds");
	if (!isolatedTools("code-review").split(",").includes("superdev_submit_result")) throw new Error("reviewer lacks typed terminal submission");
	if (!isolatedTools("build").split(",").includes("superdev_build_exec")) throw new Error("BUILD child cannot execute bounded evidence");
	if (buildCommandAllowed("sh", ["-c", "mutate Git"])) throw new Error("BUILD can escape through a shell");
	if (buildCommandAllowed("cargo", ["test"])) throw new Error("BUILD can bypass Rust-owned verification");
	if (buildCommandAllowed("git", ["diff", "--check"])) throw new Error("BUILD can invoke Git directly");
	if (!buildCommandAllowed("superdev", ["workflow", "block"])) throw new Error("BUILD cannot publish a block checkpoint");
	if (buildCommandAllowed("superdev", ["file"])) throw new Error("BUILD can escape through an unrelated service command");
	if (buildCommandAllowed("superdev", ["workflow", "transition"])) throw new Error("BUILD can transition workflow state");
	for (const command of [
		"superdev workflow block",
		"superdev workflow attempt",
		"superdev workflow correction-checkpoint",
		"superdev workflow status",
	]) {
		if (isolatedRoleMayNotRun(command)) throw new Error(`isolated role cannot perform ${command}`);
	}
	const checklist = ["requirements", "contracts", "architecture", "tests", "documentation", "scope", "consistency"]
		.map((area) => ({ area, complete: true, evidence: `${area} checked` }));
	const clean = parseRoleResult(
		"code-review",
		`analysis\nSUPERDEV_RESULT ${JSON.stringify({ status: "clean", summary: "No actionable findings.", checklist })}`,
	);
	if (clean.status !== "clean") throw new Error("structured clean review was not parsed");
	try {
		validateRoleResult("code-review", { status: "findings", summary: "too many", findings: Array(2).fill({ id: "duplicate" }), checklist }, { maxBytes: 10_000, maxFindings: 1 });
		throw new Error("review finding limit was ignored");
	} catch (error) {
		if (!String(error).includes("more than 1")) throw error;
	}
	const finding: ReviewFinding = { id: "f1", classification: "substantive", summary: "Choose policy", evidence: "Policy is absent", impact: "Behavior is unsettled", question: "Which policy?", recommendation: "Use the safe policy" };
	if (findingFingerprint(finding) === findingFingerprint({ ...finding, impact: "Different behavior" })) throw new Error("semantic fingerprint ignored impact");
	for (const [label, result] of [
		["contradictory status", { status: "complete", summary: "contradiction", findings: [{ ...finding, classification: "correctable-within-scope" }], checklist }],
		["cyclic dependencies", { status: "findings", summary: "cycle", findings: [
			{ ...finding, id: "a", classification: "correctable-within-scope", dependsOn: ["b"] },
			{ ...finding, id: "b", classification: "correctable-within-scope", dependsOn: ["a"] },
		], checklist }],
		["duplicate checklist", { status: "clean", summary: "duplicate", checklist: [...checklist, checklist[0]] }],
	] as const) {
		try {
			validateRoleResult("code-review", result, { maxBytes: 100_000, maxFindings: 100 });
			throw new Error(`${label} was accepted`);
		} catch (error) {
			if (String(error).includes("was accepted")) throw error;
		}
	}
	const bounded = boundedText("one\ntwo\nthree", { maxContextBytes: 20, maxContextLines: 2, maxArtifactBytes: 100, maxArtifacts: 2, retentionHours: 1 });
	if (!bounded.includes("Showing 2 of 3 lines")) throw new Error("model-visible output was not line bounded");
	const artifact = await IsolatedArtifact.create("smoke", "review", 16);
	try {
		artifact.writeStderr("0123456789abcdefghijklmnop");
		await artifact.finishStderr();
		if ((await stat(artifact.directory)).mode & 0o077) throw new Error("artifact directory is not owner-only");
		try {
			await artifact.writeResult({ result: "this authoritative result is too large" });
			throw new Error("oversized authoritative result was accepted");
		} catch (error) {
			if (!String(error).includes("isolated-output-overflow")) throw error;
		}
	} finally {
		await rm(artifact.directory, { recursive: true, force: true });
	}
	const fakePiDirectory = await mkdtemp(join(tmpdir(), "superdev-fake-pi-"));
	const fakePi = join(fakePiDirectory, "pi");
	const originalPath = process.env.PATH;
	let diagnosticDirectory: string | undefined;
	try {
		await writeFile(fakePi, `#!/usr/bin/env node\nconsole.log(JSON.stringify({type:"message_end",message:{role:"assistant",stopReason:"stop"}}));\nconsole.log(JSON.stringify({type:"tool_execution_start",toolName:"superdev_submit_result",args:{status:"complete"}}));\nconsole.log(JSON.stringify({type:"tool_execution_end",toolName:"superdev_submit_result",isError:true,result:{content:[{type:"text",text:"summary is required"}]}}));\n`, { mode: 0o700 });
		await chmod(fakePi, 0o700);
		process.env.PATH = `${fakePiDirectory}:${originalPath ?? ""}`;
		try {
			await isolated("scope", "exercise diagnostic capture", process.cwd());
			throw new Error("invalid terminal submission was accepted");
		} catch (error) {
			const match = String(error).match(/diagnostics: (\/[^\s]+)/);
			if (!match) throw error;
			diagnosticDirectory = dirname(match[1]);
			const diagnostic = JSON.parse(await readFile(match[1], "utf8"));
			if (diagnostic.outcome !== "terminal-protocol-failure" || diagnostic.submission.starts !== 1 || diagnostic.submission.ends !== 1 || diagnostic.submission.errors !== 1 || diagnostic.assistantEnds !== 1) {
				throw new Error("isolated terminal diagnostics omitted lifecycle evidence");
			}
		}
	} finally {
		process.env.PATH = originalPath;
		await rm(fakePiDirectory, { recursive: true, force: true });
		if (diagnosticDirectory) await rm(diagnosticDirectory, { recursive: true, force: true });
	}
	const questionTools = new Map<string, any>();
	const entries: any[] = [];
	let active = ["read", "edit"];
	let paused = false;
	const questionPi = {
		appendEntry(type: string, data: unknown) { entries.push({ type: "custom", customType: type, data: structuredClone(data) }); },
		getActiveTools() { return active; },
		setActiveTools(next: string[]) { active = next; },
		registerTool(tool: any) { questionTools.set(tool.name, tool); },
	};
	const controller = registerWorkflowQuestions(questionPi, {
		maxBytes: () => 16_384,
		maxFindings: () => 100,
		onPause: async () => { paused = true; },
		onResume: async () => { paused = false; },
		onSubmit: async () => {},
	});
	controller.begin({ version: 1, workflow: "plan-smoke", candidate: "abcdef1", status: "active", findings: [finding], answers: {} });
	if (active.includes("edit") || !active.includes("superdev_workflow_questions")) throw new Error("question discussion tools are not read-only");
	const questionTool = questionTools.get("superdev_workflow_questions");
	await questionTool.execute("q1", { action: "propose-answer", findingIds: ["f1"], proposedAnswer: "Use the safe policy" }, new AbortController().signal, undefined, { hasUI: true, ui: { confirm: async () => true } });
	await questionTool.execute("q2", { action: "pause" }, new AbortController().signal, undefined, {});
	if (!paused || controller.current()?.status !== "paused") throw new Error("question pause did not persist and release ownership");
	await questionTool.execute("q3", { action: "resume" }, new AbortController().signal, undefined, {});
	if (paused || controller.current()?.status !== "active") throw new Error("question resume did not reacquire ownership");
	try {
		parseRoleResult("code-review", "CLEAN");
		throw new Error("unstructured review was accepted");
	} catch (error) {
		if (String(error).includes("unstructured review was accepted")) throw error;
	}
	const checkpoint = await runGuardedBuildCommand(async (command, args, cwd) => {
		if (command !== "superdev" || args.join(" ") !== "workflow block" || cwd !== "/repo") throw new Error("BUILD service command changed");
		return { code: 0, stdout: "checkpointed\n", stderr: "" };
	}, "superdev", ["workflow", "block"], "/repo");
	if (checkpoint.stdout !== "checkpointed\n") throw new Error("BUILD checkpoint result was lost");
	try {
		await runGuardedBuildCommand(async () => ({ code: 0, stdout: "", stderr: "" }), "cargo", ["test"], "/repo");
		throw new Error("direct BUILD executable was accepted");
	} catch (error) {
		if (!String(error).includes("not permitted")) throw error;
	}
	if (process.platform === "linux") {
		const digest = createHash("sha256").update(await readFile(process.execPath)).digest("hex");
		const pinned = await runPinnedSuperdev(process.execPath, digest, ["-e", "process.stdout.write('pinned')"], process.cwd());
		if (pinned.code !== 0 || pinned.stdout !== "pinned") throw new Error("descriptor-pinned executable did not run");
		try {
			await runPinnedSuperdev(process.execPath, "0".repeat(64), ["--version"], process.cwd());
			throw new Error("pinned executable accepted a stale digest");
		} catch (error) {
			if (!String(error).includes("changed after the parent pinned it")) throw error;
		}
	}

	// Execute the registered phase handlers against a deterministic fake Pi and
	// workflow service. This catches orchestration gaps that registration-only
	// smoke coverage cannot observe.
	const phaseCommands = new Map<string, any>();
	const serviceCalls: string[][] = [];
	const progress: string[] = [];
	const selections: string[] = ["Approve only"];
	let phase: "scope" | "build" | "accept" = "scope";
	let revision = "revision-1";
	let owned = true;
	let humanAcceptanceRequired = false;
	let candidateEvidence = false;
	let finalCorrections = 0;
	let pendingPhaseQuestions: any;
	const baseRevision = "1".repeat(40);
	const candidateRevision = "2".repeat(40);
	const runtime: PhaseRuntime = { cancelling: false, modifyingBusy: false };
	let roleFailure: string | undefined;
	let blockedRole: string | undefined;
	let cancellableRole: string | undefined;
	let cancelNextProgress = false;
	const owner = () => owned ? {
		session_id: "session-smoke",
		last_plan_revision: revision,
		scope_base_revision: baseRevision,
		...(candidateEvidence ? { candidate_revision: candidateRevision, verified_default_revision: baseRevision } : {}),
		identity: { issue: "issue-smoke", plan: "plan-smoke", work_branch: "work/smoke", default_branch: "main" },
	} : undefined;
	const status = async () => ({
		owner: owner(), phase, canonicalPlanRevision: revision,
		humanAcceptanceRequired, executable: "/service",
		maxFinalCorrectionCycles: 3, maxScopeReviewCycles: 3,
		buildState: { currentBlock: 1, attempts: 0, finalCorrections, blocker: finalCorrections >= 3 ? "final correction limit exhausted: remaining findings" : "none" },
	});
	const phasePi = {
		registerCommand(name: string, command: any) { phaseCommands.set(name, command); },
		async exec(_command: string, args: string[]) {
			if (args[0] === "status") return { code: 0, stdout: "", stderr: "" };
			const target = args.at(-1);
			return { code: 0, stdout: `${target === "main" || target === baseRevision ? baseRevision : candidateRevision}\n`, stderr: "" };
		},
	};
	const runService = async (args: string[]) => {
		serviceCalls.push(args);
		if (args.includes("scope-checkpoint")) revision = "revision-2";
		if (args.includes("scope-review")) revision = "revision-3";
		if (args.includes("approve-scope")) { phase = "build"; revision = "revision-4"; }
		if (args.includes("final")) { phase = "accept"; revision = "revision-5"; candidateEvidence = true; }
		if (args.includes("integrate")) owned = false;
		if (args.includes("cancel")) owned = false;
		return { result: args.includes("transition") ? { state: { last_plan_revision: revision } } : { last_plan_revision: revision } };
	};
	const ui = {
		setStatus(_key: string, value?: string) { if (value) progress.push(value); },
		notify() {},
		async select() { return selections.shift(); },
		async confirm() { return true; },
		async input() { return "confirmed"; },
		async custom(factory: any) {
			return await new Promise((resolve) => {
				const component = factory({ requestRender() {} }, { fg(_style: string, text: string) { return text; } }, {}, resolve);
				component.render(80);
				if (cancelNextProgress) {
					cancelNextProgress = false;
					component.handleInput("\u001b");
				}
			});
		},
	};
	const phaseCtx = { cwd: "/repo", model: undefined, hasUI: true, sessionManager: { getSessionId: () => "session-smoke" }, ui };
	const phaseDrivers = registerPhaseDrivers({
		pi: phasePi,
		runSuperdev: runService,
		workflowStatus: status,
		authority: "authority",
		policyFrom: () => ({ timeoutSeconds: 1, maxContextBytes: 8192, maxContextLines: 200, maxReviewStateBytes: 262144, maxReviewFindings: 100, maxArtifactBytes: 10485760, maxArtifacts: 20, retentionHours: 24 }),
		questions: { begin(state: any) { pendingPhaseQuestions = state; }, current() { return pendingPhaseQuestions; } },
		isolated: async (role: string, _task: string, _cwd: string, _model: unknown, signal: AbortSignal) => {
			if (roleFailure === role) throw new Error(`${role} timed out after 1s; diagnostics: /tmp/${role}-diagnostic.json`);
			if (cancellableRole === role) return await new Promise((_resolve, reject) => {
				const cancelled = () => reject(new Error(`${role} role was cancelled; diagnostics: /tmp/${role}-cancelled.json`));
				if (signal.aborted) cancelled();
				else signal.addEventListener("abort", cancelled, { once: true });
			});
			if (blockedRole === role) return { status: "blocked", summary: `${role} blocked`, artifactPath: `/tmp/${role}-result.json` };
			return role === "scope" || role === "build"
				? { status: "complete", summary: `${role} complete` }
				: { status: role === "accept" ? "complete" : "clean", summary: `${role} clean`, checklist };
		},
		childStarted: () => () => {}, childFinished: async () => {}, reviewRuns: new Map(),
		parentServiceDigest: "digest", requiresHumanAcceptance, runtime,
	});
	await phaseDrivers.runScopePhase("", phaseCtx);
	if (runtime.lastOutcome?.status !== "ready-for-approval" || phase !== "scope") throw new Error("SCOPE did not return a typed approval gate to its skill");
	phase = "build"; revision = "revision-4"; runtime.lastOutcome = undefined;
	await phaseDrivers.runBuildPhase("", phaseCtx);
	if (owned || !serviceCalls.some((args) => args.includes("integrate"))) throw new Error("BUILD did not continue through automatic ACCEPT integration");
	if (!progress.some((value) => value.includes("SCOPE")) || !progress.some((value) => value.includes("BUILD")) || !progress.some((value) => value.includes("ACCEPT"))) {
		throw new Error("phase handlers did not publish immediate progress");
	}
	owned = true; phase = "accept"; revision = "revision-approval"; candidateEvidence = true; humanAcceptanceRequired = true; runtime.lastOutcome = undefined;
	await phaseDrivers.runAcceptPhase("", phaseCtx);
	if (!owned || runtime.lastOutcome?.status !== "ready-for-approval" || runtime.lastOutcome?.expectedRevision !== revision) {
		throw new Error("ACCEPT did not return a revision-bound typed approval gate");
	}

	owned = true; phase = "build"; revision = "revision-timeout"; candidateEvidence = false; humanAcceptanceRequired = false;
	roleFailure = "build"; runtime.lastOutcome = undefined;
	const callsBeforeTimeout = serviceCalls.length;
	await phaseDrivers.runBuildPhase("", phaseCtx);
	if (owned || runtime.lastOutcome?.status !== "failed" || runtime.lastOutcome?.failedStage !== "build implementation" || runtime.lastOutcome?.diagnosticPath !== "/tmp/build-diagnostic.json" || runtime.lastOutcome?.partialWorkPreserved !== true) throw new Error("timed-out BUILD did not release ownership with stage-specific typed recovery");
	if (serviceCalls.slice(callsBeforeTimeout).some((args) => args.includes("resume"))) throw new Error("timed-out BUILD retried without an explicit retry operation");

	owned = true; phase = "build"; revision = "revision-review-blocked"; roleFailure = undefined; blockedRole = "code-review"; finalCorrections = 0; runtime.lastOutcome = undefined;
	const callsBeforeBlockedReview = serviceCalls.length;
	await phaseDrivers.runBuildPhase("", phaseCtx);
	if (runtime.lastOutcome?.status !== "blocked" || runtime.lastOutcome?.artifactPath !== "/tmp/code-review-result.json") throw new Error("blocked code review did not return typed recovery with its exact artifact path");
	if (serviceCalls.slice(callsBeforeBlockedReview).some((args) => args.includes("correction"))) throw new Error("blocked code review consumed a semantic correction cycle");

	owned = true; phase = "accept"; revision = "revision-cancelled"; candidateEvidence = true; blockedRole = undefined; cancellableRole = "accept"; cancelNextProgress = true; runtime.lastOutcome = undefined;
	await phaseDrivers.runAcceptPhase("", phaseCtx);
	if (owned || runtime.lastOutcome?.status !== "paused" || runtime.lastOutcome?.failedStage !== "accept assessment" || runtime.lastOutcome?.diagnosticPath !== "/tmp/accept-cancelled.json") throw new Error("Esc did not pause ACCEPT with typed preserved-work recovery");

	owned = true; phase = "build"; revision = "revision-exhausted"; candidateEvidence = false; cancellableRole = undefined; finalCorrections = 3;
	await phaseDrivers.runBuildPhase("", phaseCtx);
	if (pendingPhaseQuestions?.originPhase !== "build" || pendingPhaseQuestions?.candidate !== revision) {
		throw new Error("exhausted BUILD did not preserve a revision-bound human decision queue");
	}

	let progressAborted = false;
	let progressCleared = false;
	try {
		await withProgress({ hasUI: true, ui: {
			setStatus(_key: string, value?: string) { if (!value) progressCleared = true; },
			async custom(factory: any) {
				return await new Promise((resolve) => {
					const component = factory({ requestRender() {} }, { fg(_style: string, text: string) { return text; } }, {}, resolve);
					component.handleInput("\u001b");
				});
			},
		} }, { key: "smoke-progress", title: "BUILD plan-smoke", stage: "implementation" }, async (signal) => {
			await new Promise<void>((_resolve, reject) => signal.addEventListener("abort", () => { progressAborted = true; reject(new Error("cancelled")); }, { once: true }));
		});
		throw new Error("Esc cancellation unexpectedly completed");
	} catch (error) {
		if (!progressAborted || !String(error).includes("cancelled")) throw error;
	}
	if (!progressCleared) throw new Error("Esc cancellation left stale progress status");
}

await smoke();

export default function loadedSmoke() {
	// Top-level await completes the deterministic smoke before Pi registers this fixture.
}

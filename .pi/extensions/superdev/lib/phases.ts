import { randomBytes } from "node:crypto";
import { isAbsolute } from "node:path";
import { boundedText } from "./output.ts";
import { withProgress } from "./progress.ts";
import { findingFingerprint, type ReviewFinding } from "./review.ts";
import type { QuestionState } from "./questions.ts";

export type PhaseRuntime = {
	cancelling: boolean;
	modifyingBusy: boolean;
	modifyingChild?: unknown;
	modifyingCommandAbort?: AbortController;
	currentStage?: string;
	lastOutcome?: Record<string, unknown>;
};

export function registerPhaseDrivers(deps: any) {
	const { pi, runSuperdev, workflowStatus, authority, policyFrom, questions, isolated, childStarted, childFinished, reviewRuns, parentServiceDigest, requiresHumanAcceptance, runtime } = deps;
	const hasPendingQuestions = () => ["active", "paused"].includes(questions.current?.()?.status ?? "");
	const pauseWithOutcome = async (ctx: any, phase: string, error: unknown, cancelled: boolean): Promise<boolean> => {
		let status: any;
		try { status = await workflowStatus(ctx.cwd); }
		catch { status = {}; }
		const workflow = status.owner?.identity;
		if (status.owner) {
			try { await runSuperdev(["workflow", "cancel", "--session", status.owner.session_id], ctx.cwd, authority); }
			catch { /* Preserve the original diagnostic as the actionable error. */ }
		}
		ctx.ui.setStatus("superdev-workflow", undefined);
		const rawDiagnostic = String(error);
		const diagnostic = boundedText(rawDiagnostic, policyFrom(status));
		const diagnosticPath = rawDiagnostic.match(/diagnostics:\s*([^;\n]+)/)?.[1]?.trim();
		runtime.lastOutcome = {
			status: cancelled ? "paused" : "failed",
			phase: phase.toLowerCase(),
			failedStage: runtime.currentStage ?? phase.toLowerCase(),
			diagnostic,
			...(diagnosticPath ? { diagnosticPath } : {}),
			partialWorkPreserved: true,
			workflow,
			recoveryOperations: cancelled ? ["retry"] : ["retry", "cancel"],
			recommendation: cancelled
				? "Resume with an explicit retry when ready."
				: "Inspect the diagnostic, discuss it with the active phase skill, then retry only when the cause is understood.",
		};
		ctx.ui.notify(cancelled ? `${phase} cancelled; partial work was preserved.` : `${phase} paused after a failure. The invoking skill has the diagnostic and recovery operations.`, cancelled ? "warning" : "error");
		return false;
	};
	const phaseSignal = (progressSignal: AbortSignal, commandAbort: AbortController) => {
		if (progressSignal.aborted) commandAbort.abort();
		else progressSignal.addEventListener("abort", () => commandAbort.abort(), { once: true });
		return AbortSignal.any([progressSignal, commandAbort.signal]);
	};
	const publishRoleOutcome = (phase: "scope" | "build" | "accept", result: any) => {
		runtime.lastOutcome = {
			status: result.status,
			phase,
			summary: result.summary,
			artifactPath: result.artifactPath,
			partialWorkPreserved: true,
			recoveryOperations: result.status === "blocked" ? ["retry", "cancel"] : ["cancel"],
		};
	};
	const pauseForBuildExhaustion = async (ctx: any, summary: string, findings: ReviewFinding[] = []) => {
		const status = await workflowStatus(ctx.cwd);
		const owner = status.owner;
		if (!owner || status.phase !== "build") throw new Error("BUILD correction exhaustion lost workflow ownership");
		const decision: ReviewFinding = {
			id: "build-correction-limit-exhausted",
			classification: "substantive",
			summary,
			evidence: `The configured final-correction budget is exhausted. ${status.buildState?.blocker ?? ""}`.trim(),
			impact: "BUILD cannot spend another correction cycle without fresh SCOPE approval.",
			question: "Should the complete finding set return to SCOPE for review and a fresh approval?",
			choices: [{ label: "Return to SCOPE: review findings and renew the correction budget" }],
			recommendation: "Return to SCOPE when the findings still warrant implementation work; otherwise pause for discussion.",
		};
		questions.begin({ version: 1, workflow: owner.identity.plan, candidate: owner.last_plan_revision, originPhase: "build", status: "active", findings: [decision], mechanicalFindings: findings, answers: {} });
		ctx.ui.notify(`Final correction limit exhausted: ${summary}. Findings were preserved for discussion, return to SCOPE, or pause.`, "error");
	};
	let startBuildPhase: ((args: string, ctx: any) => Promise<void>) | undefined;
	const runScopePhase = async (args: string, ctx: any, submitted?: QuestionState) => {
		if (!submitted && hasPendingQuestions()) return ctx.ui.notify("Resolve or pause the active workflow questions before starting a phase role", "error");
		if (runtime.cancelling || runtime.modifyingBusy || runtime.modifyingChild) return ctx.ui.notify("A modifying workflow role is already active", "error");
		runtime.modifyingBusy = true;
		runtime.currentStage = "scope initialization";
		const commandAbort = new AbortController();
		runtime.modifyingCommandAbort = commandAbort;
		let launchBuild = false;
		let retryScope = false;
		try {
			let correction = submitted;
			let cycle = submitted?.cycle ?? 0;
			let firstBase: string | undefined;
			while (true) {
				const initial = await workflowStatus(ctx.cwd);
				if (commandAbort.signal.aborted) throw new Error("SCOPE was cancelled during initialization");
				if (!initial.owner || initial.phase !== "scope") throw new Error("An owned SCOPE workflow is required");
				if (correction && initial.owner.last_plan_revision !== correction.candidate) throw new Error("SCOPE candidate changed; saved answers were superseded");
				if (!firstBase) {
					const resolved = await pi.exec("git", ["rev-parse", "--verify", initial.owner.scope_base_revision ?? initial.owner.identity.work_branch], { cwd: ctx.cwd });
					if (resolved.code !== 0) throw new Error("Could not resolve the pre-SCOPE revision");
					firstBase = resolved.stdout.trim();
				}
				const task = correction
					? `Apply this complete SCOPE correction batch for ${initial.owner.identity.plan}. Mechanical findings: ${JSON.stringify(correction.mechanicalFindings ?? [])}. Confirmed human answers: ${JSON.stringify(correction.answers)}.`
					: args || `Complete ${initial.owner.identity.plan} from canonical state.`;
				runtime.currentStage = correction ? "scope batched correction" : "scope authoring";
				const scoped = await withProgress(ctx, { key: "superdev-workflow", title: `SCOPE ${initial.owner.identity.plan}`, stage: correction ? "batched correction" : "scope authoring" },
					(signal, update) => isolated("scope", task, ctx.cwd, ctx.model, phaseSignal(signal, commandAbort),
						childStarted(ctx, initial, "scope", true),
						(child) => childFinished(ctx, child),
						undefined, undefined, undefined, undefined, policyFrom(initial), initial.owner!.session_id,
						(activity) => update({ stage: correction ? "batched correction" : "scope authoring", activity })));
				if (scoped.status !== "complete") {
					publishRoleOutcome("scope", scoped);
					return ctx.ui.notify(scoped.summary, "warning");
				}
				runtime.currentStage = "scope checkpoint";
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
				runtime.currentStage = "requirements review";
				const reviewed = await withProgress(ctx, { key: "superdev-workflow", title: `SCOPE ${owner.identity.plan}`, stage: "requirements review" },
					(signal, update) => isolated("requirements-review", `Review the complete immutable SCOPE candidate ${candidate} against baseline ${firstBase}.`, ctx.cwd, ctx.model, phaseSignal(signal, commandAbort),
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
							if (cycle >= (status.maxScopeReviewCycles ?? 3)) {
								questions.begin({ version: 1, workflow: owner.identity.plan, candidate: status.canonicalPlanRevision, status: "active", findings: substantive, mechanicalFindings: mechanical, cycle, answers: reused });
								ctx.ui.notify(`SCOPE correction limit exhausted: ${reviewed.summary}. Review the preserved answers, then explicitly submit them to authorize one retry cycle or pause for discussion.`, "error");
								return;
							}
							correction = { version: 1, workflow: owner.identity.plan, candidate: status.canonicalPlanRevision, status: "submitted", findings: substantive, mechanicalFindings: mechanical, cycle, answers: reused };
							continue;
						}
						questions.begin({ version: 1, workflow: owner.identity.plan, candidate: status.canonicalPlanRevision, status: "active", findings: substantive, mechanicalFindings: mechanical, cycle, answers: reused });
						ctx.ui.notify(`Requirements review found ${substantive.length} substantive and ${mechanical.length} mechanical findings. Discuss or answer them one at a time.`, "warning");
						return;
					}
					if (cycle >= (status.maxScopeReviewCycles ?? 3)) {
						const exhaustion: ReviewFinding = {
							id: "scope-correction-limit-exhausted",
							classification: "substantive",
							summary: reviewed.summary,
							evidence: `${mechanical.length} actionable findings remain after ${cycle} complete correction and review cycles.`,
							impact: "No further model correction runs automatically; the findings and retry decision must remain available for human discussion.",
							question: "Should SCOPE run one explicitly authorized retry cycle, revise intent, or remain paused?",
							choices: [{ label: "Retry one cycle" }, { label: "Revise scope intent" }],
							recommendation: "Retry one cycle only when the remaining fixes are still deterministic under approved intent.",
						};
						questions.begin({ version: 1, workflow: owner.identity.plan, candidate: status.canonicalPlanRevision, status: "active", findings: [exhaustion], mechanicalFindings: mechanical, cycle, answers: {} });
						ctx.ui.notify(`SCOPE correction limit exhausted: ${reviewed.summary}. Findings were preserved for discussion, explicit retry, or pause.`, "error");
						return;
					}
					correction = { version: 1, workflow: owner.identity.plan, candidate: status.canonicalPlanRevision, status: "submitted", findings: [], mechanicalFindings: mechanical, cycle, answers: {} };
					continue;
				}
				if (reviewed.status !== "clean") {
					publishRoleOutcome("scope", reviewed);
					return ctx.ui.notify(reviewed.summary, "error");
				}
				runtime.currentStage = "scope review evidence";
				const reviewRun = randomBytes(24).toString("hex");
				reviewRuns.set(reviewRun, { role: "requirements-review", result: reviewed, base: firstBase, candidate });
				const evidence = await runSuperdev(["workflow", "evidence", "--session", owner.session_id, "--expected-revision", owner.last_plan_revision, "--revision", status.canonicalPlanRevision, "--kind", "scope-review", "--review-session", reviewRun, "--candidate", candidate], ctx.cwd, authority) as { result?: { last_plan_revision?: string } };
				const revision = evidence.result?.last_plan_revision;
				if (!revision) throw new Error("scope evidence response omitted the new plan revision");
				reviewRuns.delete(reviewRun);
				runtime.lastOutcome = {
					status: "ready-for-approval",
					phase: "scope",
					workflow: owner.identity,
					candidate,
					expectedRevision: revision,
					recommendation: "Review the clean SCOPE summary, then use the SCOPE skill to approve or request a revision.",
				};
				ctx.ui.notify("Requirements review is clean; the SCOPE skill now owns discussion and approval", "info");
				return;
			}
		} catch (error) {
			retryScope = await pauseWithOutcome(ctx, "SCOPE", error, commandAbort.signal.aborted);
		} finally {
			if (runtime.modifyingCommandAbort === commandAbort) runtime.modifyingCommandAbort = undefined;
			runtime.modifyingBusy = false;
		}
		if (retryScope) return runScopePhase(args, ctx, submitted);
		if (launchBuild) {
			if (!startBuildPhase) throw new Error("BUILD driver is unavailable");
			await startBuildPhase("", ctx);
		}
	};
	const continueScopeFromAnswers = (state, ctx) => runScopePhase("", ctx, state);
	let runAcceptPhase: ((args: string, ctx: any) => Promise<void>) | undefined;
	const runBuildPhase = async (args: string, ctx: any) => {
			if (hasPendingQuestions()) return ctx.ui.notify("Resolve or pause the active workflow questions before starting a phase role", "error");
			if (runtime.cancelling || runtime.modifyingBusy || runtime.modifyingChild) return ctx.ui.notify("A modifying workflow role is already active", "error");
			runtime.modifyingBusy = true;
			runtime.currentStage = "build initialization";
			const commandAbort = new AbortController();
			runtime.modifyingCommandAbort = commandAbort;
			let retryBuild = false;
			try {
			let instruction = args;
			while (true) {
				const status = await workflowStatus(ctx.cwd);
				if (commandAbort.signal.aborted) throw new Error("BUILD was cancelled during initialization");
				const owner = status.owner;
				if (!owner || status.phase !== "build") throw new Error("An owned BUILD workflow is required");
				if (status.buildState && status.maxFinalCorrectionCycles !== undefined
					&& status.buildState.finalCorrections >= status.maxFinalCorrectionCycles) {
					await pauseForBuildExhaustion(ctx, status.buildState.blocker || "Final correction budget exhausted");
					return;
				}
				if (!status.executable || !isAbsolute(status.executable)) throw new Error("Rust status omitted its trusted executable path");
				runtime.currentStage = instruction ? "build batched correction" : "build implementation";
				const built = await withProgress(ctx, { key: "superdev-workflow", title: `BUILD ${owner.identity.plan}`, stage: instruction ? "batched correction" : "implementation" },
					(signal, update) => isolated("build", instruction || `Complete ${owner.identity.plan} from canonical state.`, ctx.cwd, ctx.model, phaseSignal(signal, commandAbort),
						childStarted(ctx, status, "build", true),
						(child) => childFinished(ctx, child), undefined, undefined, status.executable, parentServiceDigest,
						policyFrom(status), owner.session_id,
						(activity) => update({ stage: instruction ? "batched correction" : "implementation", activity })));
			if (commandAbort.signal.aborted) throw new Error("BUILD was cancelled before publication");
			if (built.status === "rescope") {
				runtime.currentStage = "build return to scope";
				const latest = await workflowStatus(ctx.cwd);
				if (!latest.owner || latest.phase !== "build") throw new Error("BUILD ownership changed before re-scope");
				await runSuperdev([
					"workflow", "transition", "--session", latest.owner.session_id,
					"--expected-revision", latest.owner.last_plan_revision, "--phase", "build",
					"--transition", "return-to-scope", "--feedback", built.summary,
				], ctx.cwd, authority);
				ctx.ui.setStatus("superdev-workflow", `SCOPE: ${latest.owner.identity.plan}`);
				runtime.lastOutcome = { status: "routed", phase: "scope", summary: built.summary, workflow: latest.owner.identity };
				return ctx.ui.notify("BUILD discovery was preserved on the primary issue; the same plan returned to SCOPE", "warning");
			}
			if (built.status !== "complete") {
				publishRoleOutcome("build", built);
				return ctx.ui.notify(built.summary, "error");
			}
			runtime.currentStage = "build synchronization";
			const ready = await workflowStatus(ctx.cwd);
			const currentOwner = ready.owner;
			if (!currentOwner || ready.phase !== "build") throw new Error("BUILD ownership changed before synchronization");
			const baseResult = await pi.exec("git", ["rev-parse", "--verify", currentOwner.identity.default_branch], { cwd: ctx.cwd });
			const candidateResult = await pi.exec("git", ["rev-parse", "--verify", currentOwner.identity.work_branch], { cwd: ctx.cwd });
			if (baseResult.code !== 0 || candidateResult.code !== 0) throw new Error("Could not resolve synchronization revisions");
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
			if (synchronizedBase.code !== 0 || synchronizedCandidate.code !== 0) throw new Error("Could not resolve immutable review revisions");
			const base = synchronizedBase.stdout.trim();
			const candidate = synchronizedCandidate.stdout.trim();
			runtime.currentStage = "build verification";
			const verification = await runSuperdev([
				"workflow", "evidence", "--session", synchronizedOwner.session_id,
				"--expected-revision", synchronizedOwner.last_plan_revision, "--kind", "verification",
				"--candidate", candidate,
			], ctx.cwd, authority) as { result?: { last_plan_revision?: string } };
			const verifiedRevision = verification.result?.last_plan_revision;
			if (!verifiedRevision) throw new Error("verification response omitted the new plan revision");
			runtime.currentStage = "code review";
			const reviewed = await withProgress(ctx, { key: "superdev-workflow", title: `BUILD ${synchronizedOwner.identity.plan}`, stage: "code review" },
				(signal, update) => isolated("code-review", `Review immutable diff ${base}..${candidate} and return the required structured result.`, ctx.cwd, ctx.model, phaseSignal(signal, commandAbort),
					childStarted(ctx, synchronized, "code-review"), (child) => childFinished(ctx, child), base, candidate,
					undefined, undefined, policyFrom(synchronized), synchronizedOwner.session_id,
					(activity) => update({ stage: "code review", activity })));
			runtime.currentStage = "build review routing";
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
			if (reviewed.status !== "clean" && reviewed.status !== "findings") {
				publishRoleOutcome("build", reviewed);
				return ctx.ui.notify(reviewed.summary, "error");
			}
			if (reviewed.status === "findings") {
				const correction = await runSuperdev([
					"workflow", "correction", "--session", synchronizedOwner.session_id,
					"--expected-revision", verifiedRevision, "--candidate", candidate,
					"--review-session", reviewRun,
					"--summary", reviewed.findings?.map((finding) => `${finding.id}: ${finding.summary}`).join("\n") ?? reviewed.summary,
				], ctx.cwd, authority) as { result?: { stalled?: boolean } };
				const stalled = correction.result?.stalled === true;
				if (stalled) {
					await pauseForBuildExhaustion(ctx, reviewed.summary, reviewed.findings ?? []);
					return;
				}
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
				retryBuild = await pauseWithOutcome(ctx, "BUILD", error, commandAbort.signal.aborted);
			} finally {
				if (runtime.modifyingCommandAbort === commandAbort) runtime.modifyingCommandAbort = undefined;
				runtime.modifyingBusy = false;
			}
			if (retryBuild) return runBuildPhase(args, ctx);
	};
	startBuildPhase = runBuildPhase;
	const continueBuildDecision = async (state: QuestionState, ctx: any) => {
		const status = await workflowStatus(ctx.cwd);
		const owner = status.owner;
		if (!owner || status.phase !== "build" || owner.last_plan_revision !== state.candidate) throw new Error("BUILD exhaustion decision changed; inspect status and restart discussion");
		const answer = state.answers[state.findings[0]?.id]?.answer?.trim();
		if (!answer || !/^return to scope\s*:/i.test(answer)) throw new Error("BUILD exhaustion can continue only through an explicitly confirmed `Return to SCOPE: …` decision");
		const transitioned = await runSuperdev([
			"workflow", "transition", "--session", owner.session_id,
			"--expected-revision", owner.last_plan_revision, "--phase", "build",
			"--transition", "return-to-scope", "--feedback", `${answer}\n${JSON.stringify(state.mechanicalFindings ?? [])}`,
		], ctx.cwd, authority) as { result?: { state?: { last_plan_revision?: string } } };
		const revision = transitioned.result?.state?.last_plan_revision;
		if (!revision) throw new Error("BUILD-to-SCOPE exhaustion routing omitted the new plan revision");
		return await runScopePhase("", ctx, { ...state, candidate: revision, originPhase: "scope", status: "submitted" });
	};
	const approveAccept = async (status: any, ctx: any) => {
		runtime.currentStage = "acceptance integration";
		const owner = status.owner;
		if (!owner || status.phase !== "accept" || !owner.verified_default_revision) throw new Error("an evidence-bound ACCEPT workflow is required");
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
	const executeAcceptPhase = async (_args: string, ctx: any, commandAbort: AbortController) => {
			runtime.currentStage = "accept initialization";
			if (hasPendingQuestions()) return ctx.ui.notify("Resolve or pause the active workflow questions before starting a phase role", "error");
			const status = await workflowStatus(ctx.cwd);
			const owner = status.owner;
			if (!owner || status.phase !== "accept") throw new Error("An owned ACCEPT workflow is required");
			if (!owner.candidate_revision || !owner.verified_default_revision) throw new Error("ACCEPT state is missing candidate-bound BUILD evidence");
			const acceptHeadBefore = await pi.exec("git", ["rev-parse", "HEAD"], { cwd: ctx.cwd });
			const acceptCleanBefore = await pi.exec("git", ["status", "--porcelain"], { cwd: ctx.cwd });
			if (acceptHeadBefore.code !== 0 || acceptCleanBefore.code !== 0 || acceptCleanBefore.stdout.trim()
				|| acceptHeadBefore.stdout.trim() !== owner.candidate_revision) throw new Error("ACCEPT assessment requires the clean immutable candidate at HEAD");
			runtime.currentStage = "accept assessment";
			const decision = await withProgress(ctx, { key: "superdev-workflow", title: `ACCEPT ${owner.identity.plan}`, stage: "assessment" },
				(signal, update) => isolated(
					"accept",
					`Assess whether immutable candidate ${owner.candidate_revision} is ready for the parent-owned configured acceptance decision.`,
					ctx.cwd,
					ctx.model,
					phaseSignal(signal, commandAbort),
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
				runtime.currentStage = "accept finding routing";
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
					runtime.modifyingBusy = false;
					runtime.modifyingCommandAbort = undefined;
					return await startBuildPhase(`Correct this complete ACCEPT finding set: ${JSON.stringify(decision.findings)}`, ctx);
				}
				throw new Error("BUILD driver is unavailable for ACCEPT correction");
			}
			if (decision.status !== "complete") {
				publishRoleOutcome("accept", decision);
				return ctx.ui.notify(decision.summary, "error");
			}
			const currentDefault = await pi.exec("git", ["rev-parse", "--verify", owner.identity.default_branch], { cwd: ctx.cwd });
			if (currentDefault.code !== 0) throw new Error("could not resolve the configured default branch");
			if (currentDefault.stdout.trim() !== owner.verified_default_revision) {
				runtime.currentStage = "accept stale-default recovery";
				await runSuperdev([
					"workflow", "transition", "--session", owner.session_id,
					"--expected-revision", owner.last_plan_revision, "--phase", "accept",
					"--transition", "recover-stale-default",
				], ctx.cwd, authority);
				ctx.ui.setStatus("superdev-workflow", `BUILD: ${owner.identity.plan}`);
				runtime.lastOutcome = { status: "routed", phase: "build", summary: "Default branch advanced; final evidence was invalidated." };
				return ctx.ui.notify("Default branch advanced; final evidence was invalidated and workflow returned to BUILD", "warning");
			}
			const humanAcceptanceRequired = requiresHumanAcceptance(status.humanAcceptanceRequired);
			if (humanAcceptanceRequired) {
				runtime.lastOutcome = {
					status: "ready-for-approval",
					phase: "accept",
					workflow: owner.identity,
					expectedRevision: owner.last_plan_revision,
					recommendation: "Review the clean acceptance assessment, then approve it or discuss one routed change through the ACCEPT skill.",
				};
				ctx.ui.notify("ACCEPT assessment is clean; the ACCEPT skill now owns discussion and approval", "info");
				return;
			}
			await approveAccept(status, ctx);
			runtime.lastOutcome = { status: "accepted", phase: null, workflow: owner.identity };
	};
	runAcceptPhase = async (args, ctx) => {
		const inheritedAbort = runtime.modifyingCommandAbort;
		const commandAbort = inheritedAbort ?? new AbortController();
		const ownsRuntime = !inheritedAbort;
		if (ownsRuntime) {
			if (runtime.cancelling || runtime.modifyingBusy || runtime.modifyingChild) return ctx.ui.notify("A modifying workflow role is already active", "error");
			runtime.modifyingBusy = true;
			runtime.modifyingCommandAbort = commandAbort;
		}
		try {
			return await executeAcceptPhase(args, ctx, commandAbort);
		} catch (error) {
			await pauseWithOutcome(ctx, "ACCEPT", error, commandAbort.signal.aborted);
		} finally {
			if (ownsRuntime && runtime.modifyingCommandAbort === commandAbort) {
				runtime.modifyingCommandAbort = undefined;
				runtime.modifyingBusy = false;
			}
		}
	};
	const continueAcceptRequest = async (state, ctx) => {
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
	return {
		continueScopeFromAnswers,
		continueBuildDecision,
		continueAcceptRequest,
		runScopePhase,
		runBuildPhase,
		runAcceptPhase: (args: string, ctx: any) => runAcceptPhase!(args, ctx),
		approveAccept,
	};
}

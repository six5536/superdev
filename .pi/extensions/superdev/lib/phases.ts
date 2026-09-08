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
};

export function registerPhaseDrivers(deps: any) {
	const { pi, runSuperdev, workflowStatus, authority, policyFrom, questions, isolated, childStarted, childFinished, reviewRuns, parentServiceDigest, requiresHumanAcceptance, runtime } = deps;
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
		if (runtime.cancelling || runtime.modifyingBusy || runtime.modifyingChild) return ctx.ui.notify("A modifying workflow role is already active", "error");
		runtime.modifyingBusy = true;
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
	pi.registerCommand("scope", {
		description: "Run SCOPE through exhaustive review and human approval",
		handler: (args, ctx) => runScopePhase(args, ctx),
	});
	let runAcceptPhase: ((args: string, ctx: any) => Promise<void>) | undefined;
	const runBuildPhase = async (args: string, ctx: any) => {
			if (runtime.cancelling || runtime.modifyingBusy || runtime.modifyingChild) return ctx.ui.notify("A modifying workflow role is already active", "error");
			runtime.modifyingBusy = true;
			const commandAbort = new AbortController();
			runtime.modifyingCommandAbort = commandAbort;
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
				if (runtime.modifyingCommandAbort === commandAbort) runtime.modifyingCommandAbort = undefined;
				runtime.modifyingBusy = false;
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
					runtime.modifyingBusy = false;
					runtime.modifyingCommandAbort = undefined;
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
	pi.registerCommand("accept", {
		description: "Assess ACCEPT and integrate after configured human authority",
		handler: (args, ctx) => runAcceptPhase!(args, ctx),
	});
	return { continueScopeFromAnswers, continueAcceptRequest };
}

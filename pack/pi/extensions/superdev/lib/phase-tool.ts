import { Type } from "typebox";
import { StringEnum } from "@earendil-works/pi-ai";
import { boundedText } from "./output.ts";

export function registerPhaseTool(deps: any) {
 const { pi, workflowStatus, questions, runSuperdev, authority, policyFrom, runtime, phaseContinuations } = deps;
	pi.registerTool({
		name: "superdev_run_phase",
		label: "Run Superdev phase",
		description: "Run or continue one workflow phase using human-level operations; ownership and revision mechanics remain internal",
		parameters: Type.Object({
			phase: StringEnum(["scope", "build", "accept"] as const),
			action: StringEnum(["run", "retry", "inspect", "ask", "record-answer", "revise-answer", "submit-answers", "approve", "cancel"] as const),
			intent: Type.Optional(Type.String()),
			issue: Type.Optional(Type.String()),
			plan: Type.Optional(Type.String()),
			workBranch: Type.Optional(Type.String()),
			defaultBranch: Type.Optional(Type.String()),
			findingIds: Type.Optional(Type.Array(Type.String(), { description: "For ask, supply exactly one dependency-eligible finding ID from inspect. For record-answer, name every finding the answer covers." })),
			answer: Type.Optional(Type.String()),
			destination: Type.Optional(StringEnum(["scope", "build"] as const)),
			offset: Type.Optional(Type.Number({ minimum: 0 })),
			limit: Type.Optional(Type.Number({ minimum: 1, maximum: 20 })),
		}),
		executionMode: "sequential",
		execute: async (_id, input, _signal, _update, ctx) => {
            ctx = Object.assign(Object.create(ctx), { workflowSignal: _signal });
			const respond = (result: Record<string, unknown>) => ({ content: [{ type: "text", text: JSON.stringify(result) }], details: result });
			let status: any;
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
            if (status.busy) return respond({ status: "busy", phase: input.phase });
            // Cancel is the recovery path out of a foreign claim, so it runs
            // ahead of the ownership guard. Blocking it here is what leaves a
            // claim from a stopped session with no way out.
            if (!["inspect", "cancel"].includes(input.action) && status.owner && status.owner.session_id !== ctx.sessionManager.getSessionId()) {
                throw new Error(`This checkout is claimed by Pi session ${status.owner.session_id}, which is still running or cannot be proven stopped. Cancel to release the claim, then run the phase again.`);
            }
			if (input.action === "inspect") {
				const questionState = questions.snapshot(input.offset, input.limit);
				const publicQuestions = questionState ? { ...questionState, candidate: undefined } : null;
				return respond({
					status: status.busy ? "busy" : status.owner ? "active" : status.openWorkflows?.length ? "paused" : "idle",
					phase: status.phase ?? null,
					workflow: status.owner?.identity,
					openWorkflows: status.openWorkflows ?? [],
                    defaultBranch: status.defaultBranch,
                    defaultRevision: status.defaultRevision,
					questions: publicQuestions,
				});
			}
			if (input.action === "cancel") {
				if (questions.current()?.status === "active") {
					await questions.operate({ action: "pause" }, ctx);
					return respond({ status: "paused", phase: input.phase, partialWorkPreserved: true });
				}
				if (!status.owner) return respond({ status: "idle", phase: null });
				// A claim the service could not prove abandoned, held by a session
				// that is not this one, is decided by the human. Releasing it only
				// pauses: the phase, the plan, and every commit stay in Git.
				if (status.owner.session_id !== ctx.sessionManager.getSessionId()) {
					if (!ctx.hasUI) return respond({ status: "human-input-required", phase: status.phase, recommendation: "Releasing another Pi session's claim requires an interactive Pi session." });
					if (!(await ctx.ui.confirm("Release the other Pi session's claim?", `${status.owner.identity.plan} is claimed by Pi session ${status.owner.session_id}, which cannot be proven to be running. Releasing pauses the workflow; no canonical work is lost.`))) {
						return respond({ status: "approval-declined", phase: status.phase });
					}
					await runSuperdev(["workflow", "cancel", "--session", status.owner.session_id, "--human-release"], ctx.cwd, authority);
					return respond({ status: "paused", phase: status.phase, partialWorkPreserved: true });
				}
				await runSuperdev(["workflow", "cancel", "--session", status.owner.session_id], ctx.cwd, authority);
				return respond({ status: "paused", phase: status.phase, partialWorkPreserved: true });
			}
			if (["ask", "record-answer", "revise-answer", "submit-answers"].includes(input.action)) {
				let pending = questions.current();
				if (pending && ["submitted", "superseded"].includes(pending.status)) pending = undefined;
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
				if (input.action === "ask") {
					const eligible = pending.findings.filter((finding) => !pending.answers[finding.id]
						&& (finding.dependsOn ?? []).every((id) => pending.answers[id]));
					if (input.findingIds?.length !== 1 || !eligible.some((finding) => finding.id === input.findingIds[0])) {
						return respond({
							status: "finding-selection-required", phase: input.phase,
							eligibleFindingIds: eligible.slice(0, 20).map((finding) => finding.id),
							eligibleTotal: eligible.length,
							recommendation: "Inspect the queue, then call ask with findingIds containing exactly one unanswered, dependency-eligible ID. If all answers are confirmed, use submit-answers; use revise-answer to reopen an answer.",
						});
					}
                    const result = await questions.operate({ action: "ask", findingId: input.findingIds[0] }, ctx);
                    return result;
                }
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
					default_branch: input.defaultBranch ?? status.defaultBranch,
				} : status.openWorkflows?.length === 1 ? status.openWorkflows[0] : undefined;
				if (!requested) return respond({
					status: "selection-required",
					phase: input.phase,
					openWorkflows: status.openWorkflows ?? [],
					recommendation: "Select one exact issue, plan, and issue-derived work branch semantically.",
				});
				const existing = status.openWorkflows?.find((workflow) => workflow.issue === requested.issue && workflow.plan === requested.plan && workflow.work_branch === requested.work_branch);
				if (!existing && input.phase !== "scope") throw new Error("Only SCOPE can establish a new workflow.");
				// Record this Pi process with the claim, so a session that exits
				// without pausing leaves a decidably abandoned claim rather than one
				// that blocks the checkout forever.
				const identity = ["--session", ctx.sessionManager.getSessionId(), "--issue", requested.issue, "--plan", requested.plan, "--work-branch", requested.work_branch, "--default-branch", existing?.default_branch ?? requested.default_branch, "--owner-pid", String(process.pid)];
				try {
					await runSuperdev(["workflow", existing ? "resume" : "start", ...identity], ctx.cwd, authority);
					status = await workflowStatus(ctx.cwd);
				} catch (error) {
					return failed(existing ? "workflow resume" : "workflow start", error);
				}
			}
            if (input.phase === "scope" && ["build", "accept"].includes(status.phase ?? "") && input.intent?.trim()) {
                if (!ctx.hasUI || !(await ctx.ui.confirm("Return to SCOPE?", input.intent))) return respond({ status: "human-input-required", phase: status.phase });
                await runSuperdev(["workflow", "transition", "--session", status.owner!.session_id,
                    "--expected-revision", status.owner!.last_plan_revision, "--phase", status.phase!,
                    "--transition", status.phase === "build" ? "return-to-scope" : "reject-acceptance",
                    "--feedback", input.intent], ctx.cwd, authority);
                questions.supersede("human requested new scope");
                status = await workflowStatus(ctx.cwd);
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
				if (!(await ctx.ui.confirm(`Approve reviewed ${input.phase.toUpperCase()}?`, input.phase === "scope" ? `Approve ${status.owner.identity.plan} and advance it to BUILD?` : `Accept ${status.owner.identity.plan} and leave the branch ready for your merge?`))) {
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
}

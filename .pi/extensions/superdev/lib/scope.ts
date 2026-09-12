import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { WorkflowClient, steps, type ExecutionStage, type ScopeRecord, type ScopeStep, type WorkflowStatus } from "./client.ts";
import { HumanQuestions, validateQuestion, type Question, type Reply } from "./questions.ts";
import { ExecutionController } from "./execution.ts";
import { requiresHumanAcceptance } from "./process.ts";

export type PhaseInput = {
	phase: "scope" | "build" | "accept";
	action: "inspect" | "run" | "resume" | "pause" | "continue" | "record-step" | "approve-issue" | "approve-plan" | "attach-plan" | "assess-change" | "recover-publication" | "migrate" | "rescope" | "record-discussion" | "consume-retry"
		| "commit-block" | "complete-build" | "return-to-build" | "accept";
	workflow?: string; issue?: string; plan?: string; defaultBranch?: string;
	steps?: ScopeStep[]; step?: ScopeStep; note?: string; stage?: ExecutionStage;
	block?: number; areas?: string[];
	reset?: boolean; unfinished?: string; evidence?: string[];
	document?: "issue" | "plan"; fromHash?: string; formattingOnly?: boolean; indexes?: string[];
	completedBlocks?: number[];
};
const selectionEntry = "superdev-workflow-selection-v3";
const actionLabels: Record<ScopeStep, string> = {
	"select-issue": "select or draft the issue", "interview-issue": "interview about the issue",
	"write-issue": "write or update the issue", "check-issue": "double-check the issue", "approve-issue": "approve the issue",
	"write-plan": "write the initial plan", "interview-plan": "interview about the plan", "update-plan": "update the plan",
	"check-plan": "double-check the plan", "approve-plan": "approve the plan", handoff: "choose BUILD here, BUILD in a worker, or exit",
};
const permissionSteps = steps.filter((step) => !["approve-issue", "approve-plan", "handoff"].includes(step));
const controls = ["Continue", "Discuss", "Do something else", "Pause"];

/** The controller holds only the current continuation's step permission.
 * Rust remains the sole owner of document approval and durable transitions. */
export class ScopeController {
	private selected?: string;
	private snapshot?: WorkflowStatus;
	private permitted: ScopeStep[] = [];
	private proposedIssue?: string;

	readonly execution: ExecutionController;

	constructor(private readonly pi: ExtensionAPI, readonly client: WorkflowClient, readonly questions: HumanQuestions) {
		this.execution = new ExecutionController(client, questions);
	}

	restore(ctx: ExtensionContext) {
		this.permitted = []; this.questions.clear(); this.snapshot = undefined;
		this.selected = undefined; this.proposedIssue = undefined;
		for (const entry of ctx.sessionManager.getEntries()) {
			if (entry.type === "custom" && entry.customType === selectionEntry && typeof (entry.data as { workflow?: unknown })?.workflow === "string") {
				this.selected = (entry.data as { workflow: string }).workflow;
			}
		}
	}

	view() {
		const selected = this.snapshot?.workflows.find((item) => item.record.id === this.selected);
		return {
			workflow: selected ? { id: selected.record.id, issue: selected.record.issue, plan: selected.record.plan,
				phase: selected.record.phase, step: selected.record.scope_step,
				statePath: `.superdev/workflows/${selected.record.id}.json`,
				steps: selected.record.steps.slice(-12).map((entry) => ({ ...entry, note: entry.note.slice(0, 512) })), totalSteps: selected.record.steps.length,
				discussion: selected.record.discussion?.slice(-4_096), recovery: selected.record.recovery?.slice(0, 2_048),
				// The last assessment's findings survive the conversation that
				// received them, and say whether they still describe this candidate.
				lastAssessment: selected.record.assessment ? {
					stage: selected.record.assessment.stage,
					findings: selected.record.assessment.findings.slice(0, 8_192),
					void: selected.record.assessment.void,
					stale: selected.record.assessment.candidate !== selected.record.candidate,
				} : undefined,
				stateExcerpt: true,
				approval: selected.approval, workBranch: selected.record.work_branch,
				nextAction: actionLabels[selected.record.scope_step] } : undefined,
			availableTotal: this.snapshot?.workflows.length,
			available: this.snapshot?.workflows.slice(0, 20).map(({ record }) => ({ id: record.id, issue: record.issue, plan: record.plan, phase: record.phase, step: record.scope_step })),
			currentBranch: this.snapshot?.currentBranch, defaultBranch: this.snapshot?.defaultBranch,
			claim: this.snapshot?.claim ? { workflow: this.snapshot.claim.workflow, session: this.snapshot.claim.session, worker: this.snapshot.claim.child_pid } : undefined,
			permittedSteps: [...this.permitted], pendingQuestion: this.questions.view(), controls,
		};
	}

	async inspect(ctx: ExtensionContext) {
		this.snapshot = await this.client.status(ctx.cwd);
		if (!this.selected && this.snapshot.workflows.length === 1) this.selected = this.snapshot.workflows[0].record.id;
		return { status: "inspected", ...this.view() };
	}

	private async record(ctx: ExtensionContext): Promise<ScopeRecord> {
		await this.inspect(ctx);
		const record = this.snapshot!.workflows.find((item) => item.record.id === this.selected)?.record;
		if (!record) throw new Error("Select a local workflow from inspect; missing local approval must be reconstructed, not inferred from a document");
		return record;
	}

	private select(record: ScopeRecord) {
		if (this.selected !== record.id) this.pi.appendEntry(selectionEntry, { workflow: record.id });
		this.selected = record.id;
	}

	private async change(change: Record<string, unknown>, ctx: ExtensionContext, saved?: ScopeRecord) {
		const record = saved ?? await this.record(ctx);
		return this.client.apply({ operation: "change", id: record.id, expected_revision: record.revision, change }, ctx);
	}

	private async progress(update: { entry?: { step: ScopeStep; outcome: "completed" | "skipped"; note: string }; next?: ScopeStep; discussion?: string }, ctx: ExtensionContext) {
		const record = await this.record(ctx);
		// Keep every discussion result, not only the last question. Resume must
		// not repeat answered questions or erase open points at the next step.
		const discussion = [record.discussion, update.discussion].filter(Boolean).join("\n\n") || null;
		await this.change({ action: "record-step", entry: update.entry ?? null, next: update.next ?? record.scope_step, discussion }, ctx, record);
	}

	private async saveQuestion(question: Question, ctx: ExtensionContext) {
		await this.progress({ discussion: JSON.stringify({ question }) }, ctx);
	}

	private async control(reply: Reply, ctx: ExtensionContext): Promise<unknown> {
		if (reply.status === "paused") return this.pause(ctx);
		if (reply.status === "other-action") this.permitted = [];
		return { status: reply.status === "answered" ? "discuss" : reply.status,
			...(reply.status !== "answered" && reply.text ? { discussion: reply.text } : {}), ...this.view() };
	}

	async pause(ctx: ExtensionContext) {
		this.questions.clear(); this.permitted = []; this.proposedIssue = undefined;
		await this.inspect(ctx);
		const claim = this.snapshot?.claim;
		if (claim && claim.session === ctx.sessionManager.getSessionId()) {
			// A running worker owns the checkout, so it stops before ownership is
			// released. Its saved session and progress survive the pause.
			const selected = this.snapshot?.workflows.find((item) => item.record.id === claim.workflow)?.record;
			if (selected && this.execution.view().worker) await this.execution.pause(selected, ctx);
			await this.inspect(ctx);
			await this.client.apply({ operation: "pause", id: claim.workflow }, ctx);
		}
		await this.inspect(ctx);
		return { status: this.snapshot?.claim ? "owned-elsewhere" : "paused", partialWorkPreserved: true, ...this.view() };
	}

	async run(input: PhaseInput, ctx: ExtensionContext): Promise<unknown> {
		if (input.workflow && input.workflow !== this.selected) {
			if (this.questions.view() || this.permitted.length || this.snapshot?.claim?.session === ctx.sessionManager.getSessionId()) throw new Error("Pause the current continuation before selecting another workflow");
			this.selected = input.workflow;
		}
		if (input.action === "inspect") return this.inspect(ctx);
		if (input.action === "pause") return this.pause(ctx);
		await this.inspect(ctx);
		if (input.phase !== "scope") return this.execute(input, ctx);
		if (input.action === "run" && !this.snapshot!.workflows.some((item) => item.record.id === this.selected)) {
			if (!input.issue) return { status: "selection-required", ...this.view() };
			this.proposedIssue = input.issue;
			const branch = input.defaultBranch ?? this.snapshot!.defaultBranch;
			if (!branch) return { status: "branch-selection-required", reason: this.snapshot!.defaultBranchDiagnostic, ...this.view() };
			if (this.snapshot!.currentBranch !== branch) return { status: "branch-switch-required", expectedBranch: branch,
				recommendation: `Switch explicitly to ${branch}, then run SCOPE again. No document has been created.`, ...this.view() };
			return this.questions.ask({ question: `Begin SCOPE for ${input.issue} on ${branch}? No plan, approval, commit, or work branch will be created.`,
				choices: [{ id: "continue", label: "Begin SCOPE" }], recommendation: { choiceId: "continue", reason: "Reserve the issue before drafting it." } }, ctx, async (reply, ctx) => {
				if (reply.status !== "answered" || reply.choiceId !== "continue") return this.control(reply, ctx);
				const record = await this.client.apply({ operation: "create", issue: input.issue, default_branch: branch }, ctx);
				this.select(record); this.permitted = ["select-issue"]; this.proposedIssue = undefined;
				return this.inspect(ctx);
			});
		}
		if (input.action === "migrate") {
			if (!input.issue || !input.plan || !input.note) throw new Error("Migration requires exact issue/plan identities and recovery facts to inspect");
			return this.questions.ask({ question: `Reconstruct ${input.issue} / ${input.plan} locally without any approval?`,
				choices: [{ id: "continue", label: "Reconstruct unapproved progress" }], recommendation: { choiceId: "continue", reason: "Retain completed work as inspection facts; request fresh approval." } }, ctx, async (reply, ctx) => {
				if (reply.status !== "answered" || reply.choiceId !== "continue") return this.control(reply, ctx);
				const record = await this.client.apply({ operation: "migrate", issue: input.issue, plan: input.plan, default_branch: input.defaultBranch,
					checkpoint: { completed_blocks: input.completedBlocks ?? [], unfinished: input.note, evidence: [] } }, ctx);
				this.select(record); this.permitted = []; return this.inspect(ctx);
			});
		}
		const record = await this.record(ctx);
		if (input.action === "resume" || this.snapshot!.claim?.session !== ctx.sessionManager.getSessionId()) {
			return this.questions.ask({ question: `Resume ${record.plan ?? record.issue}? Saved step: ${actionLabels[record.scope_step]}.`,
				choices: [{ id: "continue", label: "Resume the saved workflow" }], recommendation: { choiceId: "continue", reason: "Keep valid approvals; ask fresh permission for the next SCOPE step." } }, ctx, async (reply, ctx) => {
				if (reply.status !== "answered" || reply.choiceId !== "continue") return this.control(reply, ctx);
				this.select(await this.client.apply({ operation: "resume", id: record.id }, ctx));
				this.permitted = []; return this.inspect(ctx);
			});
		}
		if (input.action === "rescope") {
			if (!input.note?.trim()) throw new Error("State the requested intent change before returning to SCOPE");
			return this.questions.ask({ question: `Return ${record.plan ?? record.issue} to SCOPE? ${input.note}`,
				choices: [{ id: "continue", label: "Return to human-led SCOPE" }], recommendation: { choiceId: "continue", reason: "Discuss the changed intent before further implementation." } }, ctx, async (reply, ctx) => {
				if (reply.status !== "answered" || reply.choiceId !== "continue") return this.control(reply, ctx);
				await this.change({ action: "return-to-scope", feedback: input.note, input: reply.input }, ctx);
				this.permitted = []; return this.inspect(ctx);
			});
		}
		if (record.phase !== "scope") return { status: "rescope-required", phase: record.phase, ...this.view() };
		if (input.action === "run" || input.action === "continue") return this.permission(input.steps ?? [record.scope_step], ctx);
		if (input.action === "record-discussion") {
			if (!input.note?.trim()) throw new Error("Supply the discussion points to retain");
			await this.progress({ discussion: input.note }, ctx);
			return this.inspect(ctx);
		}
		if (input.action === "record-step") {
			if (!input.step || input.step !== this.permitted[0]) throw new Error("The next recorded action must be one the human permitted in this continuation");
			await this.progress({ entry: { step: input.step, outcome: "completed", note: input.note ?? "" },
				next: this.permitted[1] ?? steps[steps.indexOf(input.step) + 1] ?? "handoff" }, ctx);
			this.permitted.shift(); return this.inspect(ctx);
		}
		if (input.action === "attach-plan") {
			if (!input.plan || this.permitted[0] !== "write-plan") throw new Error("Plan reservation requires permission to write the plan");
			await this.change({ action: "attach-plan", plan: input.plan }, ctx); return this.inspect(ctx);
		}
		if (input.action === "approve-issue" || input.action === "approve-plan") return this.approval(input.action === "approve-issue" ? "issue" : "plan", input.indexes ?? [], ctx);
		if (input.action === "assess-change") {
			if (!input.document || !input.fromHash || input.formattingOnly === undefined || !input.note) throw new Error("Diff assessment requires document, baseline, classification, and actual diff evidence");
			const to = await this.hash(record, input.document);
			await this.change({ action: "assess-change", document: input.document, from: input.fromHash, to,
				formatting_only: input.formattingOnly, reason: input.note }, ctx);
			return this.inspect(ctx);
		}
		if (input.action === "recover-publication") {
			await this.change({ action: "recover-publication" }, ctx); return this.inspect(ctx);
		}
		throw new Error(`Unsupported SCOPE operation: ${input.action}`);
	}

	/** BUILD and ACCEPT stage execution. Stage checklists remain the skills' work. */
	private async execute(input: PhaseInput, ctx: ExtensionContext): Promise<unknown> {
		const record = await this.record(ctx);
		if (record.phase === "scope") return { status: "build-not-started", ...this.view() };
		if (input.action === "resume") {
			if (record.execution_mode !== "worker") return { status: "resumed", contextReset: "unavailable-in-current-session", ...this.view() };
			await this.execution.attach(record, ctx);
			return { status: "resumed", ...this.view(), ...this.execution.view() };
		}
		if (input.action === "run" || input.action === "continue") {
			if (!input.note?.trim()) throw new Error("Supply the stage instructions to execute");
			const stage = input.stage ?? record.stage ?? "implementation";
			const result = await this.execution.run(record, { stage, instructions: input.note,
				reset: input.reset, unfinished: input.unfinished, evidence: input.evidence }, ctx);
			// Executing here returns the stage's own checklist and facts, because
			// this conversation has no worker to inject them into.
			return { status: result.assessment === "void" ? "assessment-void"
					: result.executeHere ? "execute-here" : "stage-complete", stage,
				contextReset: result.contextReset, stageContext: result.context,
				assessment: result.assessment, diagnostic: result.diagnostic,
				result: result.text, ...this.view() };
		}
		if (input.action === "consume-retry") {
			if (!input.note?.trim()) throw new Error("Name the budgeted activity to consume an attempt from");
			await this.execution.retry(record, input.note, ctx);
			return this.inspect(ctx);
		}
		if (input.action === "commit-block") {
			if (!input.block || !input.note?.trim()) throw new Error("A block commit requires its plan block number and commit message");
			await this.execution.commitBlock(record, input.block, input.note, input.areas ?? [], ctx);
			return this.inspect(ctx);
		}
		if (input.action === "complete-build") {
			await this.execution.completeBuild(record, ctx); return this.inspect(ctx);
		}
		if (input.action === "return-to-build") {
			if (!input.note?.trim()) throw new Error("State the findings BUILD must correct");
			await this.execution.returnToBuild(record, input.note, ctx);
			return this.inspect(ctx);
		}
		if (input.action === "accept") return this.acceptance(record, ctx);
		return { status: "unsupported-execution-action", action: input.action, ...this.view() };
	}

	/**
	 * Close the workflow under project policy.
	 *
	 * The policy comes from the service, never from the model. An unreadable
	 * policy refuses rather than accepting automatically.
	 */
	private async acceptance(record: ScopeRecord, ctx: ExtensionContext): Promise<unknown> {
		if (record.phase !== "accept") return { status: "accept-not-reached", phase: record.phase, ...this.view() };
		if (!requiresHumanAcceptance(this.snapshot?.humanAcceptanceRequired)) {
			await this.execution.accept(record, undefined, ctx);
			return { status: "accepted", acceptance: "automatic", ...await this.inspect(ctx) };
		}
		const question = { question: `Accept ${record.plan ?? record.issue} at candidate ${record.candidate ?? "unknown"}? Acceptance does not merge, push, or release it.`,
			choices: [{ id: "accept", label: "Accept this candidate" }],
			recommendation: { choiceId: "accept", reason: "Accept only if the candidate meets the approved plan; otherwise discuss or return findings." } };
		return this.questions.ask(question, ctx, async (reply, ctx) => {
			if (reply.status !== "answered" || reply.choiceId !== "accept") return this.control(reply, ctx);
			const current = await this.record(ctx);
			if (current.id !== record.id || current.candidate !== record.candidate) throw new Error("The candidate changed; reassess it before accepting");
			await this.execution.accept(current, reply.input, ctx);
			return { status: "accepted", acceptance: "human", ...await this.inspect(ctx) };
		});
	}

	private async permission(requested: ScopeStep[], ctx: ExtensionContext) {
		if (this.permitted.length) return { status: "permitted", ...this.view() };
		if (requested.length === 1 && requested[0] === "approve-issue") return this.approval("issue", [], ctx);
		if (requested.length === 1 && requested[0] === "approve-plan") return this.approval("plan", [], ctx);
		if (requested.length === 1 && requested[0] === "handoff") return this.handoff(ctx);
		if (!requested.length || new Set(requested).size !== requested.length || requested.some((step) => !permissionSteps.includes(step))) {
			throw new Error("Name an explicit group of drafting/interview/check steps; document approvals are separate actions");
		}
		const question = { question: `Current: ${this.view().workflow!.nextAction}. Next: ${requested.map((step) => actionLabels[step]).join("; then ")}.`,
			choices: [{ id: "continue", label: "Continue with these named steps" }, { id: "skip", label: "Skip these named steps" }],
			recommendation: { choiceId: "continue", reason: "Perform the named actions, then save their results." } };
		await this.saveQuestion(question, ctx);
		return this.questions.ask(question, ctx, async (reply, ctx) => {
			if (reply.status !== "answered") return this.control(reply, ctx);
			if (reply.choiceId === "continue") {
				await this.progress({ next: requested[0] }, ctx);
				this.permitted = [...requested];
			} else if (reply.choiceId === "skip") {
				for (const step of requested) await this.progress({ entry: { step, outcome: "skipped", note: reply.input.text },
					next: steps[steps.indexOf(step) + 1] ?? "handoff" }, ctx);
			} else return { status: "discuss", answer: reply.answer, ...this.view() };
			await this.inspect(ctx); return { status: "permitted", ...this.view() };
		});
	}

	private async hash(record: ScopeRecord, kind: "issue" | "plan") {
		const id = kind === "issue" ? record.issue : record.plan;
		if (!id) throw new Error("Select a plan before asking for its approval");
		return createHash("sha256").update(await readFile(resolve(record.checkout, `knowledge/${kind}s/open/${id}.md`))).digest("hex");
	}

	private async approval(kind: "issue" | "plan", indexes: string[], ctx: ExtensionContext) {
		const record = await this.record(ctx);
		const id = kind === "issue" ? record.issue : record.plan;
		if (!id) throw new Error("No plan is selected");
		const hash = await this.hash(record, kind);
		const issueHash = kind === "plan" ? await this.hash(record, "issue") : undefined;
		if (issueHash && (!record.issue_approval || (record.issue_approval.original.hash !== issueHash && !record.issue_approval.compatible.includes(issueHash)))) {
			throw new Error("Approve the current issue revision before asking for plan approval");
		}
		const question = { question: `Approve ${id} at ${hash}? Open findings do not prevent your approval.`,
			choices: [{ id: "approve", label: `Approve ${id}` }], recommendation: { choiceId: "approve", reason: "Approve only if this revision states your intent; otherwise discuss or choose another action." } };
		await this.saveQuestion(question, ctx);
		return this.questions.ask(question, ctx, async (reply, ctx) => {
			if (reply.status !== "answered" || reply.choiceId !== "approve") return this.control(reply, ctx);
			const current = await this.record(ctx);
			if (current.id !== record.id || await this.hash(current, kind) !== hash || (kind === "plan" && await this.hash(current, "issue") !== issueHash)) throw new Error("The approval target changed; inspect before asking about its new revision");
			// Direct advancement is the human's choice. Record unperformed actions
			// as skipped without inventing clean interview or review results.
			const target = kind === "issue" ? "approve-issue" : "approve-plan";
			const start = steps.indexOf(current.scope_step), end = steps.indexOf(target);
			for (const step of steps.slice(start, end)) {
				if (!permissionSteps.includes(step)) continue;
				await this.progress({ entry: { step, outcome: "skipped", note: "Human advanced directly to document approval; this action was not recorded as performed." },
					next: target }, ctx);
			}
			await this.change({ action: "approve", document: kind, expected_hash: hash, input: reply.input, indexes }, ctx);
			this.permitted = []; await this.inspect(ctx);
			return { status: "approved", document: id, ...this.view() };
		}, { kind, id, hash });
	}

	async interview(question: Question, ctx: ExtensionContext) {
		validateQuestion(question);
		const selected = this.snapshot?.workflows.find((item) => item.record.id === this.selected)?.record;
		if (selected && selected.phase === "scope") await this.saveQuestion(question, ctx);
		return this.questions.ask(question, ctx, async (reply, ctx) => {
			if (reply.status !== "answered") return this.control(reply, ctx);
			if (selected?.phase === "scope") await this.progress({
				discussion: JSON.stringify({ question, answer: reply.answer, choiceId: reply.choiceId }) }, ctx);
			return { status: "answered", answer: reply.answer, choiceId: reply.choiceId };
		});
	}

	private async handoff(ctx: ExtensionContext) {
		await this.record(ctx);
		if (!this.snapshot?.workflows.find((item) => item.record.id === this.selected)?.approval.executable) {
			return { status: "approval-required", ...this.view() };
		}
		return this.questions.ask({ question: "The plan is approved. How should SCOPE end?",
			choices: [{ id: "current", label: "BUILD here" }, { id: "worker", label: "BUILD in one worker" }, { id: "exit", label: "Exit SCOPE without starting BUILD" }],
			recommendation: { choiceId: "worker", reason: "A worker will keep the controlling conversation available." } }, ctx, async (reply, ctx) => {
			if (reply.status !== "answered") return this.control(reply, ctx);
			if (reply.choiceId === "exit") return this.pause(ctx);
			const mode = reply.choiceId === "worker" ? "worker" : "current";
			const started = await this.execution.start(await this.record(ctx), mode, reply.input, ctx);
			this.permitted = []; await this.inspect(ctx);
			return { status: "build-started", mode, contextReset: started.contextReset,
				workBranch: started.record.work_branch, stage: started.record.stage, ...this.view() };
		});
	}

	/**
	 * End open work on explicit human authority.
	 *
	 * No model path reaches this: it exists only as a typed command, so the
	 * decision belongs to the person at the keyboard. Partial product work stays
	 * on its branch and is never merged, pushed, or deleted.
	 */
	async abandon(reason: string, ctx: ExtensionContext): Promise<unknown> {
		if (!reason.trim()) throw new Error("State why this work is abandoned");
		const record = await this.record(ctx);
		if (record.phase === "done" || record.phase === "abandoned") return { status: "already-closed", phase: record.phase, ...this.view() };
		const question = { question: `Abandon ${record.plan ?? record.issue}? ${reason}`,
			choices: [{ id: "abandon", label: "Abandon this work" }],
			recommendation: { choiceId: "abandon", reason: "Partial product work stays on its branch; it is not merged or deleted." } };
		return this.questions.ask(question, ctx, async (reply, ctx) => {
			if (reply.status !== "answered" || reply.choiceId !== "abandon") return this.control(reply, ctx);
			const current = await this.record(ctx);
			if (current.id !== record.id) throw new Error("The selected workflow changed; inspect before abandoning");
			// A running worker owns the checkout, so it stops before closure.
			if (this.execution.view().worker) await this.execution.pause(current, ctx);
			await this.execution.abandon(await this.record(ctx), reason, reply.input, ctx);
			this.permitted = [];
			return { status: "abandoned", partialWorkPreserved: true, ...await this.inspect(ctx) };
		});
	}

	/** Enforce selected-document drafting permission and service-only local state. */
	guard(path: string, cwd: string): string | undefined {
		const record = this.snapshot?.workflows.find((item) => item.record.id === this.selected)?.record;
		const absolute = resolve(cwd, path);
		if (/[/\\\\]\.superdev[/\\\\](workflows|cache)([/\\\\]|$)/.test(absolute)) return "Local workflow state is written only by the Rust service";
		if (!record) return this.proposedIssue && absolute.endsWith(`/${this.proposedIssue}.md`) ? "Begin SCOPE on the correct branch before drafting" : undefined;
		for (const [kind, id] of [["issue", record.issue], ["plan", record.plan]] as const) {
			if (!id || absolute !== resolve(record.checkout, `knowledge/${kind}s/open/${id}.md`)) continue;
			const allowed = kind === "issue" ? ["select-issue", "write-issue"] : ["write-plan", "update-plan"];
			if (!allowed.includes(this.permitted[0])) return "Request human permission for this drafting step before editing the document";
		}
		return undefined;
	}
}

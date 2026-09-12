// Controller-side execution: start BUILD, run stages, reset context, pause.
//
// Durable facts live in the Rust record. This object holds only the live
// worker handle, so losing it loses no progress, approval, or retry budget.
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import { WorkflowClient, type AssessmentReport, type Checkpoint, type ExecutionStage, type ScopeRecord } from "./client.ts";
import type { HumanInput, HumanQuestions } from "./questions.ts";
import { WorkerSession } from "./worker.ts";

const here = dirname(fileURLToPath(import.meta.url));
/** Stage order. Each entry is rebuilt from durable facts, never from old context. */
export const stageOrder: ExecutionStage[] = ["implementation", "verification", "review", "acceptance"];
/** Stages that must not inherit the implementation conversation or its verdicts. */
export const resetBeforeStage: ExecutionStage[] = ["review", "acceptance"];
/** The checklist each stage rebuilds from. Review stays in BUILD, after its reset. */
const stageSkill: Record<ExecutionStage, string> = {
	implementation: "build", verification: "build", review: "build", acceptance: "accept",
};

export type StageRequest = {
	stage: ExecutionStage;
	instructions: string;
	/** Force a reset; review and acceptance reset by default. */
	reset?: boolean;
	/** Partial work and uncertain outcomes to inspect before continuing. */
	unfinished?: string;
	/** Candidate-bound verification evidence produced by this stage. */
	evidence?: string[];
};

/** One execution owner per controller. The worker is the only checkout writer. */
export class ExecutionController {
	private worker?: WorkerSession;

	constructor(private readonly client: WorkflowClient, private readonly questions: HumanQuestions) {}

	view() {
		return { worker: this.worker?.view() };
	}

	private change(record: ScopeRecord, change: Record<string, unknown>, ctx: ExtensionContext) {
		return this.client.apply({ operation: "change", id: record.id, expected_revision: record.revision, change }, ctx);
	}

	/**
	 * Start execution on the human's explicit permission.
	 *
	 * The branch is created by Rust, which also refuses a dirty worktree or a
	 * pre-existing branch. A worker is attached only after that succeeds.
	 */
	async start(record: ScopeRecord, mode: "current" | "worker", input: HumanInput, ctx: ExtensionContext) {
		let started = await this.change(record, { action: "start-build", mode, input }, ctx);
		if (mode === "current") {
			// A tool context cannot navigate the session tree, so the current
			// conversation cannot be reset between stages. Disclose that rather
			// than pretending each stage begins with clean assessment context.
			return { record: started, contextReset: "unavailable-in-current-session" as const };
		}
		started = await this.attach(started, ctx);
		return { record: started, contextReset: "worker-session" as const };
	}

	/** Attach, or reattach, the one persistent worker for this workflow. */
	async attach(record: ScopeRecord, ctx: ExtensionContext): Promise<ScopeRecord> {
		if (this.worker?.running) throw new Error("A worker is already running for this checkout");
		const worker = new WorkerSession({
			// Worker questions reach the human through the controller's single
			// question interface; the worker itself receives no human input. The
			// human's control passes through unchanged, so Discuss and Pause are
			// not delivered to the worker as an answer nobody gave.
			ask: async (question, questionCtx) => {
				const reply = await this.questions.ask(question, questionCtx, async (result) => result);
				const chosen = reply as { status?: string; answer?: string; choiceId?: string };
				const status = chosen.status === "answered" || chosen.status === "discuss"
					|| chosen.status === "other-action" || chosen.status === "paused" ? chosen.status : "other-action";
				return { status, answer: chosen.answer, choiceId: chosen.choiceId };
			},
		});
		const identity = await worker.start({
			agentDir: process.env.PI_CODING_AGENT_DIR ?? join(ctx.cwd, ".superdev/agent"),
			// Worker transcripts stay under the already-ignored cache directory:
			// recovery data, not approval or progress authority.
			sessionsDir: join(ctx.cwd, ".superdev/cache/workers"),
			sessionFile: record.worker_session?.session_file,
		}, ctx);
		this.worker = worker;
		return this.change(record, { action: "attach-worker", pid: worker.view().pid,
			worker: { session: identity.session, session_file: identity.sessionFile, anchor: identity.anchor } }, ctx);
	}

	/**
	 * The stage's own checklist and the durable facts it needs.
	 *
	 * A reset empties the worker's conversation, so everything the stage relies
	 * on is rebuilt here: its checklist, the approved records, the candidate,
	 * and what earlier stages completed or left unfinished. Review and
	 * acceptance receive no implementation conversation and no earlier verdict.
	 */
	private async stageContext(record: ScopeRecord, stage: ExecutionStage): Promise<string> {
		const checklist = await readFile(join(here, `../skills/${stageSkill[stage]}/SKILL.md`), "utf8");
		const assessing = stage === "review" || stage === "acceptance";
		const facts = {
			stage,
			issue: record.issue,
			plan: record.plan,
			issuePath: `knowledge/issues/open/${record.issue}.md`,
			planPath: record.plan ? `knowledge/plans/open/${record.plan}.md` : undefined,
			workBranch: record.work_branch,
			candidate: record.candidate,
			completedBlocks: record.checkpoint.completed_blocks,
			unfinished: record.checkpoint.unfinished || undefined,
			evidence: record.checkpoint.evidence,
			consumedRetries: record.retries,
			readOnly: assessing,
			// The last assessment's own findings. An implementation stage reads
			// them to correct; a fresh assessment reads them to see what the
			// previous one said about a candidate that has since changed.
			lastAssessment: record.assessment,
		};
		return [
			`# ${stage.toUpperCase()} stage`,
			checklist,
			"## Durable facts",
			"Rebuild this stage from these facts and the named documents. No earlier conversation is available.",
			"```json", JSON.stringify(facts, null, 1), "```",
			assessing
				? [
					"Assess the candidate against the approved records and the recorded evidence.",
					"Run whatever inspection helps: diffs, searches, type-checks, the project's own test commands.",
					"Report findings; do not correct them. Leave the checkout exactly as you found it:",
					"its Git state is compared before and after this stage, and any change voids the assessment.",
					"Write scratch files outside the checkout if you need them.",
				].join(" ")
				: "Stay within the approved plan. Record results honestly, including failures.",
		].join("\n\n");
	}

	/**
	 * The checkout's complete Git state: branch, HEAD, and every pending change.
	 *
	 * Assessment is bounded by comparing this before and after the stage, rather
	 * than by shortening its tool list. A shell reaches any effect through a
	 * variable, a script, or a build tool, so the reachable commands are not
	 * enumerable; the outcome is.
	 */
	private async worktreeState(ctx: ExtensionContext): Promise<string | undefined> {
		return (await this.client.status(ctx.cwd)).worktreeState;
	}

	/**
	 * Run one stage and save its durable outcome.
	 *
	 * Progress is recorded after the stage settles, so an interrupted stage is
	 * never stored as completed work.
	 */
	async run(record: ScopeRecord, request: StageRequest, ctx: ExtensionContext, signal?: AbortSignal) {
		// Unfinished work and evidence are saved even when the stage is performed
		// in this conversation, so a crash or reset leaves notes to resume from.
		const checkpoint: Checkpoint = {
			completed_blocks: record.checkpoint.completed_blocks,
			unfinished: request.unfinished ?? record.checkpoint.unfinished,
			evidence: request.evidence?.length
				? [...record.checkpoint.evidence, ...request.evidence]
				: record.checkpoint.evidence,
		};
		const save = (updated: ScopeRecord, assessment?: AssessmentReport) =>
			this.change(updated, { action: "record-progress", stage: request.stage, checkpoint,
				candidate: updated.candidate ?? null, assessment: assessment ?? null }, ctx);
		if (!this.worker?.running) {
			if (record.execution_mode === "worker") throw new Error("The worker is not running; resume execution before running a stage");
			// In the current conversation there is no separate executor, so the
			// stage is performed here. A tool context cannot reset the session
			// tree, so the absent reset is disclosed rather than implied.
			return { record: await save(record), executeHere: true,
				context: await this.stageContext(record, request.stage),
				contextReset: "unavailable-in-current-session" as const, text: "" };
		}
		// Checkpoint before the reset, so a failure between them loses no facts.
		const reset = request.reset ?? resetBeforeStage.includes(request.stage);
		const saved = reset ? await save(record) : record;
		if (reset) await this.worker.reset(signal);
		// An assessment must judge the candidate BUILD committed, so the checkout
		// it started from is the one it has to end with.
		const assessing = resetBeforeStage.includes(request.stage);
		const before = assessing ? await this.worktreeState(ctx) : undefined;
		const text = await this.worker.run(request.stage, request.instructions,
			await this.stageContext(saved, request.stage), signal);
		const after = assessing ? await this.worktreeState(ctx) : undefined;
		const contextReset = reset ? ("worker-session" as const) : ("not-requested" as const);
		if (assessing && (before === undefined || after === undefined || before !== after)) {
			// The assessment judged a moving target, so its verdict cannot stand.
			// Its findings are still retained — they may be sound — but the record
			// says why, so nothing later mistakes this for a clean pass.
			const diagnostic = before === undefined || after === undefined
				? `The ${request.stage} stage ran but its checkout state could not be read, so it cannot be shown to have judged an unchanged candidate.`
				: `The ${request.stage} stage changed the checkout. Inspect and restore it: the assessment judged a candidate that no longer matches the one it started from.`;
			const voided = await this.change(saved, { action: "record-progress", stage: request.stage,
				checkpoint: { ...checkpoint, unfinished: [checkpoint.unfinished, diagnostic].filter(Boolean).join("\n") },
				candidate: saved.candidate ?? null,
				assessment: { stage: request.stage, candidate: saved.candidate ?? null, findings: text, void: diagnostic },
			}, ctx);
			return { record: voided, executeHere: false, contextReset, text,
				assessment: "void" as const, diagnostic };
		}
		// An assessment's findings are durable, because the conversation that
		// receives them is not: a compaction or a closed session would otherwise
		// leave a record that cannot distinguish a clean review from a lost one.
		const report: AssessmentReport | undefined = assessing
			? { stage: request.stage, candidate: saved.candidate, findings: text }
			: undefined;
		return { record: await save(saved, report), executeHere: false, contextReset, text,
			assessment: assessing ? ("unchanged-checkout" as const) : undefined };
	}

	/** Consume one bounded attempt. A reset or restart never refills a budget. */
	async retry(record: ScopeRecord, key: string, ctx: ExtensionContext) {
		return this.change(record, { action: "consume-retry", key }, ctx);
	}

	/** Commit one completed block, bounded to the areas its plan declares. */
	async commitBlock(record: ScopeRecord, block: number, message: string, areas: string[], ctx: ExtensionContext) {
		if (!Number.isSafeInteger(block) || block < 1) throw new Error("Name the plan's stable block number");
		if (!message.trim() || !areas.length) throw new Error("A block commit requires a message and the block's declared areas");
		return this.change(record, { action: "commit-block", block, message, areas }, ctx);
	}

	/** Move the immutable candidate to ACCEPT. Uncommitted work is refused. */
	async completeBuild(record: ScopeRecord, ctx: ExtensionContext) {
		return this.change(record, { action: "complete-build" }, ctx);
	}

	/** Return within-scope ACCEPT findings to BUILD; the candidate is superseded. */
	async returnToBuild(record: ScopeRecord, feedback: string, ctx: ExtensionContext) {
		if (!feedback.trim()) throw new Error("State the findings BUILD must correct");
		return this.change(record, { action: "return-to-build", feedback }, ctx);
	}

	/**
	 * Accept the candidate. Project policy decides whether human input is
	 * required; acceptance never merges, pushes, releases, or deletes a branch.
	 */
	async accept(record: ScopeRecord, input: HumanInput | undefined, ctx: ExtensionContext) {
		return this.change(record, { action: "accept", input: input ?? null }, ctx);
	}

	/**
	 * End open work on human authority.
	 *
	 * Partial product work stays on its branch: nothing is merged, pushed, or
	 * deleted. The human's reason is retained with the closed record.
	 */
	async abandon(record: ScopeRecord, reason: string, input: HumanInput, ctx: ExtensionContext) {
		if (!reason.trim()) throw new Error("State why this work is abandoned");
		return this.change(record, { action: "abandon", reason, input }, ctx);
	}

	/**
	 * Stop the worker and release its ownership.
	 *
	 * The worker's session identity is retained, so a later controller resumes
	 * that same session rather than starting a second one.
	 */
	async pause(record: ScopeRecord, ctx: ExtensionContext) {
		const outcome = await this.worker?.stop() ?? "already-stopped";
		this.worker = undefined;
		const updated = await this.change(record, { action: "detach-worker" }, ctx);
		return { record: updated, outcome };
	}
}

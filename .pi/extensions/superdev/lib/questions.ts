import { StringEnum } from "@earendil-works/pi-ai";
import { Type } from "typebox";
import type { ReviewFinding } from "./review.ts";

export type ConfirmedAnswer = { answer: string; findingIds: string[]; confirmedAt: string };
export type QuestionState = {
	version: 1;
	workflow: string;
	candidate: string;
	originPhase?: "scope" | "accept";
	status: "active" | "paused" | "submitted" | "superseded";
	findings: ReviewFinding[];
	mechanicalFindings?: ReviewFinding[];
	cycle?: number;
	answers: Record<string, ConfirmedAnswer>;
	reason?: string;
};

export const QUESTION_ENTRY = "superdev-workflow-questions-v1";

export function registerWorkflowQuestions(
	pi: any,
	options: {
		maxBytes: () => number;
		onSubmit: (state: QuestionState, ctx: any) => Promise<void>;
		onPause?: (state: QuestionState, ctx: any) => Promise<void>;
		onResume?: (state: QuestionState, ctx: any) => Promise<void>;
	},
) {
	let state: QuestionState | undefined;
	let restoreIssue: string | undefined;
	let priorTools: string[] | undefined;
	const persist = () => {
		if (!state) return;
		const bytes = Buffer.byteLength(JSON.stringify(state));
		if (bytes > options.maxBytes()) throw new Error(`workflow question state exceeded ${options.maxBytes()} bytes`);
		pi.appendEntry(QUESTION_ENTRY, state);
	};
	const activate = () => {
		if (!priorTools) priorTools = pi.getActiveTools();
		const allowed = new Set(["read", "sokf_search", "sokf_graph", "superdev_review_diff", "superdev_workflow_control", "superdev_workflow_questions"]);
		pi.setActiveTools([...new Set([...priorTools.filter((tool: string) => allowed.has(tool)), "superdev_workflow_questions"])]);
	};
	const deactivate = () => {
		if (priorTools) pi.setActiveTools(priorTools);
		priorTools = undefined;
	};
	const restore = (entries: any[]) => {
		let candidate: unknown;
		for (const entry of entries) if (entry.type === "custom" && entry.customType === QUESTION_ENTRY) candidate = entry.data;
		if (candidate !== undefined) {
			const value = candidate as Partial<QuestionState>;
			const valid = value.version === 1 && typeof value.workflow === "string" && typeof value.candidate === "string"
				&& ["active", "paused", "submitted", "superseded"].includes(String(value.status))
				&& Array.isArray(value.findings) && value.answers !== null && typeof value.answers === "object"
				&& Buffer.byteLength(JSON.stringify(value)) <= options.maxBytes();
			if (valid) state = value as QuestionState;
			else restoreIssue = "saved workflow question state is corrupt or exceeds configured limits";
		}
		if (state?.status === "active" || state?.status === "paused") activate();
		return state;
	};
	const begin = (next: QuestionState) => { state = next; persist(); activate(); };
	const current = () => state;
	const supersede = (reason: string) => {
		if (!state) return;
		state.status = "superseded";
		state.reason = reason;
		persist();
		deactivate();
	};

	pi.registerTool({
		name: "superdev_workflow_questions",
		label: "Superdev workflow questions",
		description: "Inspect, ask, confirm, revise, pause, resume, or submit the active revision-bound workflow decision queue",
		parameters: Type.Object({
			action: StringEnum(["inspect", "ask", "propose-answer", "revise", "submit", "pause", "resume"] as const),
			findingId: Type.Optional(Type.String()),
			findingIds: Type.Optional(Type.Array(Type.String())),
			proposedAnswer: Type.Optional(Type.String()),
			offset: Type.Optional(Type.Number({ minimum: 0 })),
			limit: Type.Optional(Type.Number({ minimum: 1, maximum: 20 })),
		}),
		executionMode: "sequential",
		execute: async (_id: string, input: any, _signal: AbortSignal, _update: unknown, ctx: any) => {
			if (!state || state.status === "superseded") throw new Error("no active workflow question queue");
			if (input.action === "inspect") {
				const offset = Math.max(0, Math.floor(input.offset ?? 0));
				const limit = Math.min(20, Math.max(1, Math.floor(input.limit ?? 20)));
				const page = state.findings.slice(offset, offset + limit);
				return {
					content: [{ type: "text", text: JSON.stringify({
						workflow: state.workflow,
						candidate: state.candidate,
						status: state.status,
						offset,
						total: state.findings.length,
						nextOffset: offset + page.length < state.findings.length ? offset + page.length : undefined,
						findings: page.map((finding) => ({ id: finding.id, summary: finding.summary, dependsOn: finding.dependsOn ?? [], answered: Boolean(state!.answers[finding.id]) })),
					}) }],
					details: { questionState: true },
				};
			}
			if (input.action === "pause") {
				state.status = "paused"; persist();
				await options.onPause?.(state, ctx);
				return { content: [{ type: "text", text: "Workflow questions paused and ownership released. Say ‘resume workflow questions’ to continue." }], details: { paused: true } };
			}
			if (input.action === "resume") {
				await options.onResume?.(state, ctx);
				state.status = "active"; persist(); activate();
				return { content: [{ type: "text", text: "Workflow questions resumed. Inspect the queue and ask the next dependency-eligible finding." }], details: { resumed: true } };
			}
			if (state.status !== "active") throw new Error(`workflow question queue is ${state.status}`);
			if (input.action === "revise") {
				if (!input.findingId || !state.answers[input.findingId]) throw new Error("revise requires one answered finding ID");
				const covered = state.answers[input.findingId].findingIds;
				for (const id of covered) delete state.answers[id];
				persist();
				return { content: [{ type: "text", text: `Reopened ${covered.join(", ")}. Discuss or ask it again.` }], details: { reopened: covered } };
			}
			if (input.action === "ask") {
				const finding = state.findings.find((candidate) => candidate.id === input.findingId);
				if (!finding) throw new Error("ask requires a finding from the active queue");
				const unresolved = (finding.dependsOn ?? []).filter((id) => !state!.answers[id]);
				if (unresolved.length) throw new Error(`finding depends on unresolved answers: ${unresolved.join(", ")}`);
				if (!ctx.hasUI) return { content: [{ type: "text", text: JSON.stringify({ status: "human-input-required", finding }) }], details: { humanInputRequired: true } };
				const choices = [...(finding.choices ?? []).map((choice) => choice.label), "Type another answer", "Back", "Discuss", "Pause workflow questions"];
				const selected = await ctx.ui.select(`${finding.question ?? finding.summary}\nRecommendation: ${finding.recommendation ?? "Discuss before deciding."}`, choices);
				if (!selected || selected === "Discuss") return { content: [{ type: "text", text: `Discuss finding ${finding.id}: ${finding.summary}\n${finding.evidence}\nImpact: ${finding.impact}` }], details: { discuss: finding.id } };
				if (selected === "Back") return { content: [{ type: "text", text: "Choose another dependency-eligible finding or revise an earlier answer." }], details: { back: true } };
				if (selected === "Pause workflow questions") {
					state.status = "paused"; persist();
					await options.onPause?.(state, ctx);
					return { content: [{ type: "text", text: "Workflow questions paused and ownership released." }], details: { paused: true } };
				}
				let answer = selected;
				if (selected === "Type another answer") answer = (await ctx.ui.input("Your answer", finding.question ?? finding.summary))?.trim();
				if (!answer) throw new Error("answer was empty");
				const confirmed: ConfirmedAnswer = { answer, findingIds: [finding.id], confirmedAt: new Date().toISOString() };
				state.answers[finding.id] = confirmed; persist();
				return { content: [{ type: "text", text: `Confirmed provisional answer for ${finding.id}: ${answer}. Choose the next question dynamically.` }], details: { answered: [finding.id] } };
			}
			if (input.action === "propose-answer") {
				const ids = input.findingIds?.length ? input.findingIds : input.findingId ? [input.findingId] : [];
				if (!ids.length || !input.proposedAnswer?.trim()) throw new Error("propose-answer requires finding IDs and a proposed answer");
				const findings = ids.map((id: string) => state!.findings.find((finding) => finding.id === id));
				if (findings.some((finding: ReviewFinding | undefined) => !finding)) throw new Error("proposed answer names an unknown finding");
				const impact = findings.map((finding: ReviewFinding) => `${finding.id}: ${finding.summary}`).join("\n");
				if (!ctx.hasUI || !(await ctx.ui.confirm("Confirm proposed workflow answer?", `${input.proposedAnswer}\n\nCovers:\n${impact}`))) {
					return { content: [{ type: "text", text: "Proposed answer was not confirmed; continue discussion." }], details: { confirmed: false } };
				}
				const confirmed: ConfirmedAnswer = { answer: input.proposedAnswer.trim(), findingIds: ids, confirmedAt: new Date().toISOString() };
				for (const id of ids) state.answers[id] = confirmed;
				persist();
				return { content: [{ type: "text", text: `Confirmed provisional answer covering ${ids.join(", ")}.` }], details: { confirmed: true, answered: ids } };
			}
			const unanswered = state.findings.filter((finding) => !state!.answers[finding.id]);
			if (unanswered.length) throw new Error(`cannot submit; unanswered findings: ${unanswered.map((finding) => finding.id).join(", ")}`);
			const summary = Object.values(state.answers).filter((answer, index, all) => all.indexOf(answer) === index)
				.map((answer) => `${answer.findingIds.join(", ")}: ${answer.answer}`).join("\n");
			if (!ctx.hasUI || !(await ctx.ui.confirm("Submit all workflow answers?", summary))) {
				return { content: [{ type: "text", text: "Answers remain provisional and editable." }], details: { submitted: false } };
			}
			state.status = "submitted"; persist(); deactivate();
			await options.onSubmit(state, ctx);
			return { content: [{ type: "text", text: "Submitted the complete confirmed answer set for one batched correction." }], details: { submitted: true } };
		},
	});
	return { begin, current, restore, restoreIssue: () => restoreIssue, activate, deactivate, supersede };
}

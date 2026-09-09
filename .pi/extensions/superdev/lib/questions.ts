import { StringEnum } from "@earendil-works/pi-ai";
import { Type } from "typebox";
import type { ReviewFinding } from "./review.ts";

export type ConfirmedAnswer = { answer: string; findingIds: string[]; confirmedAt: string };
export type QuestionState = {
	version: 1;
	workflow: string;
	candidate: string;
	originPhase?: "scope" | "build" | "accept";
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
		maxFindings: () => number;
		onSubmit: (state: QuestionState, ctx: any) => Promise<void>;
		onPause?: (state: QuestionState, ctx: any) => Promise<void>;
		onResume?: (state: QuestionState, ctx: any) => Promise<void>;
		exposeTool?: boolean;
	},
) {
	let state: QuestionState | undefined;
	let restoreIssue: string | undefined;
	let priorTools: string[] | undefined;
	const stateIssue = (value: Partial<QuestionState>): string | undefined => {
		if (value.version !== 1 || typeof value.workflow !== "string" || typeof value.candidate !== "string"
			|| !["active", "paused", "submitted", "superseded"].includes(String(value.status))
			|| !Array.isArray(value.findings) || value.findings.length > options.maxFindings()
			|| value.answers === null || typeof value.answers !== "object") return "saved workflow question state is corrupt or exceeds configured limits";
		const ids = new Set<string>();
		for (const finding of value.findings) {
			if (!finding || typeof finding !== "object" || typeof finding.id !== "string" || !finding.id.trim()
				|| ids.has(finding.id) || typeof finding.summary !== "string" || typeof finding.evidence !== "string"
				|| typeof finding.impact !== "string" || !Array.isArray(finding.dependsOn ?? [])) {
				return "saved workflow question state contains malformed findings";
			}
			ids.add(finding.id);
		}
		const byId = new Map(value.findings.map((finding) => [finding.id, finding]));
		const visiting = new Set<string>();
		const visited = new Set<string>();
		const visit = (id: string): boolean => {
			if (visiting.has(id)) return false;
			if (visited.has(id)) return true;
			visiting.add(id);
			for (const dependency of byId.get(id)?.dependsOn ?? []) {
				if (!ids.has(dependency) || dependency === id || !visit(dependency)) return false;
			}
			visiting.delete(id);
			visited.add(id);
			return true;
		};
		for (const id of ids) if (!visit(id)) return "saved workflow question state contains invalid or cyclic dependencies";
		for (const [id, answer] of Object.entries(value.answers ?? {})) {
			if (!ids.has(id) || !answer || typeof answer.answer !== "string" || !answer.answer.trim()
				|| !Array.isArray(answer.findingIds) || !answer.findingIds.includes(id)
				|| answer.findingIds.some((covered) => !ids.has(covered))) return "saved workflow question state contains malformed answers";
			for (const covered of answer.findingIds) {
				const peer = value.answers?.[covered];
				if (!peer || peer.answer !== answer.answer || peer.findingIds.join("\0") !== answer.findingIds.join("\0")) {
					return "saved workflow question state contains inconsistent multi-finding answers";
				}
			}
		}
		if (Buffer.byteLength(JSON.stringify(value)) > options.maxBytes()) return "saved workflow question state is corrupt or exceeds configured limits";
	};
	const persist = () => {
		if (!state) return;
		const issue = stateIssue(state);
		if (issue) throw new Error(issue);
		pi.appendEntry(QUESTION_ENTRY, state);
	};
	const activate = () => {
		if (!priorTools) priorTools = pi.getActiveTools();
		const allowed = new Set(["read", "sokf_search", "sokf_graph", "superdev_run_phase", "superdev_ask", ...(options.exposeTool === false ? [] : ["superdev_workflow_questions"])]);
		pi.setActiveTools([...new Set([...priorTools.filter((tool: string) => allowed.has(tool)), ...(options.exposeTool === false ? [] : ["superdev_workflow_questions"])])]);
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
			restoreIssue = stateIssue(value);
			if (!restoreIssue) state = value as QuestionState;
		}
		if (state?.status === "active") {
			state.status = "paused";
			state.reason = "restored after session restart; explicit resume required";
			persist();
		}
		return state;
	};
	const begin = (next: QuestionState) => { state = next; persist(); activate(); };
	const current = () => state;
	const resume = async (ctx: any) => {
		if (!state || state.status !== "paused") throw new Error("no paused workflow question queue is available");
		await options.onResume?.(state, ctx);
		state.status = "active"; persist(); activate();
	};
	const supersede = (reason: string) => {
		if (!state) return;
		state.status = "superseded";
		state.reason = reason;
		persist();
		deactivate();
	};

	const snapshot = (offsetInput = 0, limitInput = 20) => {
		if (!state || state.status === "superseded") return undefined;
		const offset = Math.max(0, Math.floor(offsetInput));
		const limit = Math.min(20, Math.max(1, Math.floor(limitInput)));
		const page = state.findings.slice(offset, offset + limit);
		return {
			workflow: state.workflow,
			candidate: state.candidate,
			phase: state.originPhase ?? "scope",
			status: state.status,
			offset,
			total: state.findings.length,
			nextOffset: offset + page.length < state.findings.length ? offset + page.length : undefined,
			findings: page.map((finding) => ({ ...finding, dependsOn: finding.dependsOn ?? [], answered: Boolean(state!.answers[finding.id]) })),
		};
	};

	const questionTool = {
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
				return {
					content: [{ type: "text", text: JSON.stringify(snapshot(input.offset, input.limit)) }],
					details: { questionState: true },
				};
			}
			if (input.action === "pause") {
				state.status = "paused"; persist();
				await options.onPause?.(state, ctx);
				deactivate();
				return { content: [{ type: "text", text: "Workflow questions paused and ownership released. Say ‘resume workflow questions’ to continue." }], details: { paused: true } };
			}
			if (input.action === "resume") {
				await resume(ctx);
				return { content: [{ type: "text", text: "Workflow questions resumed. Inspect the queue and ask the next dependency-eligible finding." }], details: { resumed: true } };
			}
			if (state.status !== "active") throw new Error(`workflow question queue is ${state.status}`);
			if (input.action === "revise") {
				if (!input.findingId || !state.answers[input.findingId]) throw new Error("revise requires one answered finding ID");
				const affected = new Set(state.answers[input.findingId].findingIds);
                let changed = true;
                while (changed) {
                    changed = false;
                    for (const finding of state.findings) {
                        if ((finding.dependsOn ?? []).some(id => affected.has(id)) && !affected.has(finding.id)) {
                            affected.add(finding.id);
                            for (const id of state.answers[finding.id]?.findingIds ?? []) affected.add(id);
                            changed = true;
                        }
                    }
                }
                const covered = [...affected];
				for (const id of covered) delete state.answers[id];
				persist();
				return { content: [{ type: "text", text: `Reopened ${covered.join(", ")}. Discuss or ask it again.` }], details: { reopened: covered } };
			}
			if (input.action === "ask") {
				const finding = state.findings.find((candidate) => candidate.id === input.findingId);
				if (!finding) throw new Error("ask requires a finding from the active queue");
				if (state.answers[finding.id]) throw new Error("answered findings must be reopened with revise before asking again");
				const unresolved = (finding.dependsOn ?? []).filter((id) => !state!.answers[id]);
				if (unresolved.length) throw new Error(`finding depends on unresolved answers: ${unresolved.join(", ")}`);
				if (!ctx.hasUI) {
					state.status = "paused"; persist();
					await options.onPause?.(state, ctx);
					deactivate();
					return { content: [{ type: "text", text: JSON.stringify({ status: "human-input-required", workflow: state.workflow, phase: state.originPhase ?? "scope", pendingAction: `answer ${finding.id}`, resume: "Resume workflow questions in an interactive session" }) }], details: { humanInputRequired: true } };
				}
				const choices = [...(finding.choices ?? []).map((choice) => choice.label), "Type another answer", "Back", "Discuss", "Pause workflow questions"];
				const selected = await ctx.ui.select(`${finding.question ?? finding.summary}\nRecommendation: ${finding.recommendation ?? "Discuss before deciding."}`, choices);
				if (!selected || selected === "Discuss") return { content: [{ type: "text", text: `Discuss finding ${finding.id}: ${finding.summary}\n${finding.evidence}\nImpact: ${finding.impact}` }], details: { discuss: finding.id } };
				if (selected === "Back") return { content: [{ type: "text", text: "Choose another dependency-eligible finding or revise an earlier answer." }], details: { back: true } };
				if (selected === "Pause workflow questions") {
					state.status = "paused"; persist();
					await options.onPause?.(state, ctx);
					deactivate();
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
				const unresolved = findings.flatMap((finding: ReviewFinding) => (finding.dependsOn ?? []).filter(id => !state!.answers[id] && !ids.includes(id)));
                if (unresolved.length) throw new Error(`finding depends on unresolved answers: ${unresolved.join(", ")}`);
                const impact = findings.map((finding: ReviewFinding) => `${finding.id}: ${finding.summary}`).join("\n");
				if (!ctx.hasUI) {
					state.status = "paused"; persist();
					await options.onPause?.(state, ctx);
					deactivate();
					return { content: [{ type: "text", text: JSON.stringify({ status: "human-input-required", workflow: state.workflow, phase: state.originPhase ?? "scope", pendingAction: `confirm answer for ${ids.join(", ")}`, resume: "Resume workflow questions in an interactive session" }) }], details: { humanInputRequired: true, confirmed: false } };
				}
				if (!(await ctx.ui.confirm("Confirm proposed workflow answer?", `${input.proposedAnswer}\n\nCovers:\n${impact}`))) {
					return { content: [{ type: "text", text: "Proposed answer was not confirmed; continue discussion." }], details: { confirmed: false } };
				}
				const replacedGroups = new Set(ids.flatMap((id: string) => state!.answers[id]?.findingIds ?? []));
				for (const [id, answer] of Object.entries(state.answers)) {
					if (answer.findingIds.some((covered) => ids.includes(covered)) || replacedGroups.has(id)) delete state.answers[id];
				}
				const confirmed: ConfirmedAnswer = { answer: input.proposedAnswer.trim(), findingIds: ids, confirmedAt: new Date().toISOString() };
				for (const id of ids) state.answers[id] = confirmed;
				persist();
				return { content: [{ type: "text", text: `Confirmed provisional answer covering ${ids.join(", ")}.` }], details: { confirmed: true, answered: ids } };
			}
			const unanswered = state.findings.filter((finding) => !state!.answers[finding.id]);
			if (unanswered.length) throw new Error(`cannot submit; unanswered findings: ${unanswered.map((finding) => finding.id).join(", ")}`);
			const groups = new Map<string, ConfirmedAnswer>();
			for (const answer of Object.values(state.answers)) groups.set(`${answer.findingIds.join("\0")}\0${answer.answer}`, answer);
			const summary = [...groups.values()].map((answer) => `${answer.findingIds.join(", ")}: ${answer.answer}`).join("\n");
			if (!ctx.hasUI) {
				state.status = "paused"; persist();
				await options.onPause?.(state, ctx);
				deactivate();
				return { content: [{ type: "text", text: JSON.stringify({ status: "human-input-required", workflow: state.workflow, phase: state.originPhase ?? "scope", pendingAction: "submit all confirmed answers", resume: "Resume workflow questions in an interactive session" }) }], details: { humanInputRequired: true, submitted: false } };
			}
			if (!(await ctx.ui.confirm("Submit all workflow answers?", summary))) {
				return { content: [{ type: "text", text: "Answers remain provisional and editable." }], details: { submitted: false } };
			}
			state.status = "submitted"; persist(); deactivate();
			try {
				await options.onSubmit(state, ctx);
			} catch (error) {
				state.status = "paused";
				state.reason = `continuation failed: ${String(error)}`;
				persist();
				throw error;
			}
			return { content: [{ type: "text", text: "Submitted the complete confirmed answer set for one batched correction." }], details: { submitted: true } };
		},
	};
	if (options.exposeTool !== false) pi.registerTool(questionTool);
	const operate = (input: any, ctx: any) => questionTool.execute("internal", input, new AbortController().signal, undefined, ctx);
	return { begin, current, snapshot, resume, restore, restoreIssue: () => restoreIssue, activate, deactivate, supersede, operate };
}

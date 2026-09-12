import { randomUUID } from "node:crypto";
import type { ExtensionContext, InputEvent } from "@earendil-works/pi-coding-agent";

export type Choice = { id: string; label: string };
export type Question = {
	question: string;
	choices: Choice[];
	recommendation: { choiceId: string; reason: string };
};
export type HumanInput = { session: string; event: string; text: string };
export type Reply =
	| { status: "answered"; answer: string; choiceId?: string; input: HumanInput }
	| { status: "discuss" | "other-action" | "paused"; text?: string };
export type ApprovalTarget = { kind: "issue" | "plan"; id: string; hash: string };
type Pending = {
	id: string;
	question: Question;
	target?: ApprovalTarget;
	respond: (reply: Reply, ctx: ExtensionContext) => Promise<unknown>;
};

const controls = ["Type an answer", "Discuss", "Do something else", "Pause"];
const normalise = (text: string) => text.trim().toLowerCase().replace(/[.!]$/, "");

/** Validate identities before adding presentation markers or reordering choices. */
export function validateQuestion(question: Question): void {
	if (!question.question.trim() || question.choices.length < 1 || question.choices.length > 8) {
		throw new Error("A question requires text and one to eight choices");
	}
	const ids = new Set<string>();
	const labels = new Set<string>();
	for (const choice of question.choices) {
		const label = normalise(choice.label);
		if (!/^[a-z][a-z0-9-]*$/.test(choice.id) || ["discuss", "pause", "cancel", "other-action"].includes(choice.id) || ids.has(choice.id) || !label || labels.has(label)
			|| controls.some((control) => normalise(control) === label) || choice.label.includes("[Recommended]")) {
			throw new Error("Choice IDs and labels must be unique and must not contain control or presentation markers");
		}
		ids.add(choice.id); labels.add(label);
	}
	if (!ids.has(question.recommendation.choiceId) || !question.recommendation.reason.trim()) {
		throw new Error("The recommendation must name a supplied choice and explain it");
	}
}

/** A single transient question. It is not an approval ledger or a saved permission grant. */
export class HumanQuestions {
	private pending?: Pending;
	constructor(private readonly report: (value: unknown, ctx: ExtensionContext) => void) {}

	view() {
		if (!this.pending) return undefined;
		return { id: this.pending.id, ...this.pending.question, target: this.pending.target };
	}

	clear() { this.pending = undefined; }

	async ask(question: Question, ctx: ExtensionContext,
		respond: Pending["respond"], target?: ApprovalTarget): Promise<unknown> {
		validateQuestion(question);
		if (this.pending) throw new Error("A question is still pending; answer it, discuss it, choose another action, or pause");
		const pending: Pending = { id: randomUUID(), question, respond, target };
		this.pending = pending;
		if (!ctx.hasUI || ctx.mode !== "tui") {
			return { status: "human-input-required", pending: this.view() };
		}
		const ordered = [...question.choices].sort((a, b) => Number(b.id === question.recommendation.choiceId) - Number(a.id === question.recommendation.choiceId));
		const labels = ordered.map((choice) => `${choice.label}${choice.id === question.recommendation.choiceId ? " [Recommended]" : ""}`);
		const selected = await ctx.ui.select(`${question.question}\n${question.recommendation.reason}`, [...labels, ...controls]);
		if (this.pending !== pending) return { status: "question-already-resolved" };
		if (!selected || selected === "Pause") return this.finish(pending, { status: "paused" }, ctx);
		if (selected === "Discuss") return this.finish(pending, { status: "discuss" }, ctx);
		if (selected === "Do something else") return this.finish(pending, { status: "other-action" }, ctx);
		if (selected === "Type an answer") {
			const text = await ctx.ui.input(question.question);
			if (!text?.trim()) return this.finish(pending, { status: "paused" }, ctx);
			const reply = this.parse(text, pending, ctx, true);
			return reply ? this.finish(pending, reply, ctx) : this.finish(pending, { status: "discuss", text }, ctx);
		}
		const choice = ordered[labels.indexOf(selected)];
		if (!choice) throw new Error("The UI returned a choice outside the pending question");
		return this.finish(pending, this.answer(choice.label, ctx, choice.id), ctx);
	}

	/** Pi's existing interactive path (including earlier transformers) is trusted.
	 * Extension/worker input, RPC, and model arguments cannot call this path. */
	async onInput(event: InputEvent, ctx: ExtensionContext) {
		const pending = this.pending;
		if (!pending || event.source !== "interactive" || ctx.mode !== "tui") return { action: "continue" as const };
		const reply = this.parse(event.text, pending, ctx, false);
		if (!reply) return { action: "continue" as const }; // Ordinary discussion leaves the question intact.
		try { this.report(await this.finish(pending, reply, ctx), ctx); }
		catch (error) { this.report({ status: "failed", diagnostic: String(error), partialWorkPreserved: true }, ctx); }
		return { action: "handled" as const };
	}

	private answer(text: string, ctx: ExtensionContext, choiceId?: string): Reply {
		return { status: "answered", answer: text, choiceId,
			input: { session: ctx.sessionManager.getSessionId(), event: randomUUID(), text } };
	}

	private parse(text: string, pending: Pending, ctx: ExtensionContext, typed: boolean): Reply | undefined {
		const value = normalise(text);
		if (value === "pause" || value === "cancel") return { status: "paused" };
		if (value === "discuss") return { status: "discuss" };
		if (value === "do something else") return { status: "other-action" };
		if (pending.target) {
			const target = pending.target;
			const phrases = ["approve", "i approve", `approve ${target.id}`, `i approve ${target.id}`,
				`approve the ${target.kind}`, `i approve the ${target.kind}`, `approve this ${target.kind}`, `i approve this ${target.kind}`];
			return phrases.includes(value) ? this.answer(text, ctx, "approve") : undefined;
		}
		const choice = pending.question.choices.find((choice) => value === normalise(choice.id) || value === normalise(choice.label));
		if (choice) return this.answer(text, ctx, choice.id);
		if (typed) return this.answer(text.trim(), ctx);
		if (/^answer:\s*\S/i.test(text)) return this.answer(text.replace(/^answer:\s*/i, "").trim(), ctx);
		return undefined;
	}

	private async finish(pending: Pending, reply: Reply, ctx: ExtensionContext): Promise<unknown> {
		if (this.pending !== pending) return { status: "question-already-resolved" };
		// Consume before any asynchronous write. A UI reply and a chat reply
		// cannot authorise the same action twice. Discussion retains the question.
		if (reply.status !== "discuss") this.pending = undefined;
		return pending.respond(reply, ctx);
	}
}

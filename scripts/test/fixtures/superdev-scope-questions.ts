import assert from "node:assert/strict";
import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import { HumanQuestions, validateQuestion, type Reply } from "../../../.pi/extensions/superdev/lib/questions.ts";

const question = { question: "Which format?", choices: [{ id: "json", label: "JSON" }, { id: "text", label: "Plain text" }],
	recommendation: { choiceId: "text", reason: "It fits the stated reader." } };
const replies: Reply[] = [], reports: unknown[] = [];
let labels: string[] = [], selected: string | undefined = "Plain text [Recommended]";
let dialogs = 0;
const ctx = { cwd: process.cwd(), mode: "tui", hasUI: true,
	sessionManager: { getSessionId: () => "human-session" },
	ui: { async select(_title: string, options: string[]) { dialogs++; labels = options; return selected; },
		async input() { return "A typed answer"; }, confirm() { throw new Error("A second confirmation is forbidden"); } },
} as unknown as ExtensionContext;
const gate = new HumanQuestions((value) => reports.push(value));
const receive = async (reply: Reply) => { replies.push(reply); return reply; };
await gate.ask(question, ctx, receive);
assert.equal(labels[0], "Plain text [Recommended]");
assert.deepEqual(labels.slice(-3), ["Discuss", "Do something else", "Pause"]);
assert.equal(replies[0].status, "answered");
if (replies[0].status === "answered") {
	assert.equal(replies[0].choiceId, "text"); assert.equal(replies[0].answer, "Plain text");
	assert.equal(replies[0].input.text, "Plain text");
}
selected = "Discuss";
await gate.ask(question, ctx, receive);
const pending = gate.view();
assert.equal(replies.at(-1)?.status, "discuss"); assert.ok(pending);
assert.deepEqual(await gate.onInput({ type: "input", source: "interactive", text: "Can we discuss alternatives?" }, ctx), { action: "continue" });
assert.deepEqual(gate.view(), pending);
assert.equal(dialogs, 2, "discussion reopened the dialog");
await gate.onInput({ type: "input", source: "interactive", text: "answer: Another format" }, ctx);
assert.equal((replies.at(-1) as Extract<Reply, {status:"answered"}>).answer, "Another format");
assert.equal(gate.view(), undefined);
selected = "Do something else";
await gate.ask(question, ctx, receive);
assert.equal(replies.at(-1)?.status, "other-action");
selected = undefined;
await gate.ask(question, ctx, receive);
assert.equal(replies.at(-1)?.status, "paused");
selected = "Type an answer";
await gate.ask(question, ctx, receive);
assert.equal((replies.at(-1) as Extract<Reply, {status:"answered"}>).answer, "A typed answer");
assert.throws(() => validateQuestion({ ...question, recommendation: { choiceId: "missing", reason: "No such choice" } }));
assert.throws(() => validateQuestion({ ...question, choices: [{ id: "same", label: "One" }, { id: "same", label: "Two" }] }));
assert.throws(() => validateQuestion({ ...question, choices: [{ id: "json", label: "Discuss" }] }));
console.log("SUPERDEV_QUESTIONS_PASS");

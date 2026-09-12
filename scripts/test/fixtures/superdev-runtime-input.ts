// Probe Pi's input provenance and context capabilities, not the product authority service.
import assert from "node:assert/strict";
import { Type } from "typebox";
import type { ExtensionContext, InputEvent } from "@earendil-works/pi-coding-agent";
import { proofSession } from "./superdev-runtime-session.ts";

const [cwd, scenario] = process.argv.slice(2);
assert.ok(cwd);
type Action = { name: "approve-issue" | "approve-plan"; document: string; revision: string };
let pending: Action | undefined;
let authorised: { action: Action; input: string } | undefined;
const seen: InputEvent[] = [];
let eventContext: ExtensionContext | undefined;
let toolContext: ExtensionContext | undefined;
let toolResult: boolean | undefined;
const attempt: Action = { name: "approve-issue", document: "issue-001", revision: "v1" };

// Minimal proof matcher, not the complete conversational parser. No model boolean.
const phrase = (action: Action) => `I approve ${action.document}`;
function take(action: Action) {
	const match = authorised && JSON.stringify(authorised.action) === JSON.stringify(action);
	authorised = undefined;
	return Boolean(match);
}
const { session, requests, errors } = await proofSession({
	cwd,
	// A separately loaded extension, not a model call or a fabricated event.
	beforeFactory: scenario === "transformed" ? (pi) => {
		pi.on("input", (event) => {
			if (event.text === "Discuss this issue") return { action: "transform", text: phrase(attempt) };
		});
	} : undefined,
	factory(pi) {
		pi.on("input", (event, ctx) => {
			seen.push(structuredClone(event));
			eventContext = ctx;
			if (event.source !== "interactive" || ctx.mode !== "tui") return;
			if (!pending || event.text !== phrase(pending)) return;
			authorised = { action: structuredClone(pending), input: event.text };
			pending = undefined;
			return { action: "handled" };
		});
		pi.registerTool({ name: "proof_approve", label: "Approval probe", description: "Try a model-generated approval",
			parameters: Type.Object({ approved: Type.Boolean() }),
			execute(_id, _input, _signal, _update, ctx) {
				toolContext = ctx;
				toolResult = take(attempt);
				return { content: [{ type: "text", text: String(toolResult) }], details: {}, terminate: true };
			},
		});
	},
	reply: () => [{ type: "toolCall", id: "model-approval", name: "proof_approve", arguments: { approved: true } }],
});
try {
	if (scenario === "transformed") {
		pending = attempt;
		await session.prompt("Discuss this issue", { source: "interactive" });
		const observed = seen.at(-1)!;
		assert.equal(observed.source, "interactive");
		assert.equal(observed.text, phrase(attempt));
		// The human accepted this trust boundary: earlier interactive-input
		// transformers are responsible for preserving approval intent.
		assert.equal(take(attempt), true);
		assert.equal(take(attempt), false, "transformed approval was reusable");
		pending = attempt;
		await session.sendUserMessage("Discuss this issue");
		assert.equal(seen.at(-1)?.source, "extension");
		assert.equal(seen.at(-1)?.text, phrase(attempt));
		assert.equal(toolResult, false, "a transform authorised extension input");
		await session.prompt("Discuss this issue", { source: "rpc" });
		assert.equal(seen.at(-1)?.source, "rpc");
		assert.equal(toolResult, false, "a transform authorised RPC input");
		assert.deepEqual(pending, attempt);
		await session.prompt("Discuss this issue", { source: "interactive" });
		assert.equal(take({ ...attempt, revision: "v2" }), false, "transformed approval transferred to changed bytes");
		assert.deepEqual(errors, []);
		console.log(JSON.stringify({ trustBoundary: "interactive-input-transformers", nonHumanInput: "rejected",
			submitted: "Discuss this issue", observed }));
	} else await directInput();
} finally { session.dispose(); }

async function directInput() {
	const issue: Action = { name: "approve-issue", document: "issue-001", revision: "v1" };
	const plan: Action = { name: "approve-plan", document: "plan-001", revision: "v1" };
	pending = issue;
	// Exercise the public extension-message route; it must not become typed input.
	await session.sendUserMessage(phrase(issue));
	assert.equal(seen.at(-1)?.source, "extension");
	assert.equal(toolResult, false);
	assert.equal(authorised, undefined);
	assert.deepEqual(pending, issue);
	await session.prompt(phrase(issue), { source: "rpc" });
	assert.equal(seen.at(-1)?.source, "rpc");
	assert.equal(toolResult, false);
	for (const discussion of ["Yes", "Discuss this issue"]) {
		await session.prompt(discussion);
		assert.equal(authorised, undefined, "discussion or ambiguous agreement became approval");
		assert.deepEqual(pending, issue);
	}

	// This is the same SDK input entry point as interactive mode, not a stubbed hook.
	const before = requests.length;
	await session.prompt(phrase(issue), { source: "interactive" });
	assert.equal(seen.at(-1)?.source, "interactive");
	assert.equal(requests.length, before, "an approval needs neither a model nor a second dialog");
	assert.ok(authorised);
	assert.equal(take(plan), false, "issue approval transferred to a plan");
	assert.equal(take(issue), false, "a mismatched/replayed reply was reusable");

	pending = issue;
	await session.prompt(phrase(issue));
	assert.equal(take({ ...issue, revision: "v2" }), false, "approval transferred to changed bytes");
	pending = issue;
	await session.prompt(phrase(issue));
	assert.equal(take(issue), true);
	assert.equal(take(issue), false);
	pending = plan;
	await session.prompt("I approved issue-001 earlier");
	assert.equal(authorised, undefined);
	assert.deepEqual(pending, plan);

	assert.ok(eventContext && toolContext);
	// The pinned public context does not expose tool-driven tree navigation.
	assert.equal("navigateTree" in eventContext, false);
	assert.equal("navigateTree" in toolContext, false);
	assert.equal(typeof toolContext.compact, "function");
	assert.deepEqual(errors, []);
	console.log(JSON.stringify({ inputProvenance: "passed", currentSessionReset: "command-only; tool fallback required" }));
}

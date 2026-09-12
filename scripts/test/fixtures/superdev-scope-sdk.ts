// The production adapter, real Pi input/tool dispatch, and real Rust/Git publication.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { AssistantMessage } from "@earendil-works/pi-ai";
import superdev from "../../../.pi/extensions/superdev/index.ts";
import { proofSession } from "./superdev-runtime-session.ts";

const [root] = process.argv.slice(2);
const issue = "issue-001-human-led", plan = "plan-042-independent";
const issuePath = `knowledge/issues/open/${issue}.md`, planPath = `knowledge/plans/open/${plan}.md`;
const issueText = `---\ntype: Issue\nid: ${issue}\ntitle: Human-led scope\ndescription: Test scope.\nlifecycle: open\n---\n\n# Issue\n\nThe intended outcome.\n`;
const planText = `---\ntype: Plan\nid: ${plan}\ntitle: Independent plan\ndescription: Test plan.\nlifecycle: open\nlinks:\n  - rel: implements\n    to: ${issue}\n---\n\n# Plan\n\nImplement the intended outcome.\n`;
function git(...args: string[]) { return execFileSync("git", args, { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim(); }
git("init", "-q", "-b", "main"); git("config", "user.name", "Test"); git("config", "user.email", "test@example.com"); git("config", "commit.gpgsign", "false");
await writeFile(join(root, ".gitignore"), ".superdev/cache/\n.superdev/workflows/\nagent/\nsessions/\n");
git("add", ".gitignore"); git("commit", "-qm", "init");
let next: { name: string; arguments: Record<string, unknown> } | undefined;
let sequence = 0;
async function start() {
	return proofSession({ cwd: root, factory: superdev,
		beforeFactory(pi) { pi.on("input", (event) => event.text === "Trusted transformer reply" ? { action: "transform", text: "I approve the issue" } : undefined); },
		reply: (): AssistantMessage["content"] => {
			if (!next) return [{ type: "text", text: "Scripted discussion." }];
			const tool = next; next = undefined;
			return [{ type: "toolCall", id: `scope-tool-${sequence++}`, ...tool }];
		},
	});
}
let runtime = await start();
async function invokeScope(request: string) {
	// Extension sendUserMessage schedules a turn; command completion alone does
	// not mean that scheduled turn has started or settled.
	let unsubscribe = () => {};
	const ended = new Promise<void>((done) => {
		unsubscribe = runtime.session.subscribe((event) => { if (event.type === "agent_end") done(); });
	});
	try {
		await runtime.session.prompt(`/superdev ${request}`, { source: "interactive" });
		await ended;
		await runtime.session.waitForIdle();
	} finally { unsubscribe(); }
}
async function tool(name: string, args: Record<string, unknown>, error = false) {
	next = { name, arguments: args };
	await runtime.session.prompt("Run the next scripted tool", { source: "extension" });
	const result = runtime.session.messages.findLast((message) => message.role === "toolResult");
	assert.ok(result && result.role === "toolResult");
	assert.equal(Boolean(result.isError), error, JSON.stringify(result));
	return result;
}
async function phase(action: string, extra: Record<string, unknown> = {}, error = false) {
	return tool("superdev_run_phase", { phase: "scope", action, ...extra }, error);
}
async function record() {
	const status = JSON.parse(execFileSync("superdev", ["workflow", "status", "--json"], { cwd: root, encoding: "utf8" }));
	return status.result.workflows[0]?.record;
}
async function human(text: string) {
	const requests = runtime.requests.length;
	await runtime.session.prompt(text, { source: "interactive" });
	assert.equal(runtime.requests.length, requests, "a resolved human action started a model turn or a second confirmation");
}
try {
	await invokeScope("Scope the fixture");
	git("switch", "-qc", "unrelated");
	const wrongBranch = await phase("run", { issue });
	assert.equal((wrongBranch.details as { status: string }).status, "branch-switch-required");
	assert.equal(await record(), undefined);
	await tool("write", { path: issuePath, content: issueText }, true);
	git("switch", "main");
	await phase("run", { issue });
	await runtime.session.sendUserMessage("continue");
	await runtime.session.prompt("continue", { source: "rpc" });
	assert.equal(await record(), undefined, "non-human input reserved a workflow");
	await human("continue");
	assert.equal((await record()).plan, undefined);
	await tool("write", { path: issuePath, content: issueText });
	await phase("record-step", { step: "select-issue", note: "Draft saved; not approved" });
	await tool("write", { path: issuePath, content: "unpermitted" }, true);
	assert.equal(await readFile(join(root, issuePath), "utf8"), issueText);
	await phase("continue", { steps: ["interview-issue"] }); await human("skip");
	assert.equal((await record()).steps.at(-1).outcome, "skipped");
	for (const round of [1, 2]) {
		await phase("continue", { steps: ["interview-issue"] }); await human("continue");
		await tool("superdev_ask", { question: `Interview round ${round}: how much?`,
			choices: [{ id: "small", label: "Keep it small" }, { id: "large", label: "Include more" }],
			recommendation: { choiceId: "small", reason: "Stay within the requested boundary." } });
		if (round === 1) {
			await human("discuss");
			await runtime.session.prompt("What does small exclude?", { source: "interactive" });
		}
		await human("answer: Keep the requested boundary");
		await phase("record-step", { step: "interview-issue", note: `Interview ${round}: keep the requested boundary` });
	}
	assert.equal((await record()).steps.filter((entry: any) => entry.step === "interview-issue" && entry.outcome === "completed").length, 2);
	await phase("continue", { steps: ["write-issue", "check-issue"] }); await human("continue");
	await tool("write", { path: issuePath, content: issueText });
	await phase("record-step", { step: "write-issue", note: "Issue written" });
	// Pause in the middle of a permitted group: its unused permission must end.
	await phase("record-discussion", { note: "Retain the open policy question across pause" });
	await phase("pause");
	assert.match((await record()).discussion, /Retain the open policy question/);
	await phase("resume"); await human("continue");
	await phase("record-step", { step: "check-issue", note: "Not yet permitted after pause" }, true);
	await phase("continue", { steps: ["check-issue"] }); await human("continue");
	await phase("record-step", { step: "check-issue", note: "One open finding remains for the human" });
	await phase("continue", { steps: ["check-issue"] }); await human("continue");
	await phase("record-step", { step: "check-issue", note: "Repeated review preserves the open finding" });
	await phase("approve-issue");
	await runtime.session.prompt("Looks good", { source: "interactive" });
	assert.equal((await record()).issue_approval, undefined);
	await phase("approve-issue", { approved: true }, true);
	await runtime.session.sendUserMessage("Trusted transformer reply");
	await runtime.session.prompt("Trusted transformer reply", { source: "rpc" });
	assert.equal((await record()).issue_approval, undefined);
	await writeFile(join(root, issuePath), `${issueText}\nChanged while the approval question was open.\n`);
	await human("I approve the issue");
	assert.equal((await record()).issue_approval, undefined, "stale bytes were approved");
	await phase("approve-issue");
	await human("Trusted transformer reply");
	assert.ok((await record()).issue_approval);
	assert.equal(git("branch", "--list", "work/*"), "");
	assert.equal(git("log", "--format=%s").split("\n").length, 2);
	await runtime.session.prompt("I approve the issue", { source: "interactive" });
	assert.equal(git("log", "--format=%s").split("\n").length, 2, "approval replay published twice");
	await phase("continue", { steps: ["write-plan"] }); await human("continue");
	await phase("attach-plan", { plan });
	await tool("write", { path: planPath, content: planText });
	await phase("record-step", { step: "write-plan", note: "Initial plan saved" });
	await phase("approve-plan"); await human(`I approve ${plan}`);
	const approved = await record();
	assert.equal([...approved.discussion.matchAll(/"answer":/g)].length, 2, "a later question erased an earlier answer");
	assert.match(approved.discussion, /Retain the open policy question/);
	assert.equal(approved.phase, "scope");
	assert.equal(approved.work_branch, undefined);
	assert.equal(approved.scope_step, "handoff");
	assert.ok(approved.steps.some((entry: any) => entry.step === "check-plan" && entry.outcome === "skipped"));
	assert.ok(approved.steps.some((entry: any) => entry.note.includes("open finding")), "direct advancement discarded findings");
	await phase("continue"); await human("exit");
	assert.ok(runtime.requests.every((request) => JSON.stringify(request).split("# SCOPE").length === 2), "the invoked SCOPE checklist was missing or duplicated");
	const sessionId = runtime.session.sessionId;
	runtime.session.dispose();
	runtime = await start();
	assert.notEqual(runtime.session.sessionId, sessionId);
	await invokeScope("Resume the fixture");
	await phase("inspect");
	await phase("resume"); await human("continue");
	assert.deepEqual((await record()).plan_approval, approved.plan_approval);
	await tool("write", { path: planPath, content: "Permission must not survive resume" }, true);
	assert.equal(await readFile(join(root, planPath), "utf8"), planText);
	await phase("pause");
	assert.ok(runtime.requests.every((request) => !JSON.stringify(request).includes("superdev-terminal-repair")));
	assert.deepEqual(runtime.errors, []);
	assert.equal(git("branch", "--list", "work/*"), "");
	console.log("SUPERDEV_SCOPE_SDK_PASS");
} finally { runtime.session.dispose(); }

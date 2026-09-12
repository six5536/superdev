// The production adapter driving real BUILD and ACCEPT against Rust and Git.
//
// Execution runs in the controlling conversation, so this proves the stage
// operations, commit bounds, retry budgets, and closure policy without needing
// a model inside the worker. The worker runtime has its own proof.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { AssistantMessage } from "@earendil-works/pi-ai";
import superdev from "../../../.pi/extensions/superdev/index.ts";
import { proofSession } from "./superdev-runtime-session.ts";

const [root] = process.argv.slice(2);
const issue = "issue-001-execute", plan = "plan-042-execute";
const issuePath = `knowledge/issues/open/${issue}.md`, planPath = `knowledge/plans/open/${plan}.md`;
const issueText = `---\ntype: Issue\nid: ${issue}\ntitle: Execute the plan\ndescription: Test execution.\nlifecycle: open\n---\n\n# Issue\n\nThe intended outcome.\n`;
const planText = `---\ntype: Plan\nid: ${plan}\ntitle: Execution plan\ndescription: Test execution plan.\nlifecycle: open\nlinks:\n  - rel: implements\n    to: ${issue}\n---\n\n# Plan\n\nImplement the intended outcome in one block over \`src\`.\n`;
function git(...args: string[]) { return execFileSync("git", args, { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim(); }
git("init", "-q", "-b", "main"); git("config", "user.name", "Test"); git("config", "user.email", "test@example.com"); git("config", "commit.gpgsign", "false");
await writeFile(join(root, ".gitignore"), ".superdev/cache/\n.superdev/workflows/\nagent/\nsessions/\n");
git("add", ".gitignore"); git("commit", "-qm", "init");

let next: { name: string; arguments: Record<string, unknown> } | undefined;
let sequence = 0;
const runtime = await proofSession({ cwd: root, factory: superdev,
	reply: (): AssistantMessage["content"] => {
		if (!next) return [{ type: "text", text: "Scripted discussion." }];
		const tool = next; next = undefined;
		return [{ type: "toolCall", id: `execute-tool-${sequence++}`, ...tool }];
	},
});
async function tool(name: string, args: Record<string, unknown>, error = false) {
	next = { name, arguments: args };
	await runtime.session.prompt("Run the next scripted tool", { source: "extension" });
	const result = runtime.session.messages.findLast((message) => message.role === "toolResult");
	assert.ok(result && result.role === "toolResult");
	assert.equal(Boolean(result.isError), error, JSON.stringify(result));
	return result as { details?: Record<string, unknown> };
}
const phase = (phaseName: string, action: string, extra: Record<string, unknown> = {}, error = false) =>
	tool("superdev_run_phase", { phase: phaseName, action, ...extra }, error);
async function record() {
	const status = JSON.parse(execFileSync("superdev", ["workflow", "status", "--json"], { cwd: root, encoding: "utf8" }));
	return status.result.workflows[0]?.record;
}
async function human(text: string) {
	const requests = runtime.requests.length;
	await runtime.session.prompt(text, { source: "interactive" });
	assert.equal(runtime.requests.length, requests, "a resolved human action started a model turn");
}

try {
	// Reach an approved plan through the human-led SCOPE path.
	await phase("scope", "run", { issue }); await human("continue");
	await mkdir(join(root, "knowledge/issues/open"), { recursive: true });
	await writeFile(join(root, issuePath), issueText);
	await phase("scope", "record-step", { step: "select-issue", note: "Issue drafted" });
	await phase("scope", "approve-issue"); await human(`I approve ${issue}`);
	await phase("scope", "continue", { steps: ["write-plan"] }); await human("continue");
	await phase("scope", "attach-plan", { plan });
	await mkdir(join(root, "knowledge/plans/open"), { recursive: true });
	await writeFile(join(root, planPath), planText);
	await phase("scope", "record-step", { step: "write-plan", note: "Plan drafted" });
	await phase("scope", "approve-plan"); await human(`I approve ${plan}`);

	// Execution operations are refused until BUILD actually starts.
	assert.equal((await phase("build", "run", { note: "Implement block one" })).details!.status, "build-not-started");
	assert.equal((await record()).work_branch, undefined);

	// The handoff starts BUILD in this conversation and creates the branch.
	await phase("scope", "continue"); await human("BUILD here");
	const started = await record();
	assert.equal(started.phase, "build");
	assert.equal(started.work_branch, "work/001-execute");
	assert.equal(started.stage, "implementation");
	assert.equal(git("branch", "--show-current"), "work/001-execute");

	// A current-session stage discloses that it has no clean assessment context,
	// and returns the checklist and facts a worker would have been given.
	const stage = await phase("build", "run", { note: "Implement block one over src" });
	assert.equal(stage.details!.status, "execute-here");
	assert.equal(stage.details!.contextReset, "unavailable-in-current-session");
	const stageContext = String((stage.details as { stageContext?: string }).stageContext);
	assert.match(stageContext, /# IMPLEMENTATION stage/);
	assert.match(stageContext, /superdev_run_phase/, "the stage carried no BUILD checklist");
	assert.match(stageContext, new RegExp(plan), "the stage named no approved plan");
	assert.match(stageContext, /work\/001-execute/, "the stage named no work branch");

	// Partial work and verification results are durable, so a reset or crash
	// leaves notes to resume from rather than an empty checkpoint.
	await phase("build", "run", { note: "Continue block one",
		unfinished: "src/main.txt is half written; the error path is missing",
		evidence: ["cargo test: 3 passed"] });
	const midBlock = await record();
	assert.match(midBlock.checkpoint.unfinished, /half written/, "unfinished work was not saved");
	assert.deepEqual(midBlock.checkpoint.evidence, ["cargo test: 3 passed"], "verification evidence was not saved");

	// A block commit is bounded to the areas its plan declares.
	await mkdir(join(root, "src"), { recursive: true });
	await writeFile(join(root, "src/main.txt"), "block one\n");
	await writeFile(join(root, "unrelated.txt"), "not part of this block\n");
	await phase("build", "commit-block", { block: 1, note: "feat: block one", areas: ["src"] }, true);
	assert.equal(git("log", "--format=%s").split("\n").length, 3, "a refused block commit still changed history");
	await rm(join(root, "unrelated.txt"));
	await phase("build", "commit-block", { block: 1, note: "feat: block one", areas: ["src"] });
	const committed = await record();
	assert.deepEqual(committed.checkpoint.completed_blocks, [1]);
	assert.equal(committed.candidate, git("rev-parse", "HEAD"));
	// Committing the block settles its unfinished work.
	assert.equal(committed.checkpoint.unfinished, "");
	// A mid-block reset is requestable, not only the stage-boundary ones.
	const midReset = await phase("build", "run", { note: "Continue after a full context", reset: true });
	assert.equal(midReset.details!.contextReset, "unavailable-in-current-session",
		"the current conversation implied a reset it cannot perform");
	// A block is committed once; repeating it would overstate progress.
	await phase("build", "commit-block", { block: 1, note: "feat: block one again", areas: ["src"] }, true);

	// Retry budgets come from project policy and are durable and finite. No
	// manifest exists yet, so the safe defaults apply: three final corrections.
	for (let attempt = 0; attempt < 3; attempt++) await phase("build", "consume-retry", { note: "final-correction" });
	assert.equal((await record()).retries["final-correction"], 3);
	await phase("build", "consume-retry", { note: "final-correction" }, true);
	await phase("build", "consume-retry", { note: "no-such-budget" }, true);

	// An uncommitted change is unfinished work, not a candidate.
	await writeFile(join(root, "src/main.txt"), "uncommitted\n");
	await phase("build", "complete-build", {}, true);
	git("checkout", "--", "src/main.txt");
	await phase("build", "complete-build");
	const candidate = await record();
	assert.equal(candidate.phase, "accept");
	assert.equal(candidate.stage, "acceptance");

	// Without a readable policy, acceptance refuses rather than assuming it.
	await phase("accept", "accept", {}, true);
	assert.equal((await record()).phase, "accept");

	// ACCEPT findings return to BUILD and supersede the candidate.
	await phase("accept", "return-to-build", { note: "Block one omits its error path" });
	const returned = await record();
	assert.equal(returned.phase, "build");
	assert.equal(returned.candidate, undefined, "stale acceptance evidence survived a correction");
	assert.match(returned.discussion, /omits its error path/);
	assert.deepEqual(returned.plan_approval, candidate.plan_approval, "a correction discarded plan approval");
	assert.equal(returned.retries["final-correction"], 3, "returning to BUILD refilled a retry budget");
	await writeFile(join(root, "src/main.txt"), "block one, with its error path\n");
	await phase("build", "commit-block", { block: 2, note: "fix: add the error path", areas: ["src"] });

	// Project policy alone decides whether a human must confirm acceptance. The
	// policy is committed like any other project file: an untracked one would
	// leave the worktree dirty, and a dirty worktree has no candidate.
	await mkdir(join(root, ".superdev"), { recursive: true });
	await writeFile(join(root, ".superdev/config.toml"), "blueprint = \"0.2.0\"\n[workflow]\nhuman_acceptance_required = true\nmax_final_correction_cycles = 3\n");
	git("add", ".superdev/config.toml"); git("commit", "-qm", "chore: set the acceptance policy");
	await phase("build", "complete-build");
	const defaultBefore = git("rev-parse", "main");
	await phase("accept", "accept");
	assert.equal((await record()).phase, "accept", "a pending question accepted the candidate");
	// A document-approval phrase is not an acceptance choice. It is ordinary
	// discussion, so it reaches the model and leaves the question pending.
	await runtime.session.prompt(`I approve ${plan}`, { source: "interactive" });
	assert.equal((await record()).phase, "accept", "an approval phrase closed the workflow");
	await human("Accept this candidate");
	const accepted = await record();
	assert.equal(accepted.phase, "done");
	assert.equal(accepted.stage, undefined);
	assert.equal(git("rev-parse", "main"), defaultBefore, "acceptance merged into the default branch");
	assert.equal(git("branch", "--show-current"), "work/001-execute", "acceptance moved or deleted the work branch");
	assert.deepEqual(runtime.errors, []);
	console.log("SUPERDEV_EXECUTION_SDK_PASS");
} finally { runtime.session.dispose(); }

// The production worker controller driving a real forked worker process.
//
// This exercises the shipped `WorkerSession` and `worker-host.ts`, not a test
// re-implementation of them: one persistent session across stages, stage-scoped
// instructions and tools, questions routed to the controller with the human's
// own control, bounded shutdown, and termination of a stuck tree.
import assert from "node:assert/strict";
import { fork } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import { WorkerSession } from "../../../.pi/extensions/superdev/lib/worker.ts";
import type { QuestionReply } from "../../../.pi/extensions/superdev/lib/worker-host.ts";

const [root] = process.argv.slice(2);
const agentDir = join(root, "agent");
const sessionsDir = join(root, "sessions");
await mkdir(agentDir, { recursive: true });
await mkdir(sessionsDir, { recursive: true });
await writeFile(join(root, "requests.jsonl"), "");
const provider = new URL("./superdev-worker-provider.ts", import.meta.url).pathname;
const ctx = { cwd: root } as ExtensionContext;
const config = { agentDir, sessionsDir, extensions: [provider], offline: true };
const script = (steps: string[]) => writeFile(join(root, "script.json"), JSON.stringify(steps));
const requests = async () => (await readFile(join(root, "requests.jsonl"), "utf8"))
	.trim().split("\n").filter(Boolean).map((line) => JSON.parse(line));
const lastRequest = async () => (await requests()).at(-1);
const toolNames = (request: { tools?: Array<{ name: string }> }) => (request.tools ?? []).map((tool) => tool.name).sort();

const asked: string[] = [];
let reply: QuestionReply = { status: "answered", answer: "Keep it narrow", choiceId: "narrow" };
const hooks = {
	async ask(question: { question: string }) {
		asked.push(question.question);
		// The controller, not the worker, decides how a question is answered.
		return reply;
	},
};

await script(["BLOCK_1_DONE"]);
let worker = new WorkerSession(hooks);
const identity = await worker.start(config, ctx);
assert.ok(identity.session && identity.anchor, "the worker reported no resumable identity");
assert.ok(worker.running);

// An implementation stage receives its own checklist and may change files.
const first = await worker.run("implementation", "Implement block 1.", "# IMPLEMENTATION stage\nBUILD_CHECKLIST\nISSUE_FACT PLAN_FACT");
assert.equal(first, "BLOCK_1_DONE");
const building = await lastRequest();
assert.match(JSON.stringify(building), /BUILD_CHECKLIST/, "the stage checklist never reached the worker");
assert.match(JSON.stringify(building), /ISSUE_FACT PLAN_FACT/, "the stage's durable facts never reached the worker");
for (const tool of ["read", "write", "edit", "bash"]) {
	assert.ok(toolNames(building).includes(tool), `implementation lost ${tool}`);
}

// A reset empties the conversation, and the next stage still arrives complete:
// its own checklist and evidence, without the implementation or its verdict.
await script(["ask", "REVIEW_DONE"]);
await worker.reset();
const review = await worker.run("review", "Assess the candidate.",
	"# REVIEW stage\nREVIEW_CHECKLIST\nCANDIDATE_FACT");
assert.deepEqual(asked, ["Which boundary applies?"], "the worker did not route its question to the controller");
assert.equal(review, "REVIEW_DONE");
const reviewing = await lastRequest();
assert.doesNotMatch(JSON.stringify(reviewing), /BLOCK_1_DONE|BUILD_CHECKLIST/, "the reset kept the implementation conversation");
assert.match(JSON.stringify(reviewing), /REVIEW_CHECKLIST/, "the review stage lost its checklist to the reset");
assert.match(JSON.stringify(reviewing), /CANDIDATE_FACT/);
// A reviewer keeps bash: diffs, searches and type-checks are how a review is
// done. What stops it changing the candidate is the controller comparing the
// checkout's Git state before and after, not a shorter tool list.
for (const tool of ["read", "bash"]) {
	assert.ok(toolNames(reviewing).includes(tool), `review cannot inspect with ${tool}`);
}
for (const tool of ["write", "edit"]) {
	assert.ok(!toolNames(reviewing).includes(tool), `review was offered ${tool}, inviting an accident that voids it`);
}

// Returning to implementation restores its tools, so the stage owns them.
await script(["BLOCK_2_DONE"]);
await worker.run("implementation", "Correct block 2.", "# IMPLEMENTATION stage\nBUILD_CHECKLIST");
assert.ok(toolNames((await lastRequest())!).includes("edit"), "a corrected stage did not regain its tools");

// The human's control reaches the worker as itself, not as an invented answer.
await script(["ask", "AFTER_DISCUSS"]);
reply = { status: "discuss" };
assert.equal(await worker.run("implementation", "Ask something the human discusses.", "# IMPLEMENTATION stage"), "AFTER_DISCUSS");
const discussed = JSON.stringify(await lastRequest());
assert.match(discussed, /"status":"discuss"/, "Discuss was not delivered as a control");
assert.doesNotMatch(discussed, /Question was not answered/, "Discuss was delivered as a fabricated answer");
reply = { status: "answered", answer: "Keep it narrow", choiceId: "narrow" };

// A rejected controller question must not leave the worker waiting, and must
// not be reported to it as an answer the human gave.
await script(["ask", "RECOVERED"]);
const failing = new WorkerSession({ async ask() { throw new Error("refused"); } });
try {
	await failing.start(config, ctx);
	assert.match(await failing.run("implementation", "Ask something the controller refuses.", "# IMPLEMENTATION stage"), /RECOVERED/);
} finally { await failing.stop(); }

// A second worker for the same checkout is refused by the controller object.
await assert.rejects(worker.start(config, ctx), /already running/);

// An orderly stop keeps the session identity for the next controller.
assert.equal(await worker.stop(), "stopped");
assert.equal(worker.running, false);
assert.equal(await worker.stop(), "already-stopped");

// Restart reuses the same session file; work is not replayed from scratch.
await script(["RESUMED"]);
worker = new WorkerSession(hooks);
const resumed = await worker.start({ ...config, sessionFile: identity.sessionFile }, ctx);
assert.equal(resumed.session, identity.session, "restart created a second worker session");
assert.equal(resumed.sessionFile, identity.sessionFile);
assert.equal(await worker.run("implementation", "Continue after restart.", "# IMPLEMENTATION stage"), "RESUMED");
await worker.stop();

// A busy worker still stops in an orderly way: Pi cancels its running tool.
await script(["hang"]);
const busy = new WorkerSession(hooks);
await busy.start(config, ctx);
// Observe the outcome first: the in-flight stage is rejected during the stop,
// and an unobserved rejection would fail this process.
const cancelled = busy.run("implementation", "Run a command that does not finish.", "# IMPLEMENTATION stage")
	.then(() => "resolved", (error: Error) => error.message);
assert.equal(await busy.stop(), "stopped");
assert.match(await cancelled, /stopped before finishing/, "an interrupted stage was reported as finished");

// Losing the controller asks the worker to pause at its own boundary rather
// than cutting the turn off. The host is forked directly here, because only
// the owner of the channel can close it without signalling the process.
await script(["PAUSED_CLEANLY"]);
{
	const configPath = join(root, "orphan.json");
	await writeFile(configPath, JSON.stringify({ ...config, cwd: root }));
	const host = new URL("../../../.pi/extensions/superdev/lib/worker-host.ts", import.meta.url).pathname;
	const child = fork(host, [configPath], {
		cwd: root, stdio: ["ignore", "pipe", "pipe", "ipc"],
		execArgv: ["--experimental-transform-types", "--disable-warning=ExperimentalWarning"],
	});
	const exited = new Promise<number | null>((resolve) => child.once("exit", resolve));
	await new Promise<void>((resolve) => child.on("message", (message: { type: string }) => {
		if (message.type === "ready") resolve();
	}));
	child.send({ type: "run", id: "orphan-1", stage: "implementation",
		instructions: "Work while the controller disappears.", context: "# IMPLEMENTATION stage" });
	child.disconnect();
	// It ends itself cleanly rather than being killed, so its session file and
	// whatever the turn committed remain for the next controller.
	assert.equal(await exited, 0, "a disconnected worker did not pause and exit cleanly");
}

// An unresponsive worker is terminated after its bounded grace, together with
// its children. A terminated stage is never reported as completed work.
await script(["hang"]);
const stuck = new WorkerSession(hooks, { graceMs: 1_000 });
await stuck.start(config, ctx);
const pid = stuck.view().pid!;
const interrupted = stuck.run("implementation", "Run a command that does not finish.", "# IMPLEMENTATION stage")
	.then(() => "resolved", (error: Error) => error.message);
// Wait for the worker's own child command, then make the worker unresponsive.
await new Promise((resolve) => setTimeout(resolve, 1_500));
process.kill(pid, "SIGSTOP");
assert.equal(await stuck.stop(), "terminated");
assert.match(await interrupted, /stopped before finishing/);
assert.equal(stuck.running, false);
assert.throws(() => process.kill(pid, 0), { code: "ESRCH" }, "the stuck process tree survived termination");

console.log("SUPERDEV_WORKER_RUNTIME_PASS");

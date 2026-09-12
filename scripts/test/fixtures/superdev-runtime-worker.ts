import assert from "node:assert/strict";
import { mkdir, readFile, readdir, rename, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { Type } from "typebox";
import { SessionManager, type ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { proofSession, text } from "./superdev-runtime-session.ts";

const [cwd, operation] = process.argv.slice(2);
assert.ok(cwd && operation);
const checkpointPath = join(cwd, "checkpoint.json");
type Stage = "build" | "review" | "accept";
type Checkpoint = {
	sessionId: string; sessionFile: string; anchor: string; stage: Stage;
	completed: string[]; unfinished: string; attempts: number;
};
const checklists = {
	build: "BUILD_CHECKLIST: implement the remaining approved work and verify it.",
	review: "REVIEW_CHECKLIST: assess the immutable candidate against the approved records.",
	accept: "ACCEPT_CHECKLIST: check the candidate and the acceptance evidence.",
};

async function worker() {
	const saved: Checkpoint | undefined = operation === "resume"
		? JSON.parse(await readFile(checkpointPath, "utf8")) : undefined;
	let stage: Stage = saved?.stage ?? "build";
	let pi: ExtensionAPI;
	let cancelReset = false;
	const runtime = await proofSession({
		cwd, sessionFile: saved?.sessionFile,
		factory(api) {
			pi = api;
			api.on("before_agent_start", () => ({ message: {
				customType: "proof-stage", display: false,
				content: `${checklists[stage]}\nISSUE_FACT PLAN_FACT CANDIDATE_FACT`,
			} }));
			api.on("session_before_tree", () => cancelReset ? { cancel: true } : undefined);
			api.on("session_before_compact", ({ preparation }) => ({ compaction: {
				summary: "OLD_COMPACTION_VERDICT",
				firstKeptEntryId: preparation.firstKeptEntryId, tokensBefore: preparation.tokensBefore,
			} }));
		},
		reply: () => [{ type: "text", text: stage === "build" ? "IMPLEMENTATION_VERDICT" : `${stage}_result` }],
	});
	const { session, requests, errors } = runtime;
	try {
		const checkpoint: Checkpoint = saved ?? {
			sessionId: session.sessionId, sessionFile: session.sessionFile!,
			anchor: session.sessionManager.appendCustomEntry("proof-root", {}), stage,
			completed: [], unfinished: "block-1", attempts: 2,
		};
		assert.equal(session.sessionId, checkpoint.sessionId);
		assert.equal(session.sessionFile, checkpoint.sessionFile);
		const activate = () => pi!.setActiveTools(stage === "build" ? ["read", "write", "edit", "bash"] : ["read"]);
		activate();

		async function reset(next: Stage, path = checkpointPath, anchor = checkpoint.anchor) {
			await session.waitForIdle();
			const nextCheckpoint = { ...checkpoint, stage: next };
			await writeFile(`${path}.tmp`, JSON.stringify(nextCheckpoint));
			await rename(`${path}.tmp`, path);
			const result = await session.navigateTree(anchor, { summarize: false, label: "proof-reset" });
			if (result.cancelled) throw new Error("Reset cancelled; execution remains paused");
			stage = next;
			checkpoint.stage = next;
			activate();
			assert.equal(session.sessionId, checkpoint.sessionId);
			assert.deepEqual(session.messages, [], "reset retained old messages");
			assert.equal(savedContext(checkpoint.sessionFile).messages.length, 0,
				"reset did not persist before the next prompt");
		}

		if (!saved) {
			await session.prompt("BLOCK_1_IMPLEMENTATION", { source: "extension" });
			assert.match(text(requests.at(-1)!), /BUILD_CHECKLIST/);
			checkpoint.completed.push("block-1");
			checkpoint.unfinished = "block-2";
			await reset("build");
			await session.prompt("BLOCK_2_PARTIAL", { source: "extension" });
			assert.doesNotMatch(text(requests.at(-1)!), /BLOCK_1_IMPLEMENTATION/);
			checkpoint.unfinished = "block-2: remaining checks";
			await reset("build");
			await session.prompt("BLOCK_2_CONTINUATION", { source: "extension" });

			// Exercise real Pi compaction, including its retained recent messages.
			await session.compact();
			await session.prompt("After ordinary compaction", { source: "extension" });
			assert.match(text(requests.at(-1)!), /OLD_COMPACTION_VERDICT/);
			assert.match(JSON.stringify(savedContext(checkpoint.sessionFile)), /OLD_COMPACTION_VERDICT/);
			checkpoint.completed.push("block-2");
			checkpoint.unfinished = "";
			await reset("review");
		} else {
			// Restore from local facts and a saved clean path, not old conversation.
			assert.deepEqual(checkpoint.completed, ["block-1", "block-2"]);
			assert.equal(checkpoint.attempts, 2);
			assert.deepEqual(session.messages, []);
			assert.equal(stage, "review");
		}

		await session.prompt("Assess the bound candidate", { source: "extension" });
		const reviewRequest = requests.at(-1)!;
		assert.deepEqual(reviewRequest.tools?.map((tool) => tool.name), ["read"]);
		assert.match(text(reviewRequest), /REVIEW_CHECKLIST/);
		assert.match(text(reviewRequest), /ISSUE_FACT PLAN_FACT CANDIDATE_FACT/);
		assert.doesNotMatch(text(reviewRequest), /IMPLEMENTATION_VERDICT|OLD_COMPACTION_VERDICT|BUILD_CHECKLIST|BLOCK_[12]/);
		assert.equal((text(reviewRequest).match(/REVIEW_CHECKLIST/g) ?? []).length, 1);
		await reset("accept");
		await session.prompt("Assess acceptance", { source: "extension" });
		assert.deepEqual(requests.at(-1)!.tools?.map((tool) => tool.name), ["read"]);
		assert.match(text(requests.at(-1)!), /ACCEPT_CHECKLIST/);
		assert.doesNotMatch(text(requests.at(-1)!), /REVIEW_CHECKLIST|review_result|IMPLEMENTATION_VERDICT/);

		// Failed checkpoints cannot run the navigation, even when the filesystem fails.
		const oldLeaf = session.sessionManager.getLeafId();
		const oldMessages = structuredClone(session.messages);
		const badPath = join(cwd, "checkpoint-directory");
		await mkdir(badPath, { recursive: true });
		await assert.rejects(reset("build", badPath), /EISDIR|EPERM|EACCES/);
		assert.equal(session.sessionManager.getLeafId(), oldLeaf);
		assert.deepEqual(session.messages, oldMessages);
		await assert.rejects(reset("build", checkpointPath, "missing-anchor"), /not found/);
		assert.equal(session.sessionManager.getLeafId(), oldLeaf);
		assert.deepEqual(session.messages, oldMessages);
		cancelReset = true;
		await assert.rejects(reset("build"), /Reset cancelled/);
		assert.equal(session.sessionManager.getLeafId(), oldLeaf);
		assert.deepEqual(session.messages, oldMessages);
		cancelReset = false;

		// End at a durably empty review path; a separate OS process must reopen it.
		await reset("review");
		assert.deepEqual(errors, []);
		assert.equal((await readdir(join(cwd, "sessions"))).filter((name) => name.endsWith(".jsonl")).length, 1);
		console.log(JSON.stringify({ sessionId: session.sessionId, sessionFile: session.sessionFile,
			requests: requests.length, completed: checkpoint.completed, attempts: checkpoint.attempts }));
	} finally { session.dispose(); }
}

// SessionManager.open is the public restart reader, not an in-memory assertion.
function savedContext(path: string) {
	return SessionManager.open(path).buildSessionContext();
}

async function pendingTool() {
	const entered = Promise.withResolvers<void>();
	const release = Promise.withResolvers<void>();
	let called = false;
	const { session, errors } = await proofSession({
		cwd,
		factory(pi) {
			pi.registerTool({ name: "proof_hold", label: "Hold", description: "Hold a test tool open",
				parameters: Type.Object({}),
				async execute() { entered.resolve(); await release.promise;
					return { content: [{ type: "text", text: "settled" }], details: {} }; },
			});
		},
		reply: () => {
			if (called) return [{ type: "text", text: "Tool completed." }];
			called = true;
			return [{ type: "toolCall", id: "hold-1", name: "proof_hold", arguments: {} }];
		},
	});
	const anchor = session.sessionManager.appendCustomEntry("proof-root", {});
	try {
		const run = session.prompt("Run the holding tool", { source: "extension" });
		await entered.promise;
		const oldLeaf = session.sessionManager.getLeafId();
		await assert.rejects(session.navigateTree(anchor, { summarize: false }), /current response to finish/);
		assert.equal(session.sessionManager.getLeafId(), oldLeaf);
		release.resolve();
		await run;
		await session.waitForIdle();
		const entries = session.sessionManager.getEntries();
		assert.ok(entries.some((entry) => entry.type === "message"
			&& entry.message.role === "toolResult" && entry.message.toolCallId === "hold-1"));
		await session.navigateTree(anchor, { summarize: false, label: "after-settled-tool" });
		assert.deepEqual(session.messages, []);
		assert.deepEqual(errors, []);
		console.log(JSON.stringify({ pendingTool: "refused-until-settled" }));
	} finally { release.resolve(); await session.abort(); session.dispose(); }
}

if (operation === "pending") await pendingTool();
else await worker();

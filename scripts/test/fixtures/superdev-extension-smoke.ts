// Replacement surface tests. Retired child authoring/review-barrier tests are
// superseded by the real SDK SCOPE journey and the persistent-runtime suite.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { chmod, copyFile, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { loadSkills, formatSkillsForPrompt } from "@earendil-works/pi-coding-agent";
import superdev from "../../../.pi/extensions/superdev/index.ts";
import { WORKER_MARKER, stageTools } from "../../../.pi/extensions/superdev/lib/worker-host.ts";
import { modelMayNotRun, requiresHumanAcceptance } from "../../../.pi/extensions/superdev/lib/process.ts";
import { pinService } from "../../../.pi/extensions/superdev/lib/service-pin.ts";
import { runPinnedSuperdev } from "../../../.pi/extensions/superdev/lib/service-exec.ts";

const commands = new Map<string, any>(), tools = new Map<string, any>(), events = new Map<string, any>();
superdev({ on(name: string, callback: any) { events.set(name, callback); },
	registerTool(tool: any) { tools.set(tool.name, tool); }, registerCommand(name: string, command: any) { commands.set(name, command); },
} as never);
assert.deepEqual([...tools.keys()].sort(), ["superdev_ask", "superdev_run_phase"]);
// Inside the workflow's own worker this extension is discovered like any other
// project extension. It must stand aside: registering the workflow tool there
// would give a worker an authority surface it never holds, and its own ask tool
// would collide with this one and stop the worker starting at all.
{
	const workerTools = new Map<string, any>(), workerCommands = new Map<string, any>(), workerEvents = new Map<string, any>();
	process.env[WORKER_MARKER] = "1";
	try {
		superdev({ on(name: string, callback: any) { workerEvents.set(name, callback); },
			registerTool(tool: any) { workerTools.set(tool.name, tool); },
			registerCommand(name: string, command: any) { workerCommands.set(name, command); },
		} as never);
	} finally { delete process.env[WORKER_MARKER]; }
	assert.deepEqual([...workerTools.keys()], [], "the controller extension registered tools inside the worker");
	assert.deepEqual([...workerCommands.keys()], [], "the controller extension registered commands inside the worker");
	assert.deepEqual([...workerEvents.keys()], [], "the controller extension took over worker events");
}
for (const name of ["superdev", "superdev-status", "superdev-resume", "superdev-cancel"]) assert.ok(commands.has(name));
assert.equal(commands.has("superdev-force"), false, "SCOPE does not need a clean-review override path");
assert.equal(events.has("input"), true);
assert.equal(events.has("agent_end"), false, "retired terminal-repair instructions remain active");
assert.equal(events.get("before_agent_start")(), undefined, "startup injected an unused orchestration prompt");
const nativeInput = tools.get("superdev_ask").parameters.properties;
assert.ok(nativeInput.choices.items.properties.id);
assert.ok(nativeInput.recommendation.properties.choiceId);
assert.ok(!JSON.stringify(tools.get("superdev_run_phase").parameters).includes("human_approved"));
// An assessment keeps bash, because a reviewer that cannot run a diff or a
// type-check reviews by reading alone. It is bounded by comparing the
// checkout's Git state before and after, which a shell cannot evade, rather
// than by a command list it could reach around with a variable or a script.
for (const stage of ["review", "acceptance"] as const) {
	for (const inspecting of ["read", "bash"]) {
		assert.ok(stageTools[stage].includes(inspecting), `${stage} cannot inspect with ${inspecting}`);
	}
	for (const mutating of ["write", "edit"]) {
		assert.ok(!stageTools[stage].includes(mutating), `${stage} was offered ${mutating}, inviting an accident that voids it`);
	}
}
for (const stage of ["implementation", "verification"] as const) {
	for (const tool of ["read", "write", "edit", "bash"]) {
		assert.ok(stageTools[stage].includes(tool), `${stage} lost ${tool}`);
	}
}
const directory = await mkdtemp(join(tmpdir(), "superdev-discovery-"));
try {
	const names = ["file", "scope", "build", "accept", "grill-me", "double-check"];
	const defaults = loadSkills({ cwd: process.cwd(), agentDir: directory, skillPaths: [], includeDefaults: true });
	assert.ok(defaults.skills.every((skill) => !names.includes(skill.name)), "bundled skills leaked into default discovery");
	const resources = events.get("resources_discover")();
	const discovered = loadSkills({ cwd: directory, agentDir: directory, skillPaths: resources.skillPaths, includeDefaults: false });
	const prompt = formatSkillsForPrompt(discovered.skills);
	for (const name of names) {
		const matches = discovered.skills.filter((skill) => skill.name === name);
		assert.equal(matches.length, 1);
		assert.ok(!matches[0].disableModelInvocation && prompt.includes(`<name>${name}</name>`));
		assert.equal(await readFile(matches[0].filePath, "utf8"), await readFile(join(process.cwd(), `pack/pi/extensions/superdev/skills/${name}/SKILL.md`), "utf8"));
	}
	for (const path of ["index.ts", "lib/client.ts", "lib/scope.ts", "lib/questions.ts", "lib/ask-tool.ts", "lib/phase-tool.ts", "lib/service-exec.ts", "lib/service-pin.ts", "lib/process.ts", "lib/execution.ts", "lib/worker.ts", "lib/worker-host.ts"]) {
		assert.equal(await readFile(join(process.cwd(), ".pi/extensions/superdev", path), "utf8"), await readFile(join(process.cwd(), "pack/pi/extensions/superdev", path), "utf8"), path);
	}
	for (const path of ["prompts/scope.md", "prompts/requirements-review.md", "prompts/orchestrator.md", "lib/phases.ts", "lib/intake.ts"]) {
		await assert.rejects(readFile(join(process.cwd(), ".pi/extensions/superdev", path)), { code: "ENOENT" });
		await assert.rejects(readFile(join(process.cwd(), "pack/pi/extensions/superdev", path)), { code: "ENOENT" });
	}
} finally { await rm(directory, { recursive: true, force: true }); }

assert.throws(() => requiresHumanAcceptance(undefined), /omitted/);
assert.equal(requiresHumanAcceptance(true), true); assert.equal(requiresHumanAcceptance(false), false);
// Authority and Git mutation never reach the model's shell, including through
// the retired v2 verbs an obsolete instruction might still name.
for (const text of ["superdev workflow apply", "superdev workflow start", "superdev workflow transition",
	"superdev workflow commit", "superdev workflow abandon", "git commit -am bypass", "git switch other"]) {
	assert.ok(modelMayNotRun(text), text);
}
for (const text of ["superdev workflow status", "cargo test", "git diff --check"]) assert.equal(modelMayNotRun(text), false, text);

// Preserve executable snapshot and digest checks while removing the old engine.
if (process.platform === "linux") {
	const root = await mkdtemp(join(tmpdir(), "superdev-launcher-test-"));
	let pinned: Awaited<ReturnType<typeof pinService>> | undefined;
	try {
		const launcher = join(root, "launcher"), native = join(root, "native");
		await copyFile("/bin/echo", native);
		await writeFile(launcher, '#!/bin/sh\nroot="$(dirname "$0")"\n[ -f "$root/native" ] || exit 2\nprintf \'{"protocol":"superdev-workflow/v3","result":{"executable":"%s/native"}}\\n\' "$root"\n');
		await chmod(launcher, 0o700);
		assert.notEqual((await runPinnedSuperdev(launcher, undefined, ["workflow", "status"], root)).code, 0);
		pinned = await pinService(launcher, root);
		await rm(native); await rm(launcher);
		assert.equal((await runPinnedSuperdev(pinned.path, pinned.digest, ["stable-service"], root)).stdout.trim(), "stable-service");
		await assert.rejects(runPinnedSuperdev(pinned.path, "0".repeat(64), ["bad"], root), /changed after/);
		const digest = createHash("sha256").update(await readFile(process.execPath)).digest("hex");
		const result = await runPinnedSuperdev(process.execPath, digest, ["-e", "process.stdin.on('data', b => process.stdout.write(b))"], root, undefined, undefined, "stdin request");
		assert.equal(result.stdout, "stdin request");
	} finally { pinned?.dispose(); await rm(root, { recursive: true, force: true }); }
}

console.log("SUPERDEV_WORKFLOW_SMOKE_PASS");
export default function loadedSmoke() {}

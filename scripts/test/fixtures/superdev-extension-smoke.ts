import { createHash } from "node:crypto";
import { readFile, rm, stat } from "node:fs/promises";

import superdev, { buildCommandAllowed, isolatedRoleMayNotRun, isolatedTools, parseRoleResult, requiresHumanAcceptance, runGuardedBuildCommand, runPinnedSuperdev } from "../../../.pi/extensions/superdev/index.ts";
import { IsolatedArtifact, boundedText } from "../../../.pi/extensions/superdev/lib/output.ts";
import { registerWorkflowQuestions } from "../../../.pi/extensions/superdev/lib/questions.ts";
import { findingFingerprint, validateRoleResult, type ReviewFinding } from "../../../.pi/extensions/superdev/lib/review.ts";

export default async function smoke() {
	const commands: string[] = [];
	const tools: string[] = [];
	const fake = {
		on() {
			// Registration is enough for this load smoke.
		},
		registerCommand(name: string) {
			commands.push(name);
		},
		registerTool(tool: { name: string }) {
			tools.push(tool.name);
		},
	};
	superdev(fake as never);
	for (const command of [
		"superdev",
		"scope",
		"build",
		"accept",
		"superdev-status",
		"superdev-resume",
		"superdev-cancel",
		"superdev-abandon",
		"file",
	]) {
		if (!commands.includes(command)) throw new Error(`missing command ${command}`);
	}
	if (!tools.includes("superdev_isolated_role")) throw new Error("missing isolated role tool");
	if (tools.includes("superdev_review_diff")) throw new Error("immutable review diff leaked into the main agent tool set");
	if (!tools.includes("superdev_workflow_control")) throw new Error("missing UI-gated workflow control tool");
	if (!tools.includes("superdev_workflow_questions")) throw new Error("missing stateful workflow question tool");
	if (!requiresHumanAcceptance(true) || requiresHumanAcceptance(false)) throw new Error("configured acceptance policy changed");
	try {
		requiresHumanAcceptance(undefined);
		throw new Error("missing acceptance policy was treated as automatic");
	} catch (error) {
		if (!String(error).includes("omitted")) throw error;
	}
	for (const command of [
		"superdev workflow evidence",
		"superdev workflow scope-checkpoint",
		"superdev workflow correction",
		"superdev workflow sync",
		"superdev workflow transition",
		"superdev workflow integrate",
	]) {
		if (!isolatedRoleMayNotRun(command)) throw new Error(`isolated role may bypass ${command}`);
	}
	if (!isolatedRoleMayNotRun("git commit -am bypass")) throw new Error("isolated role may mutate Git history");
	if (isolatedTools("scope").split(",").includes("bash")) throw new Error("SCOPE child has direct shell access");
	if (isolatedTools("build").split(",").includes("bash")) throw new Error("BUILD child has direct shell access");
	if (isolatedTools("accept").split(",").some((tool) => ["bash", "edit", "write"].includes(tool))) throw new Error("ACCEPT assessor is not read-only");
	if (isolatedTools("code-review").split(",").includes("read")) throw new Error("final reviewer can bypass immutable diff bounds");
	if (!isolatedTools("code-review").split(",").includes("superdev_submit_result")) throw new Error("reviewer lacks typed terminal submission");
	if (!isolatedTools("build").split(",").includes("superdev_build_exec")) throw new Error("BUILD child cannot execute bounded evidence");
	if (buildCommandAllowed("sh", ["-c", "mutate Git"])) throw new Error("BUILD can escape through a shell");
	if (buildCommandAllowed("cargo", ["test"])) throw new Error("BUILD can bypass Rust-owned verification");
	if (buildCommandAllowed("git", ["diff", "--check"])) throw new Error("BUILD can invoke Git directly");
	if (!buildCommandAllowed("superdev", ["workflow", "block"])) throw new Error("BUILD cannot publish a block checkpoint");
	if (buildCommandAllowed("superdev", ["file"])) throw new Error("BUILD can escape through an unrelated service command");
	if (buildCommandAllowed("superdev", ["workflow", "transition"])) throw new Error("BUILD can transition workflow state");
	for (const command of [
		"superdev workflow block",
		"superdev workflow attempt",
		"superdev workflow correction-checkpoint",
		"superdev workflow status",
	]) {
		if (isolatedRoleMayNotRun(command)) throw new Error(`isolated role cannot perform ${command}`);
	}
	const checklist = ["requirements", "contracts", "architecture", "tests", "documentation", "scope", "consistency"]
		.map((area) => ({ area, complete: true, evidence: `${area} checked` }));
	const clean = parseRoleResult(
		"code-review",
		`analysis\nSUPERDEV_RESULT ${JSON.stringify({ status: "clean", summary: "No actionable findings.", checklist })}`,
	);
	if (clean.status !== "clean") throw new Error("structured clean review was not parsed");
	try {
		validateRoleResult("code-review", { status: "findings", summary: "too many", findings: Array(2).fill({ id: "duplicate" }), checklist }, { maxBytes: 10_000, maxFindings: 1 });
		throw new Error("review finding limit was ignored");
	} catch (error) {
		if (!String(error).includes("more than 1")) throw error;
	}
	const finding: ReviewFinding = { id: "f1", classification: "substantive", summary: "Choose policy", evidence: "Policy is absent", impact: "Behavior is unsettled", question: "Which policy?", recommendation: "Use the safe policy" };
	if (findingFingerprint(finding) === findingFingerprint({ ...finding, impact: "Different behavior" })) throw new Error("semantic fingerprint ignored impact");
	for (const [label, result] of [
		["contradictory status", { status: "complete", summary: "contradiction", findings: [{ ...finding, classification: "correctable-within-scope" }], checklist }],
		["cyclic dependencies", { status: "findings", summary: "cycle", findings: [
			{ ...finding, id: "a", classification: "correctable-within-scope", dependsOn: ["b"] },
			{ ...finding, id: "b", classification: "correctable-within-scope", dependsOn: ["a"] },
		], checklist }],
		["duplicate checklist", { status: "clean", summary: "duplicate", checklist: [...checklist, checklist[0]] }],
	] as const) {
		try {
			validateRoleResult("code-review", result, { maxBytes: 100_000, maxFindings: 100 });
			throw new Error(`${label} was accepted`);
		} catch (error) {
			if (String(error).includes("was accepted")) throw error;
		}
	}
	const bounded = boundedText("one\ntwo\nthree", { maxContextBytes: 20, maxContextLines: 2, maxArtifactBytes: 100, maxArtifacts: 2, retentionHours: 1 });
	if (!bounded.includes("Showing 2 of 3 lines")) throw new Error("model-visible output was not line bounded");
	const artifact = await IsolatedArtifact.create("smoke", "review", 16);
	try {
		artifact.writeStderr("0123456789abcdefghijklmnop");
		await artifact.finishStderr();
		if ((await stat(artifact.directory)).mode & 0o077) throw new Error("artifact directory is not owner-only");
		try {
			await artifact.writeResult({ result: "this authoritative result is too large" });
			throw new Error("oversized authoritative result was accepted");
		} catch (error) {
			if (!String(error).includes("isolated-output-overflow")) throw error;
		}
	} finally {
		await rm(artifact.directory, { recursive: true, force: true });
	}
	const questionTools = new Map<string, any>();
	const entries: any[] = [];
	let active = ["read", "edit"];
	let paused = false;
	const questionPi = {
		appendEntry(type: string, data: unknown) { entries.push({ type: "custom", customType: type, data: structuredClone(data) }); },
		getActiveTools() { return active; },
		setActiveTools(next: string[]) { active = next; },
		registerTool(tool: any) { questionTools.set(tool.name, tool); },
	};
	const controller = registerWorkflowQuestions(questionPi, {
		maxBytes: () => 16_384,
		maxFindings: () => 100,
		onPause: async () => { paused = true; },
		onResume: async () => { paused = false; },
		onSubmit: async () => {},
	});
	controller.begin({ version: 1, workflow: "plan-smoke", candidate: "abcdef1", status: "active", findings: [finding], answers: {} });
	if (active.includes("edit") || !active.includes("superdev_workflow_questions")) throw new Error("question discussion tools are not read-only");
	const questionTool = questionTools.get("superdev_workflow_questions");
	await questionTool.execute("q1", { action: "propose-answer", findingIds: ["f1"], proposedAnswer: "Use the safe policy" }, new AbortController().signal, undefined, { hasUI: true, ui: { confirm: async () => true } });
	await questionTool.execute("q2", { action: "pause" }, new AbortController().signal, undefined, {});
	if (!paused || controller.current()?.status !== "paused") throw new Error("question pause did not persist and release ownership");
	await questionTool.execute("q3", { action: "resume" }, new AbortController().signal, undefined, {});
	if (paused || controller.current()?.status !== "active") throw new Error("question resume did not reacquire ownership");
	try {
		parseRoleResult("code-review", "CLEAN");
		throw new Error("unstructured review was accepted");
	} catch (error) {
		if (String(error).includes("unstructured review was accepted")) throw error;
	}
	const checkpoint = await runGuardedBuildCommand(async (command, args, cwd) => {
		if (command !== "superdev" || args.join(" ") !== "workflow block" || cwd !== "/repo") throw new Error("BUILD service command changed");
		return { code: 0, stdout: "checkpointed\n", stderr: "" };
	}, "superdev", ["workflow", "block"], "/repo");
	if (checkpoint.stdout !== "checkpointed\n") throw new Error("BUILD checkpoint result was lost");
	try {
		await runGuardedBuildCommand(async () => ({ code: 0, stdout: "", stderr: "" }), "cargo", ["test"], "/repo");
		throw new Error("direct BUILD executable was accepted");
	} catch (error) {
		if (!String(error).includes("not permitted")) throw error;
	}
	if (process.platform === "linux") {
		const digest = createHash("sha256").update(await readFile(process.execPath)).digest("hex");
		const pinned = await runPinnedSuperdev(process.execPath, digest, ["-e", "process.stdout.write('pinned')"], process.cwd());
		if (pinned.code !== 0 || pinned.stdout !== "pinned") throw new Error("descriptor-pinned executable did not run");
		try {
			await runPinnedSuperdev(process.execPath, "0".repeat(64), ["--version"], process.cwd());
			throw new Error("pinned executable accepted a stale digest");
		} catch (error) {
			if (!String(error).includes("changed after the parent pinned it")) throw error;
		}
	}
}

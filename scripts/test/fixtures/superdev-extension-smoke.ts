import { createHash } from "node:crypto";
import { loadSkills, formatSkillsForPrompt } from "@earendil-works/pi-coding-agent";
import { chmod, copyFile, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

import superdev, { buildCommandAllowed, isolated, isolatedRoleMayNotRun, isolatedTools, parseRoleResult, requiresHumanAcceptance, runGuardedBuildCommand, runPinnedSuperdev } from "../../../.pi/extensions/superdev/index.ts";
import { IsolatedArtifact, boundedText } from "../../../.pi/extensions/superdev/lib/output.ts";
import { registerPhaseDrivers, type PhaseRuntime } from "../../../.pi/extensions/superdev/lib/phases.ts";
import { registerPhaseTool } from "../../../.pi/extensions/superdev/lib/phase-tool.ts";
import { pinService } from "../../../.pi/extensions/superdev/lib/service-pin.ts";
import { registerIntakeTools } from "../../../.pi/extensions/superdev/lib/intake.ts";
import { withProgress } from "../../../.pi/extensions/superdev/lib/progress.ts";
import { registerWorkflowQuestions } from "../../../.pi/extensions/superdev/lib/questions.ts";
import { roleResultSchemaFor, roles, validateRoleResult, type ReviewFinding } from "../../../.pi/extensions/superdev/lib/review.ts";

async function smoke() {
	if (process.platform === "linux") {
		const root = await mkdtemp(join(tmpdir(), "superdev-launcher-test-"));
		let pinned: Awaited<ReturnType<typeof pinService>> | undefined;
		try {
			const launcher = join(root, "launcher");
			const native = join(root, "native");
			await copyFile("/bin/echo", native);
			await writeFile(launcher, '#!/bin/sh\nroot="$(dirname "$0")"\n[ -f "$root/native" ] || exit 2\nprintf \'{"protocol":"superdev-workflow/v2","result":{"executable":"%s/native"}}\\n\' "$root"\n');
			await chmod(launcher, 0o700);
			// Copying a relative-path launcher to /proc/fd loses its original root.
			const broken = await runPinnedSuperdev(launcher, undefined, ["workflow", "status", "--json"], root);
			if (broken.code === 0) throw new Error("launcher regression did not reproduce");
			pinned = await pinService(launcher, root);
			await rm(native);
			await rm(launcher);
			// The source checkout may now contain an older launcher or a rebuilt
			// executable. Neither affects the parent/child service snapshot.
			const result = await runPinnedSuperdev(pinned.path, pinned.digest, ["stable-service"], root);
			if (result.code !== 0 || result.stdout.trim() !== "stable-service") throw new Error("service snapshot depended on the changed checkout");
		} finally {
			pinned?.dispose();
			await rm(root, { recursive: true, force: true });
		}
	}
	const commands: string[] = [];
	const tools: string[] = [];
	const toolDefinitions = new Map<string, any>();
	const eventHandlers = new Map<string, any>();
	const fake = {
		on(name: string, handler: any) {
			eventHandlers.set(name, handler);
		},
		registerCommand(name: string) {
			commands.push(name);
		},
		registerTool(tool: { name: string }) {
			tools.push(tool.name);
			toolDefinitions.set(tool.name, tool);
		},
	};
	superdev(fake as never);
	const skillAgentDir = await mkdtemp(join(tmpdir(), "superdev-skill-discovery-"));
	try {
		const defaults = loadSkills({ cwd: process.cwd(), agentDir: skillAgentDir, skillPaths: [], includeDefaults: true });
		if (defaults.skills.some((skill) => ["file", "scope", "build", "accept"].includes(skill.name))) throw new Error("Superdev skills leaked into general Pi discovery");
		const discover = eventHandlers.get("resources_discover");
		if (!discover) throw new Error("extension did not register its skills");
		const startup = await discover({ cwd: skillAgentDir, reason: "startup" });
		const reloaded = await discover({ cwd: skillAgentDir, reason: "reload" });
		if (JSON.stringify(startup) !== JSON.stringify(reloaded)) throw new Error("skill registration differs on reload");
		const discovered = loadSkills({ cwd: skillAgentDir, agentDir: skillAgentDir, skillPaths: startup.skillPaths, includeDefaults: false });
		const prompt = formatSkillsForPrompt(discovered.skills);
		if (discovered.skills.some((skill) => skill.name === "issue")) throw new Error("retired issue skill remains discoverable");
		const filing = await readFile(join(process.cwd(), ".pi/extensions/superdev/skills/file/SKILL.md"), "utf8");
		for (const instruction of ["Choose the next unused number", "If already on the default branch", "worktree on the default branch", "superdev validate --fix", "Commit only the authored record", "preserve unfinished files", "refuse pre-existing edits", "read-only `superdev validate`", "git commit --only", "../../../../skills/sokf-authoring/SKILL.md"]) {
			if (!filing.includes(instruction)) throw new Error(`file skill omitted ${instruction}`);
		}
		if (filing.includes("superdev_file_issue") || filing.includes("superdev_run_phase")) throw new Error("file skill still depends on deterministic filing or workflow state");
		const scopeSkill = await readFile(join(process.cwd(), ".pi/extensions/superdev/skills/scope/SKILL.md"), "utf8");
		if (!scopeSkill.includes("Git mutation commands are permitted only while following `../file/SKILL.md`") || !scopeSkill.split("\n").find((line) => line.startsWith("allowed-tools:"))?.split(" ").includes("bash")) {
			throw new Error("SCOPE cannot delegate issue creation to the native file skill");
		}
		for (const name of ["file", "scope", "build", "accept"]) {
			const skill = discovered.skills.find((skill) => skill.name === name);
			if (!skill || skill.disableModelInvocation || !skill.filePath.endsWith(`/.pi/extensions/superdev/skills/${name}/SKILL.md`) || !prompt.includes(`<name>${name}</name>`)) {
				throw new Error(`native Pi discovery omitted ${name} from its prompt`);
			}
			const packed = await readFile(join(process.cwd(), "pack/pi/extensions/superdev/skills", name, "SKILL.md"), "utf8");
			if (packed !== await readFile(skill.filePath, "utf8")) throw new Error(`${name} skill pack differs`);
		}
	} finally { await rm(skillAgentDir, { recursive: true, force: true }); }
	for (const role of ["scope", "requirements-review", "build", "code-review", "accept"]) {
		const prompt = await readFile(join(process.cwd(), ".pi", "extensions", "superdev", "prompts", `${role}.md`), "utf8");
		if (!prompt.includes("only tool call in a separate tool-call message") || !prompt.includes("exactly one accepted result") || !prompt.includes("submit again") || !prompt.includes("Invoke the actual tool") || prompt.includes("final assistant turn")) {
			throw new Error(`${role} prompt does not keep terminal submission separate and singular`);
		}
	}
	const scopePrompt = await readFile(join(process.cwd(), ".pi", "extensions", "superdev", "prompts", "scope.md"), "utf8");
	if (!scopePrompt.includes("primary issue and canonical plan named by the task") || !scopePrompt.includes("Keep their established identities")) {
		throw new Error("SCOPE prompt permits identity drift");
	}
	const phaseSource = await readFile(join(process.cwd(), ".pi", "extensions", "superdev", "lib", "phases.ts"), "utf8");
	for (const identityTask of ["Prepare issue ${initial.owner.identity.issue} and plan ${initial.owner.identity.plan}", "Review issue ${owner.identity.issue} and plan ${owner.identity.plan}", "Build issue ${owner.identity.issue} from approved plan ${owner.identity.plan}", "Assess whether issue ${owner.identity.issue} and plan ${owner.identity.plan}"]) {
		if (!phaseSource.includes(identityTask)) throw new Error(`phase task omits canonical identity: ${identityTask}`);
	}
	for (const removed of ["scope", "build", "accept", "issue", "file"]) {
		if (commands.includes(removed)) throw new Error(`legacy phase command ${removed} remains public`);
	}
	for (const command of [
		"superdev",
		"superdev-status",
		"superdev-resume",
		"superdev-cancel",
		"superdev-abandon",
	]) {
		if (!commands.includes(command)) throw new Error(`missing command ${command}`);
	}
	if (!tools.includes("superdev_run_phase")) throw new Error("missing generic phase tool");
	if (tools.includes("superdev_file_issue") || !tools.includes("superdev_ask")) throw new Error("filing must be skill-only; questions remain available");
	for (const role of roles) {
		const schema = roleResultSchemaFor(role);
		if (Boolean(schema.properties.findings) !== schema.properties.status.enum.includes("findings") || schema.additionalProperties !== false) throw new Error(`${role} advertises contradictory result fields`);
	}
	const priorChildRole = process.env.SUPERDEV_CHILD_ROLE;
	process.env.SUPERDEV_CHILD_ROLE = "scope";
	try {
		const childTools = new Map<string, any>();
		const childHooks = new Map<string, any[]>();
		const repairs: any[] = [];
		let childActiveTools = ["read", "edit", "superdev_submit_result"];
		superdev({
			on(name: string, handler: any) { childHooks.set(name, [...(childHooks.get(name) ?? []), handler]); },
			registerCommand() {}, registerTool(tool: any) { childTools.set(tool.name, tool); },
			setActiveTools(tools: string[]) { childActiveTools = tools; },
			sendMessage(message: any, options: any) { repairs.push({ message, options }); },
		} as never);
		const submit = childTools.get("superdev_submit_result");
		if (submit.parameters.properties.findings || submit.parameters.properties.status.enum.join(",") !== "complete,blocked") throw new Error("SCOPE submission schema invites reviewer findings or invalid statuses");
		try {
			await submit.execute("rejected", { status: "complete", summary: "   " });
			throw new Error("an undecodable SCOPE result was accepted");
		} catch (error) {
			if (!String(error).includes("No result was accepted") || !String(error).includes("submit again")) throw error;
		}
		const end = async (stopReason: string, aborted = false) => {
			const controller = new AbortController();
			if (aborted) controller.abort();
			for (const hook of childHooks.get("agent_end") ?? []) await hook({ messages: [{ role: "assistant", stopReason }] }, { signal: controller.signal });
		};
		await end("aborted", true);
		await end("error");
		if (repairs.length) throw new Error("terminal repair continued cancellation or provider failure");
		await end("stop");
		if (repairs.length !== 1 || childActiveTools.join(",") !== "superdev_submit_result" || repairs[0].options.deliverAs !== "followUp" || !repairs[0].message.content.includes("invalid terminal result")) throw new Error("stopped child did not receive one terminal-only repair with its rejection");
		await end("stop");
		if (repairs.length !== 1) throw new Error("terminal repair loop is unbounded");
		for (const toolName of ["read", "edit", "superdev_run_phase"]) {
			const blocks = await Promise.all((childHooks.get("tool_call") ?? []).map((hook) => hook({ toolName, input: {} })));
			if (!blocks.some((block) => block?.block && block.terminate)) throw new Error("terminal repair allowed repeated work");
		}
		const first = await submit.execute("first", { status: "complete", summary: "Scoped once" });
		const duplicate = await submit.execute("duplicate", { status: "complete", summary: "Scoped twice" });
		if (first.details?.superdevResult?.summary !== "Scoped once" || first.terminate !== true) throw new Error("first terminal result was not accepted");
		await end("stop");
		if (repairs.length !== 1) throw new Error("accepted result triggered terminal repair");
		if (duplicate.details?.superdevDuplicate !== true || duplicate.details?.superdevResult !== undefined || duplicate.terminate !== true) {
			throw new Error("duplicate terminal result was not terminated without replacing authority");
		}
	} finally {
		if (priorChildRole === undefined) delete process.env.SUPERDEV_CHILD_ROLE;
		else process.env.SUPERDEV_CHILD_ROLE = priorChildRole;
	}
	if (tools.includes("superdev_review_diff")) throw new Error("immutable review diff leaked into the main agent tool set");
	if (tools.includes("superdev_workflow_control")) throw new Error("low-level workflow control leaked");
	if (tools.includes("superdev_workflow_questions")) throw new Error("internal workflow question tool was exposed");
	const phaseSchema = JSON.stringify(toolDefinitions.get("superdev_run_phase")?.parameters);
	for (const action of ["run", "retry", "inspect", "ask", "record-answer", "revise-answer", "submit-answers", "approve", "cancel"]) {
		if (!phaseSchema.includes(`\"${action}\"`)) throw new Error(`generic phase tool omitted ${action}`);
	}
	if (phaseSchema.includes("expectedRevision") || phaseSchema.includes("session")) throw new Error("generic phase tool exposed internal ownership mechanics");
	const priorVerification = process.env.SUPERDEV_VERIFICATION_ACTIVE;
	process.env.SUPERDEV_VERIFICATION_ACTIVE = "1";
	try {
		const phaseTool = toolDefinitions.get("superdev_run_phase");
		const inspect = await phaseTool.execute("inspect", { phase: "scope", action: "inspect" }, undefined, undefined, { cwd: "/repo" });
		if (inspect.details?.status !== "idle" || inspect.details?.questions !== null) throw new Error("generic inspect did not return typed idle state");
		const cancel = await phaseTool.execute("cancel", { phase: "scope", action: "cancel" }, undefined, undefined, { cwd: "/repo" });
		if (cancel.details?.status !== "idle") throw new Error("generic cancel did not return typed idle state");

	} finally {
		if (priorVerification === undefined) delete process.env.SUPERDEV_VERIFICATION_ACTIVE;
		else process.env.SUPERDEV_VERIFICATION_ACTIVE = priorVerification;
	}
	if (!requiresHumanAcceptance(true) || requiresHumanAcceptance(false)) throw new Error("configured acceptance policy changed");
	try {
		requiresHumanAcceptance(undefined);
		throw new Error("missing acceptance policy was treated as automatic");
	} catch (error) {
		if (!String(error).includes("omitted")) throw error;
	}
	for (const command of [
		"superdev workflow evidence",
		"superdev workflow scope-baseline",
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
	// A completed review that contradicted its own status is routed, not discarded.
	const routed = validateRoleResult("code-review", { status: "clean", summary: "contradiction", findings: [{ ...finding, classification: "correctable-within-scope" }] }, { maxBytes: 100_000, maxFindings: 100 });
	if (routed.status !== "findings" || routed.findings?.length !== 1) throw new Error("a contradicted status discarded a completed review");
	// A role without a findings channel reports completed work in its summary alone.
	const unrouted = validateRoleResult("scope", { status: "complete", summary: "resolved corrections", findings: [{ ...finding }] }, { maxBytes: 100_000, maxFindings: 100 });
	if (unrouted.status !== "complete" || unrouted.findings) throw new Error("an unroutable finding set was not dropped from a completed result");
	// Quality and completeness belong to the author; only decodability is enforced.
	const sparse = validateRoleResult("code-review", { status: "findings", summary: "sparse", findings: [{ id: "only-id", classification: "requires-scope", summary: "", evidence: "", impact: "" }] }, { maxBytes: 100_000, maxFindings: 100 });
	if (sparse.findings?.length !== 1) throw new Error("an incomplete finding was discarded instead of routed");
	if (validateRoleResult("code-review", { status: "clean", summary: "no checklist" }, { maxBytes: 100_000, maxFindings: 100 }).status !== "clean") {
		throw new Error("a clean review required a checklist the parent never reads");
	}
	for (const [label, result] of [
		["duplicate finding IDs", { status: "findings", summary: "duplicate", findings: [{ ...finding, id: "a", classification: "requires-scope" }, { ...finding, id: "a", classification: "requires-scope" }] }],
		["cyclic dependencies", { status: "findings", summary: "cycle", findings: [
			{ ...finding, id: "a", classification: "correctable-within-scope", dependsOn: ["b"] },
			{ ...finding, id: "b", classification: "correctable-within-scope", dependsOn: ["a"] },
		] }],
		["unknown dependency", { status: "findings", summary: "dangling", findings: [{ ...finding, id: "a", classification: "requires-scope", dependsOn: ["missing"] }] }],
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
		artifact.writeStdout("0123456789abcdefghijklmnop");
		artifact.writeStderr("0123456789abcdefghijklmnop");
		await Promise.all([artifact.finishStdout(), artifact.finishStderr()]);
		if (artifact.stdoutSummary().bytes !== 16 || artifact.stdoutSummary().discardedBytes !== 10) throw new Error("raw isolated trace was not bounded");
		if ((await stat(artifact.stdoutPath)).mode & 0o077) throw new Error("raw isolated trace is not owner-only");
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
	const fakePiDirectory = await mkdtemp(join(tmpdir(), "superdev-fake-pi-"));
	const fakePi = join(fakePiDirectory, "pi");
	const originalPath = process.env.PATH;
	let diagnosticDirectory: string | undefined;
	try {
		await writeFile(fakePi, `#!/usr/bin/env node\nconsole.log(JSON.stringify({type:"message_end",message:{role:"assistant",stopReason:"stop"}}));\nconsole.log(JSON.stringify({type:"tool_execution_start",toolName:"superdev_submit_result",args:{status:"complete"}}));\nconsole.log(JSON.stringify({type:"tool_execution_end",toolName:"superdev_submit_result",isError:true,result:{content:[{type:"text",text:"summary is required"}]}}));\n`, { mode: 0o700 });
		await chmod(fakePi, 0o700);
		process.env.PATH = `${fakePiDirectory}:${originalPath ?? ""}`;
		try {
			await isolated("scope", "exercise diagnostic capture", process.cwd());
			throw new Error("invalid terminal submission was accepted");
		} catch (error) {
			const match = String(error).match(/diagnostics: (\/[^\s]+)/);
			if (!match) throw error;
			if (!String(error).includes("summary is required")) throw new Error("parent failure hid the submission rejection");
			diagnosticDirectory = dirname(match[1]);
			const diagnostic = JSON.parse(await readFile(match[1], "utf8"));
			if (diagnostic.outcome !== "terminal-protocol-failure" || diagnostic.submission.starts !== 1 || diagnostic.submission.ends !== 1 || diagnostic.submission.errors !== 1 || diagnostic.assistantEnds !== 1) {
				throw new Error("isolated terminal diagnostics omitted lifecycle evidence");
			}
			if (!diagnostic.events?.path?.endsWith("/events.jsonl") || diagnostic.events.bytes === 0 || diagnostic.events.discardedBytes !== 0) {
				throw new Error("isolated terminal diagnostic omitted its retained raw event trace");
			}
			const trace = await readFile(diagnostic.events.path, "utf8");
			if (!trace.includes('"toolName":"superdev_submit_result"')) throw new Error("raw isolated trace omitted terminal events");
		}
		for (const text of ["to=functions.superdev_submit_result ... wait tool target commentary not final?", "界".repeat(10_000)]) {
			await writeFile(fakePi, `#!/usr/bin/env node\nif (${text.length < 100}) { for (let i = 0; i < 3000; i++) console.log(JSON.stringify({type:"padding",text:"x".repeat(4096)})); }\nconsole.log(JSON.stringify({type:"message_end",message:{role:"assistant",stopReason:"stop",provider:"test-provider",model:"test-model",content:[{type:"text",text:${JSON.stringify(text)}},{type:"thinking",thinking:"private reasoning"}]}}));\n`);
			try {
				await isolated("scope", "exercise prose-only submission", process.cwd());
				throw new Error("prose-only submission was accepted");
			} catch (error) {
				const match = String(error).match(/diagnostics: (\/[^\s]+)/);
				if (!match) throw error;
				const diagnostic = JSON.parse(await readFile(match[1], "utf8"));
				const last = diagnostic.lastAssistant;
				if (diagnostic.outcome !== "terminal-protocol-failure" || diagnostic.submission.starts !== 0 || last.toolCallCount !== 0 || last.textBytes !== Buffer.byteLength(text)) throw new Error("prose-only diagnostics omitted terminal evidence");
				if (last.model !== "test-model" || last.provider !== "test-provider" || !diagnostic.launch.tools.includes("superdev_submit_result") || !/^[a-f0-9]{64}$/.test(diagnostic.launch.promptSha256)) throw new Error("diagnostics omitted launch/model identity");
				if (text.length < 100 && (last.text !== text || diagnostic.events.discardedBytes === 0)) throw new Error("diagnostics lost the simulated invocation after raw trace truncation");
				if (Buffer.byteLength(last.text) > 2_300 || JSON.stringify(diagnostic).includes("private reasoning")) throw new Error("assistant diagnostic was unbounded or leaked reasoning");
				if (!diagnostic.terminalTimeline.some((event: any) => event.type === "assistant-end")) throw new Error("timeline omitted assistant completion");
				await rm(dirname(match[1]), { recursive: true, force: true });
			}
		}
		await writeFile(fakePi, `#!/usr/bin/env node\nconst result={status:"complete",summary:"accepted once"};\nconsole.log(JSON.stringify({type:"turn_start"}));\nconsole.log(JSON.stringify({type:"tool_execution_start",toolName:"superdev_submit_result",args:result}));\nconsole.log(JSON.stringify({type:"tool_execution_end",toolName:"superdev_submit_result",isError:false,result:{details:{superdevResult:result}}}));\nconsole.log(JSON.stringify({type:"tool_execution_start",toolName:"superdev_submit_result",args:result}));\nconsole.log(JSON.stringify({type:"tool_execution_end",toolName:"superdev_submit_result",isError:false,result:{details:{superdevDuplicate:true}}}));\n`, { mode: 0o700 });
		const accepted = await isolated("scope", "exercise duplicate suppression", process.cwd());
		if (accepted.status !== "complete" || accepted.summary !== "accepted once" || !accepted.artifactPath) throw new Error("one-shot terminal authority was not preserved");
		const acceptedDirectory = dirname(accepted.artifactPath);
		const acceptedDiagnostic = JSON.parse(await readFile(join(acceptedDirectory, "diagnostic.json"), "utf8"));
		if (acceptedDiagnostic.outcome !== "complete" || acceptedDiagnostic.submission.successful !== 1 || acceptedDiagnostic.submission.duplicates !== 1) {
			throw new Error("duplicate terminal submission was not recorded without failing the child");
		}
		if (!acceptedDiagnostic.terminalTimeline.some((event: any) => event.tool === "superdev_submit_result" && event.duplicate === true)) {
			throw new Error("terminal timeline omitted the duplicate event");
		}
		await rm(acceptedDirectory, { recursive: true, force: true });
	} finally {
		process.env.PATH = originalPath;
		await rm(fakePiDirectory, { recursive: true, force: true });
		if (diagnosticDirectory) await rm(diagnosticDirectory, { recursive: true, force: true });
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

	// Execute the registered phase handlers against a deterministic fake Pi and
	// workflow service. This catches orchestration gaps that registration-only
	// smoke coverage cannot observe.
	const phaseCommands = new Map<string, any>();
	const serviceCalls: string[][] = [];
	const progress: string[] = [];
	const selections: string[] = ["Approve only"];
	let phase: "scope" | "build" | "accept" = "scope";
	let revision = "revision-1";
	let owned = true;
	let humanAcceptanceRequired = false;
	let candidateEvidence = false;
	let finalCorrections = 0;
	let pendingPhaseQuestions: any;
	const baseRevision = "1".repeat(40);
	const candidateRevision = "2".repeat(40);
	const runtime: PhaseRuntime = { cancelling: false, modifyingBusy: false };
	let roleFailure: string | undefined;
	let blockedRole: string | undefined;
	let reviewFindings: any[] | undefined;
	let codeReviewFindings: any[] | undefined;
	const isolatedTasks: Array<{ role: string; task: string }> = [];
	let cancellableRole: string | undefined;
	let cancelNextProgress = false;
	const owner = () => owned ? {
		session_id: "session-smoke",
		last_plan_revision: revision,
		scope_base_revision: baseRevision,
		...(candidateEvidence ? { candidate_revision: candidateRevision, verified_default_revision: baseRevision } : {}),
		identity: { issue: "issue-smoke", plan: "plan-smoke", work_branch: "work/smoke", default_branch: "main" },
	} : undefined;
	const status = async () => ({
		owner: owner(), phase, canonicalPlanRevision: revision,
		humanAcceptanceRequired, executable: "/service", executableSha256: lateDigest,
		maxFinalCorrectionCycles: 3, maxScopeReviewCycles: 3,
		buildState: { currentBlock: 1, attempts: 0, finalCorrections, blocker: finalCorrections >= 3 ? "final correction limit exhausted: remaining findings" : "none" },
	});
	let attested = false;
    let lateDigest: string | undefined;
    const phasePi = {
		registerCommand(name: string, command: any) { phaseCommands.set(name, command); },
		async exec(_command: string, args: string[]) {
			if (args[0] === "status") return { code: 0, stdout: "", stderr: "" };
			const target = args.at(-1);
			return { code: 0, stdout: `${target === "main" || target === baseRevision ? baseRevision : attested ? "c".repeat(40) : candidateRevision}\n`, stderr: "" };
		},
	};
	const runService = async (args: string[]) => {
		serviceCalls.push(args);
		// The authoring role leaves the valid plan unchanged; the checkpoint commit
		// must still reach requirements review with the same canonical revision.
		if (args.includes("commit")) revision = "revision-3";
		if (args.includes("approve-scope")) { phase = "build"; revision = "revision-4"; }
		if (args.includes("complete-build")) { phase = "accept"; revision = "revision-5"; candidateEvidence = true; attested = true; }
		if (args.includes("integrate")) throw new Error("ACCEPT must never merge");
        if (args.includes("--transition") && args.at(-1) === "accept") owned = false;
		if (args.includes("cancel")) owned = false;
		return { result: args.includes("transition") ? { state: { last_plan_revision: revision } } : { last_plan_revision: revision } };
	};
	const ui = {
		setStatus(_key: string, value?: string) { if (value) progress.push(value); },
		notify() {},
		async select() { return selections.shift(); },
		async confirm() { return true; },
		async input() { return "confirmed"; },
		async custom(factory: any) {
			return await new Promise((resolve) => {
				const component = factory({ requestRender() {} }, { fg(_style: string, text: string) { return text; } }, {}, resolve);
				component.render(80);
				if (cancelNextProgress) {
					cancelNextProgress = false;
					component.handleInput("\u001b");
				}
			});
		},
	};
	const phaseCtx = { cwd: "/repo", model: undefined, hasUI: true, sessionManager: { getSessionId: () => "session-smoke" }, ui };
	const phaseDrivers = registerPhaseDrivers({
		pi: phasePi,
		runSuperdev: runService,
		workflowStatus: status,
		authority: "authority",
		policyFrom: () => ({ timeoutSeconds: 1, maxContextBytes: 8192, maxContextLines: 200, maxReviewStateBytes: 262144, maxReviewFindings: 100, maxArtifactBytes: 10485760, maxArtifacts: 20, retentionHours: 24 }),
		questions: { begin(state: any) { pendingPhaseQuestions = state; }, current() { return pendingPhaseQuestions; } },
		isolated: async (role: string, task: string, _cwd: string, _model: unknown, signal: AbortSignal, _spawn: any, _close: any, _base: any, _candidate: any, executable: any, digest: any) => {
			isolatedTasks.push({ role, task });
            if (role === "build" && (executable !== "/service" || digest !== "digest")) throw new Error("BUILD did not use its invocation-time executable pin");
			if (roleFailure === role) throw new Error(`${role} timed out after 1s; diagnostics: /tmp/${role}-diagnostic.json`);
			if (cancellableRole === role) return await new Promise((_resolve, reject) => {
				const cancelled = () => reject(new Error(`${role} role was cancelled; diagnostics: /tmp/${role}-cancelled.json`));
				if (signal.aborted) cancelled();
				else signal.addEventListener("abort", cancelled, { once: true });
			});
			if (blockedRole === role) return { status: "blocked", summary: `${role} blocked`, artifactPath: `/tmp/${role}-result.json` };
			if (role === "requirements-review" && reviewFindings) return { status: "findings", summary: "mixed findings", findings: reviewFindings, checklist };
			if (role === "code-review" && codeReviewFindings) return { status: "findings", summary: "correctable findings", findings: codeReviewFindings, checklist };
			return role === "scope" || role === "build"
				? { status: "complete", summary: `${role} complete` }
				: { status: role === "accept" ? "complete" : "clean", summary: `${role} clean`, checklist };
		},
		childStarted: () => () => {}, childFinished: async () => {}, reviewRuns: new Map(),
		requiresHumanAcceptance, runtime,
	});
	await phaseDrivers.runScopePhase("", phaseCtx);
	if (runtime.lastOutcome?.status !== "ready-for-approval" || phase !== "scope") throw new Error("SCOPE did not return a typed approval gate to its skill");
	const scopeCommit = serviceCalls.find((args) => args.includes("commit"));
	if (!scopeCommit || scopeCommit[scopeCommit.indexOf("--expected-revision") + 1] !== "revision-1") throw new Error("unchanged SCOPE did not checkpoint at the original plan revision");
	if (serviceCalls.some((args) => args.some((value) => ["evidence", "assess", "sync", "scope-baseline", "scope-checkpoint", "correction"].includes(value)))) {
		throw new Error("a phase driver still calls a removed service verb");
	}
	phase = "build"; revision = "revision-4"; runtime.lastOutcome = undefined; lateDigest = "digest";
	await phaseDrivers.runBuildPhase("", phaseCtx);
	if (owned || serviceCalls.some((args) => args.includes("integrate"))) throw new Error("BUILD did not finish at the manual merge boundary");
	if (!progress.some((value) => value.includes("SCOPE")) || !progress.some((value) => value.includes("BUILD")) || !progress.some((value) => value.includes("ACCEPT"))) {
		throw new Error("phase handlers did not publish immediate progress");
	}
	owned = true; phase = "accept"; revision = "revision-approval"; candidateEvidence = true; humanAcceptanceRequired = true; runtime.lastOutcome = undefined;
	await phaseDrivers.runAcceptPhase("", phaseCtx);
	if (!owned || runtime.lastOutcome?.status !== "ready-for-approval" || runtime.lastOutcome?.expectedRevision !== revision) {
		throw new Error("ACCEPT did not return a revision-bound typed approval gate");
	}

	owned = true; phase = "build"; revision = "revision-timeout"; candidateEvidence = false; humanAcceptanceRequired = false;
	roleFailure = "build"; runtime.lastOutcome = undefined;
	const callsBeforeTimeout = serviceCalls.length;
	await phaseDrivers.runBuildPhase("", phaseCtx);
	if (owned || runtime.lastOutcome?.status !== "failed" || runtime.lastOutcome?.failedStage !== "build implementation" || runtime.lastOutcome?.diagnosticPath !== "/tmp/build-diagnostic.json" || runtime.lastOutcome?.partialWorkPreserved !== true) throw new Error("timed-out BUILD did not release ownership with stage-specific typed recovery");
	if (serviceCalls.slice(callsBeforeTimeout).some((args) => args.includes("resume"))) throw new Error("timed-out BUILD retried without an explicit retry operation");

	owned = true; phase = "build"; revision = "revision-review-blocked"; roleFailure = undefined; blockedRole = "code-review"; finalCorrections = 0; runtime.lastOutcome = undefined;
	const callsBeforeBlockedReview = serviceCalls.length;
	await phaseDrivers.runBuildPhase("", phaseCtx);
	if (runtime.lastOutcome?.status !== "blocked" || runtime.lastOutcome?.artifactPath !== "/tmp/code-review-result.json") throw new Error("blocked code review did not return typed recovery with its exact artifact path");
	if (serviceCalls.slice(callsBeforeBlockedReview).some((args) => args.includes("correction"))) throw new Error("blocked code review consumed a semantic correction cycle");

	owned = true; phase = "accept"; revision = "revision-cancelled"; candidateEvidence = true; blockedRole = undefined; cancellableRole = "accept"; cancelNextProgress = true; runtime.lastOutcome = undefined;
	await phaseDrivers.runAcceptPhase("", phaseCtx);
	if (owned || runtime.lastOutcome?.status !== "paused" || runtime.lastOutcome?.failedStage !== "accept assessment" || runtime.lastOutcome?.diagnosticPath !== "/tmp/accept-cancelled.json") throw new Error("Esc did not pause ACCEPT with typed preserved-work recovery");

	// The correction budget is counted in the driver, not parsed out of the plan.
	owned = true; phase = "build"; revision = "revision-exhausted"; candidateEvidence = false; cancellableRole = undefined;
	pendingPhaseQuestions = undefined;
	codeReviewFindings = [{ id: "cr-1", classification: "correctable-within-scope", summary: "Correct it", evidence: "absent", impact: "drift" }];
	const tasksBeforeExhaustion = isolatedTasks.length;
	await phaseDrivers.runBuildPhase("", phaseCtx);
	codeReviewFindings = undefined;
	if (pendingPhaseQuestions?.originPhase !== "build" || pendingPhaseQuestions?.candidate !== revision) {
		throw new Error("exhausted BUILD did not preserve a revision-bound human decision queue");
	}
	const corrections = isolatedTasks.slice(tasksBeforeExhaustion).filter((entry) => entry.role === "build" && entry.task.includes("correction batch")).length;
	if (corrections !== 2) throw new Error(`BUILD spent ${corrections} correction cycles instead of its configured budget`);

	// A mixed requirements review must ask before correcting anything, because a
	// mechanical finding may declare an unanswered substantive finding as a dependency.
	owned = true; phase = "scope"; revision = "revision-mixed"; finalCorrections = 0;
	pendingPhaseQuestions = undefined; runtime.lastOutcome = undefined;
	reviewFindings = [
		{ id: "sub-1", classification: "substantive", summary: "Choose the policy", evidence: "absent", impact: "unsettled", question: "Which policy?", recommendation: "The safe one" },
		{ id: "mech-1", classification: "mechanical", summary: "Update the map", evidence: "absent", impact: "drift", dependsOn: ["sub-1"] },
	];
	const tasksBeforeMixed = isolatedTasks.length;
	await phaseDrivers.runScopePhase("", phaseCtx);
	if (pendingPhaseQuestions?.findings?.length !== 1 || pendingPhaseQuestions?.mechanicalFindings?.length !== 1) {
		throw new Error("mixed requirements review did not preserve the complete finding set for one decision batch");
	}
	if (isolatedTasks.slice(tasksBeforeMixed).some((entry) => entry.task.includes("correction batch"))) {
		throw new Error("mixed requirements review corrected a dependent finding before its substantive answer");
	}
	reviewFindings = undefined;
	await phaseDrivers.continueScopeFromAnswers({ ...pendingPhaseQuestions, status: "submitted", answers: { "sub-1": { answer: "Use the safe policy", findingIds: ["sub-1"], confirmedAt: "now" } } }, phaseCtx);
	const correctionTask = isolatedTasks.findLast((entry) => entry.task.includes("correction batch"))?.task ?? "";
	for (const required of ["sub-1", "mech-1", "Use the safe policy", "dependsOn"]) {
		if (!correctionTask.includes(required)) throw new Error(`SCOPE correction batch omitted ${required}`);
	}

    {
    const publicTools = new Map<string, any>();
    const questionPi = { registerTool(tool: any) { publicTools.set(tool.name, tool); }, registerCommand() {},
        getActiveTools() { return ["read", "superdev_run_phase"]; }, setActiveTools() {}, appendEntry() {} };
    let selectedChoices: string[] = [];
    const questionCtx = { ...phaseCtx, ui: { ...ui, async select(_question: string, choices: string[]) { selectedChoices = choices; return choices[0]; } } };
    const queue = registerWorkflowQuestions(questionPi, { exposeTool: false, maxBytes: () => 262144, maxFindings: () => 100, onSubmit: async () => {} });
    queue.begin({ version: 1, workflow: "plan-smoke", candidate: "queue-revision", status: "active", findings: [{ id: "choice", classification: "substantive", summary: "Choose", evidence: "Evidence", impact: "Impact", question: "Which?", recommendation: "First", choices: [{ label: "First" }, { label: "Second" }] }], answers: {} });
    registerPhaseTool({ pi: questionPi, workflowStatus: async () => ({ owner: { session_id: "session-smoke", last_plan_revision: "queue-revision", identity: { plan: "plan-smoke" } }, phase: "scope" }), questions: queue, runtime: {}, policyFrom: () => ({ maxContextBytes: 8192, maxContextLines: 200 }), phaseContinuations: {} });
    queue.begin({ ...queue.current()!, findings: [...queue.current()!.findings, { ...queue.current()!.findings[0], id: "later", dependsOn: ["choice"] }] });
    for (const findingIds of [undefined, [], ["missing"], ["choice", "missing"], ["later"]]) {
        const selection = await publicTools.get("superdev_run_phase").execute("ask", { phase: "scope", action: "ask", findingIds }, undefined, undefined, questionCtx);
        if (selection.details?.status !== "finding-selection-required" || !selection.details?.eligibleFindingIds.includes("choice") || Object.keys(queue.current()!.answers).length || selectedChoices.length) throw new Error("missing or invalid ask ID failed the workflow or selected silently");
    }
    const asked = await publicTools.get("superdev_run_phase").execute("ask", { phase: "scope", action: "ask", findingIds: ["choice"] }, undefined, undefined, questionCtx);
    if (!asked.details?.answered?.includes("choice") || !selectedChoices.includes("Type another answer") || !selectedChoices.includes("Discuss")) throw new Error("Public phase tool cannot reach the question selector");
    const answeredSelection = await publicTools.get("superdev_run_phase").execute("ask", { phase: "scope", action: "ask", findingIds: ["choice"] }, undefined, undefined, questionCtx);
    if (answeredSelection.details?.status !== "finding-selection-required" || answeredSelection.details?.eligibleFindingIds.join(",") !== "later") throw new Error("ask did not preserve answered/dependency eligibility");
    await publicTools.get("superdev_run_phase").execute("ask", { phase: "scope", action: "ask", findingIds: ["later"] }, undefined, undefined, questionCtx);
    await queue.operate({ action: "submit" }, questionCtx);
    const revised = await publicTools.get("superdev_run_phase").execute("change", { phase: "scope", action: "record-answer", answer: "A later human revision" }, undefined, undefined, questionCtx);
    if (revised.details?.status !== "answer-recorded") throw new Error("A submitted prior batch blocked a later human revision");
    registerIntakeTools({ pi: questionPi });
    const intake = await publicTools.get("superdev_ask").execute("intake", { question: "Choose scope", choices: ["Small", "Large"], recommendation: "Small" }, undefined, undefined, questionCtx);
    if (intake.details?.answer !== "Small" || !selectedChoices.includes("Discuss in chat")) throw new Error("Intake choices are unavailable");

    }
	let progressAborted = false;
	let progressCleared = false;
	try {
		await withProgress({ hasUI: true, ui: {
			setStatus(_key: string, value?: string) { if (!value) progressCleared = true; },
			async custom(factory: any) {
				return await new Promise((resolve) => {
					const component = factory({ requestRender() {} }, { fg(_style: string, text: string) { return text; } }, {}, resolve);
					component.handleInput("\u001b");
				});
			},
		} }, { key: "smoke-progress", title: "BUILD plan-smoke", stage: "implementation" }, async (signal) => {
			await new Promise<void>((_resolve, reject) => signal.addEventListener("abort", () => { progressAborted = true; reject(new Error("cancelled")); }, { once: true }));
		});
		throw new Error("Esc cancellation unexpectedly completed");
	} catch (error) {
		if (!progressAborted || !String(error).includes("cancelled")) throw error;
	}
	if (!progressCleared) throw new Error("Esc cancellation left stale progress status");
}

await smoke();
console.log("SUPERDEV_WORKFLOW_SMOKE_PASS");

export default function loadedSmoke() {
	// Top-level await completes the deterministic smoke before Pi registers this fixture.
}

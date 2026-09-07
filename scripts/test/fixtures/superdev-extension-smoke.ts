import superdev, { buildCommandAllowed, isolatedRoleMayNotRun, isolatedTools, parseRoleResult, runGuardedBuildCommand } from "../../../.pi/extensions/superdev/index.ts";

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
	if (!tools.includes("superdev_review_diff")) throw new Error("missing read-only review diff tool");
	if (!tools.includes("superdev_workflow_control")) throw new Error("missing UI-gated workflow control tool");
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
	if (!isolatedTools("build").split(",").includes("superdev_build_exec")) throw new Error("BUILD child cannot execute bounded evidence");
	if (buildCommandAllowed("sh", ["-c", "mutate Git"])) throw new Error("BUILD can escape through a shell");
	if (buildCommandAllowed("./script", [])) throw new Error("BUILD can escape through an unbounded executable path");
	if (!buildCommandAllowed("just", ["check"])) throw new Error("BUILD cannot run a project-declared executable");
	if (buildCommandAllowed("git", ["-C", ".", "commit"])) throw new Error("BUILD can bypass Git operation checks");
	if (buildCommandAllowed("git", ["commit"])) throw new Error("BUILD can commit directly");
	if (!buildCommandAllowed("git", ["diff", "--check"])) throw new Error("BUILD cannot inspect Git");
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
	const clean = parseRoleResult(
		"code-review",
		'analysis\nSUPERDEV_RESULT {"status":"clean","summary":"No actionable findings."}',
	);
	if (clean.status !== "clean") throw new Error("structured clean review was not parsed");
	try {
		parseRoleResult("code-review", "CLEAN");
		throw new Error("unstructured review was accepted");
	} catch (error) {
		if (String(error).includes("unstructured review was accepted")) throw error;
	}
	const ok = (stdout = "") => ({ code: 0, stdout, stderr: "" });
	const stable = [ok("aaa\n"), ok("checked\n"), ok("aaa\n")];
	const stableResult = await runGuardedBuildCommand(async () => stable.shift()!, "just", ["check"], "/repo");
	if (stableResult.stdout !== "checked\n") throw new Error("guarded BUILD result was lost");
	const movedCalls: string[][] = [];
	const moved = [ok("aaa\n"), ok(), ok("bbb\n"), ok()];
	try {
		await runGuardedBuildCommand(async (command, args) => {
			movedCalls.push([command, ...args]);
			return moved.shift()!;
		}, "cargo", ["test"], "/repo");
		throw new Error("moved BUILD HEAD was accepted");
	} catch (error) {
		if (!String(error).includes("attempted to mutate Git history")) throw error;
	}
	if (movedCalls.at(-1)?.join(" ") !== "git update-ref HEAD aaa bbb") throw new Error("moved BUILD HEAD was not rolled back by CAS");
	const rollbackFailure = [ok("aaa\n"), ok(), ok("bbb\n"), { code: 1, stdout: "", stderr: "stale" }];
	try {
		await runGuardedBuildCommand(async () => rollbackFailure.shift()!, "npm", ["test"], "/repo");
		throw new Error("BUILD rollback failure was accepted");
	} catch (error) {
		if (!String(error).includes("rollback failed")) throw error;
	}
}

import superdev, { parseRoleResult } from "../../../.pi/extensions/superdev/index.ts";

export default function smoke() {
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
}

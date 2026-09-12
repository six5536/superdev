import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { WorkflowClient } from "./lib/client.ts";
import { HumanQuestions } from "./lib/questions.ts";
import { ScopeController } from "./lib/scope.ts";
import { registerPhaseTool } from "./lib/phase-tool.ts";
import { registerAskTool } from "./lib/ask-tool.ts";
import { WORKER_MARKER } from "./lib/worker-host.ts";
import { modelMayNotRun } from "./lib/process.ts";

const here = dirname(fileURLToPath(import.meta.url));

export default function superdev(pi: ExtensionAPI) {
	// Old role processes must stop before migration. They cannot acquire v3 authority.
	if (process.env.SUPERDEV_CHILD_ROLE) {
		pi.on("tool_call", (event) => event.toolName === "read" ? undefined : {
			block: true, reason: "Legacy isolated workflow roles are retired; stop this process before migration", terminate: true,
		});
		return;
	}
	// Inside the workflow's own worker this extension is discovered like any
	// other project extension. It stands aside: the worker holds no controller
	// capability, so registering the workflow tool here would give it an
	// authority surface it must never have, and its own ask tool would collide
	// with this one. Other project extensions still load, so the worker keeps
	// capabilities such as knowledge search.
	if (process.env[WORKER_MARKER]) return;
	pi.on("resources_discover", () => ({ skillPaths: [join(here, "skills")] }));
	const client = new WorkflowClient();
	const report = (value: unknown, _ctx: ExtensionContext) => pi.sendMessage({
		customType: "superdev-workflow-state", content: JSON.stringify(value), display: true,
	}, { triggerTurn: false });
	const questions = new HumanQuestions(report);
	const controller = new ScopeController(pi, client, questions);
	registerPhaseTool(pi, controller);
	// Asking the human a well-formed question is useful outside a workflow too,
	// so the tool is available whenever the extension is.
	registerAskTool(pi, (question, ctx) => controller.interview(question, ctx));

	pi.on("input", (event, ctx) => questions.onInput(event, ctx));
	pi.on("session_start", (_event, ctx) => { controller.restore(ctx); });
	pi.on("before_agent_start", () => {
		const state = controller.view();
		if (!state.workflow && !state.pendingQuestion) return;
		return { message: { customType: "superdev-workflow-state", content: JSON.stringify(state), display: false } };
	});
	pi.on("tool_call", (event, ctx) => {
		if (event.toolName === "bash" && controller.view().workflow) {
			const command = String((event.input as { command?: unknown }).command ?? "");
			if (modelMayNotRun(command)) return { block: true, reason: "Use the workflow service for authority and Git mutation; branch switches require an explicit human action" };
		}
		if (!["write", "edit"].includes(event.toolName)) return;
		const path = (event.input as { path?: unknown }).path;
		if (typeof path !== "string") return;
		const reason = controller.guard(path, ctx.cwd);
		if (reason) return { block: true, reason };
	});
	pi.on("session_before_switch", async (_event, ctx) => {
		if (!controller.view().claim && !questions.view()) return;
		try { await controller.pause(ctx); }
		catch (error) { ctx.ui.notify(String(error), "error"); return { cancel: true }; }
	});
	pi.on("session_before_fork", async (_event, ctx) => {
		if (!controller.view().claim && !questions.view()) return;
		try { await controller.pause(ctx); }
		catch (error) { ctx.ui.notify(String(error), "error"); return { cancel: true }; }
	});
	pi.on("session_shutdown", async (_event, ctx) => {
		try { if (controller.view().claim) await controller.pause(ctx); }
		catch (error) { ctx.ui.notify(`Workflow ownership was not released: ${String(error)}`, "error"); }
		finally { await client.dispose(); }
	});
	pi.registerCommand("superdev", {
		description: "Invoke the human-led SCOPE checklist",
		handler: async (args, ctx) => {
			if (!ctx.isIdle()) { ctx.ui.notify("Wait for the current turn before invoking SCOPE", "warning"); return; }
			const skill = await readFile(join(here, "skills/scope/SKILL.md"), "utf8");
			pi.sendUserMessage(`${skill}\n\nUser request: ${args || "Inspect local workflow progress"}`);
		},
	});
	pi.registerCommand("superdev-status", {
		description: "Show local progress and current approval validity",
		handler: async (_args, ctx) => report(await controller.inspect(ctx), ctx),
	});
	pi.registerCommand("superdev-resume", {
		description: "Resume local progress with fresh SCOPE permission",
		handler: async (args, ctx) => report(await controller.run({ phase: "scope", action: "resume", workflow: args.trim() || undefined }, ctx), ctx),
	});
	pi.registerCommand("superdev-cancel", {
		description: "Pause without discarding progress, documents, or valid approval",
		handler: async (_args, ctx) => report(await controller.pause(ctx), ctx),
	});
	// Ending work is the human's decision, so it lives as a typed command that
	// no model can reach. SCOPE no longer has a review barrier to override:
	// the human may approve a document over open findings at any time, so the
	// retired force-gate command has nothing left to force.
	pi.registerCommand("superdev-abandon", {
		description: "Human-only abandonment with knowledge disposition",
		handler: async (args, ctx) => {
			const reason = args.trim() || (ctx.hasUI ? (await ctx.ui.input("Reason for abandonment"))?.trim() : undefined);
			if (!reason) { ctx.ui.notify("Abandonment needs a reason; nothing was changed", "warning"); return; }
			try { report(await controller.abandon(reason, ctx), ctx); }
			catch (error) { ctx.ui.notify(`Nothing was abandoned: ${String(error)}`, "error"); }
		},
	});
}

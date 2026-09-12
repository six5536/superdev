// One persistent worker process hosting a single pinned-SDK session.
//
// The worker holds no workflow authority: it has no controller capability, so
// it cannot approve documents, claim ownership, or record durable progress.
// The controller owns all of that and speaks to this process over Node's IPC
// channel, which is separate from stdout and therefore cannot be imitated by
// ordinary command output.
import { readFile } from "node:fs/promises";
import { join, resolve as resolvePath } from "node:path";
import { fileURLToPath } from "node:url";
import {
	createAgentSession, DefaultResourceLoader, ModelRuntime, SessionManager, SettingsManager,
	type ExtensionAPI, type ExtensionFactory,
} from "@earendil-works/pi-coding-agent";
import { registerAskTool } from "./ask-tool.ts";
import type { Question } from "./questions.ts";

/**
 * Marks a process as the workflow's worker.
 *
 * The controller extension is discovered inside the worker like any other
 * project extension. It stands aside when it sees this, so the worker gains no
 * workflow-authority surface and no tool is registered twice in one process.
 */
export const WORKER_MARKER = "SUPERDEV_WORKFLOW_WORKER";

/** Stages the worker executes, in order. */
export type WorkerStage = "implementation" | "verification" | "review" | "acceptance";

/**
 * Tools each stage may use.
 *
 * Review and acceptance keep `bash`, because a reviewer that cannot run a diff,
 * a search, or a type-checker is reviewing by reading alone. What stops them
 * changing the candidate is not a shorter tool list — a shell reaches any
 * effect through a variable, a script, or a build tool — but the controller
 * comparing the checkout's Git state before and after the stage.
 *
 * `write` and `edit` stay out: they serve no purpose in an assessment that
 * bash cannot serve outside the checkout, so offering them only invites an
 * accident that would void the stage.
 */
export const stageTools: Record<WorkerStage, string[]> = {
	implementation: ["read", "write", "edit", "bash", "superdev_ask"],
	verification: ["read", "write", "edit", "bash", "superdev_ask"],
	review: ["read", "bash", "sokf_search", "sokf_graph", "superdev_ask"],
	acceptance: ["read", "bash", "sokf_search", "sokf_graph", "superdev_ask"],
};
/** Bounded wait for a running turn to reach its own boundary after a disconnect. */
export const SETTLE_MS = 30_000;

/** Worker startup facts. `extensions` loads additional local extension modules. */
export type WorkerConfig = {
	cwd: string;
	agentDir: string;
	sessionsDir: string;
	/** Existing session file; the same session is reused across stages and restarts. */
	sessionFile?: string;
	/** Extra extension module paths, for offline or project-specific capability. */
	extensions?: string[];
	/** Skip model network access; the model must already be resolvable locally. */
	offline?: boolean;
};
/** A human reply, carrying the control the human chose rather than only text. */
export type QuestionReply = {
	status: "answered" | "discuss" | "other-action" | "paused";
	answer?: string;
	choiceId?: string;
};
export type ToWorker =
	| { type: "run"; id: string; stage: WorkerStage; instructions: string; context: string }
	| { type: "reset"; id: string; anchor: string }
	| { type: "reply"; id: string; reply: QuestionReply }
	| { type: "shutdown"; id: string };
export type FromWorker =
	| { type: "ready"; session: string; sessionFile?: string; anchor: string }
	| { type: "question"; id: string; question: Question }
	| { type: "activity"; text: string }
	| { type: "done"; id: string; text: string }
	| { type: "failed"; id: string; diagnostic: string }
	| { type: "stopped" };

/**
 * Report to the controller, if it is still listening.
 *
 * A disconnected controller is an expected end of the conversation, not a
 * worker fault, so the worker reports it on stderr and keeps its own durable
 * files intact instead of dying with an unhandled channel error.
 */
const send = (message: FromWorker) => {
	const report = (reason: unknown) =>
		process.stderr.write(`superdev worker: ${message.type} was not delivered: ${String(reason)}\n`);
	if (!process.connected) { report("controller disconnected"); return; }
	// A closed pipe is reported to this callback. Without it, Node emits an
	// unhandled `error` event on the process and the worker dies mid-turn —
	// exactly the abrupt loss the pause is meant to avoid.
	try { process.send?.(message, undefined, undefined, (error) => { if (error) report(error); }); }
	catch (error) { report(error); }
};

/** Build the worker session. Exported so tests drive the real production path. */
export async function startWorker(config: WorkerConfig) {
	const pending = new Map<string, (reply: QuestionReply) => void>();
	let sequence = 0;
	// The stage owns the worker's instructions and tools. It is set before each
	// turn, so a reset leaves the next stage its own context rather than none.
	let stage: WorkerStage = "implementation";
	let context = "";
	let api: ExtensionAPI | undefined;
	const stageFactory: ExtensionFactory = (pi: ExtensionAPI) => {
		api = pi;
		// A reset empties the conversation, so the stage checklist and evidence
		// are injected on every turn rather than carried in the transcript.
		pi.on("before_agent_start", () => context
			? { message: { customType: "superdev-stage", content: context, display: false } }
			: undefined);
		// The worker never renders a dialog. Every question reaches the human
		// through the controller, which alone can receive interactive input.
		registerAskTool(pi, async (question: Question) => {
			const id = `worker-question-${sequence++}`;
			return new Promise<QuestionReply>((resolve) => {
				pending.set(id, resolve);
				send({ type: "question", id, question });
			});
		});
	};
	const settingsManager = SettingsManager.inMemory({ compaction: { enabled: false, keepRecentTokens: 1 } });
	const resourceLoader = new DefaultResourceLoader({
		cwd: config.cwd, agentDir: config.agentDir, settingsManager,
		extensionFactories: [stageFactory],
		additionalExtensionPaths: config.extensions,
	});
	await resourceLoader.reload();
	const failures = resourceLoader.getExtensions().errors;
	if (failures.length) throw new Error(`Worker extensions failed to load: ${JSON.stringify(failures)}`);
	const modelRuntime = await ModelRuntime.create({
		authPath: join(config.agentDir, "auth.json"),
		modelsPath: join(config.agentDir, "models.json"),
		modelsStorePath: join(config.agentDir, "models-store.json"),
		allowModelNetwork: !config.offline,
	});
	const sessionManager = config.sessionFile
		? SessionManager.open(config.sessionFile)
		: SessionManager.create(config.cwd, config.sessionsDir);
	const { session } = await createAgentSession({
		cwd: config.cwd, agentDir: config.agentDir, modelRuntime, resourceLoader, settingsManager, sessionManager,
	});
	// Worker prompts are never interactive input, so nothing the worker sends can
	// be mistaken for a human approval in the controller.
	await session.bindExtensions({ mode: "print", onError: (error) => send({ type: "failed", id: "extension", diagnostic: `${error.event}: ${error.error}` }) });

	/** Apply a stage's instructions and tools. Unknown tool names are ignored. */
	const enter = (next: WorkerStage, stageContext: string) => {
		stage = next;
		context = stageContext;
		if (!api) return;
		const available = new Set(api.getAllTools().map((tool) => tool.name));
		api.setActiveTools(stageTools[stage].filter((name) => available.has(name)));
	};
	return { session, pending, enter, stage: () => stage, activeTools: () => api?.getActiveTools() ?? [] };
}

async function main() {
	const [configPath] = process.argv.slice(2);
	if (!configPath) throw new Error("Worker requires its configuration path");
	const config: WorkerConfig = JSON.parse(await readFile(configPath, "utf8"));
	const { session, pending, enter } = await startWorker(config);
	const anchor = session.sessionManager.appendCustomEntry("superdev-worker-root", {});
	let busy = false;
	let stopping = false;
	process.on("message", (message: ToWorker) => {
		void handle(message).catch((error) => send({ type: "failed", id: message.id, diagnostic: String(error) }));
	});
	// A lost controller asks for a pause rather than cutting the work off: the
	// running turn reaches its own boundary first, so its partial work and any
	// commits it made are left in a state the next controller can inspect.
	process.on("disconnect", () => {
		stopping = true;
		void settle().finally(() => { session.dispose(); process.exit(0); });
	});

	async function settle() {
		const deadline = Date.now() + SETTLE_MS;
		while (busy && Date.now() < deadline) await new Promise((wait) => setTimeout(wait, 100));
		// A turn still running at the deadline is stopped, because waiting
		// forever would leave an unreachable process holding the checkout.
		if (busy) await session.abort().catch(() => {});
	}

	async function handle(message: ToWorker) {
		if (message.type === "reply") {
			const resolve = pending.get(message.id);
			pending.delete(message.id);
			// A reply for an unknown question is discarded rather than applied to
			// whatever happens to be pending now.
			resolve?.(message.reply);
			return;
		}
		if (message.type === "shutdown") {
			await session.abort();
			send({ type: "stopped" });
			session.dispose();
			process.disconnect?.();
			return;
		}
		if (stopping) { send({ type: "failed", id: message.id, diagnostic: "Worker is stopping after losing its controller" }); return; }
		if (busy) { send({ type: "failed", id: message.id, diagnostic: "Worker is already running a stage turn" }); return; }
		busy = true;
		try {
			if (message.type === "reset") {
				// Reset only at a settled boundary. Pi refuses navigation while a
				// tool is pending, so an interrupted turn cannot be erased.
				await session.waitForIdle();
				const result = await session.navigateTree(message.anchor, { summarize: false, label: "superdev-stage-reset" });
				if (result.cancelled) throw new Error("Context reset was cancelled; the stage remains where it was");
				send({ type: "done", id: message.id, text: "" });
				return;
			}
			enter(message.stage, message.context);
			await session.prompt(message.instructions, { source: "extension" });
			await session.waitForIdle();
			const last = session.messages.findLast((entry) => entry.role === "assistant");
			const text = last?.content.filter((part) => part.type === "text").map((part) => part.text).join("\n") ?? "";
			send({ type: "done", id: message.id, text });
		} finally { busy = false; }
	}

	send({ type: "ready", session: session.sessionId, sessionFile: session.sessionFile, anchor });
}

// Run only when this module is the process entry point, never when the
// controller imports its types.
if (process.argv[1] && resolvePath(process.argv[1]) === fileURLToPath(import.meta.url)) {
	await main();
}

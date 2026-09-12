// Controller-side ownership of at most one persistent worker process.
//
// The controller stays responsive while the worker executes. Only one of them
// writes the checkout at a time, and the durable record — not this object —
// remains the authority for progress, approvals, and consumed retries.
import { fork, type ChildProcess } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import { killProcessTree } from "./service-exec.ts";
import { WORKER_MARKER, type FromWorker, type QuestionReply, type ToWorker, type WorkerConfig, type WorkerStage } from "./worker-host.ts";
import type { Question } from "./questions.ts";

const here = dirname(fileURLToPath(import.meta.url));
/** Bounded grace for an orderly stop before the process tree is terminated. */
export const SHUTDOWN_GRACE_MS = 10_000;
/** Controller options. The grace period is injectable so tests can prove termination. */
export type WorkerOptions = { graceMs?: number };
/** Identity a controller must record before the worker can be recovered. */
export type WorkerIdentity = { session: string; sessionFile?: string; anchor?: string };
export type WorkerHooks = {
	/**
	 * Route one worker question to the human through the controller's interface.
	 *
	 * The reply carries the control the human chose, so Discuss and Pause reach
	 * the worker as themselves rather than as an answer they never gave.
	 */
	ask(question: Question, ctx: ExtensionContext): Promise<QuestionReply>;
	/** Report worker activity without starting a model turn. */
	activity?(text: string): void;
};

/** One worker process. A stopped worker is never assumed to have finished its stage. */
export class WorkerSession {
	private child?: ChildProcess;
	private configDirectory?: string;
	private identity?: WorkerIdentity;
	private sequence = 0;
	private readonly waiting = new Map<string, { resolve: (text: string) => void; reject: (error: Error) => void }>();
	private exit?: { code: number | null; signal: NodeJS.Signals | null };

	private readonly graceMs: number;

	constructor(private readonly hooks: WorkerHooks, options: WorkerOptions = {}) {
		this.graceMs = options.graceMs ?? SHUTDOWN_GRACE_MS;
	}

	view() {
		return {
			running: this.running, pid: this.child?.pid, session: this.identity?.session,
			sessionFile: this.identity?.sessionFile, anchor: this.identity?.anchor,
			lastExit: this.exit,
		};
	}

	get running(): boolean {
		return Boolean(this.child && this.child.exitCode === null && this.child.signalCode === null);
	}

	/** Start, or reattach to, the one worker session for this workflow. */
	async start(config: Omit<WorkerConfig, "cwd">, ctx: ExtensionContext): Promise<WorkerIdentity> {
		if (this.running) throw new Error("A worker is already running for this checkout");
		this.exit = undefined;
		this.configDirectory = await mkdtemp(join(tmpdir(), "superdev-worker-"));
		const configPath = join(this.configDirectory, "worker.json");
		// The configuration is a private file, so a session path never appears
		// in an inspectable argument list.
		await writeFile(configPath, JSON.stringify({ ...config, cwd: ctx.cwd } satisfies WorkerConfig), { mode: 0o600 });
		const ready = Promise.withResolvers<WorkerIdentity>();
		const child = fork(join(here, "worker-host.ts"), [configPath], {
			cwd: ctx.cwd, detached: true, stdio: ["ignore", "pipe", "pipe", "ipc"],
			// The controller capability never reaches the worker environment, and
			// the marker makes the discovered controller extension stand aside.
			env: { ...process.env, SUPERDEV_UI_AUTHORITY: undefined, [WORKER_MARKER]: "1" } as NodeJS.ProcessEnv,
			execArgv: ["--experimental-transform-types", "--disable-warning=ExperimentalWarning"],
		});
		this.child = child;
		let diagnostic = "";
		child.stderr?.setEncoding("utf8");
		child.stderr?.on("data", (chunk: string) => { diagnostic = (diagnostic + chunk).slice(-8_192); });
		child.on("message", (message: FromWorker) => {
			if (message.type === "ready") {
				this.identity = { session: message.session, sessionFile: message.sessionFile, anchor: message.anchor };
				ready.resolve(this.identity);
				return;
			}
			if (message.type === "question") {
				void this.hooks.ask(message.question, ctx)
					.then((reply) => this.post({ type: "reply", id: message.id, reply }))
					// A failed question must not leave the worker waiting forever. It
					// is reported as unanswered, never as a fabricated answer.
					.catch((error) => this.post({ type: "reply", id: message.id,
						reply: { status: "other-action", answer: `The question was not put to the human: ${String(error)}` } }));
				return;
			}
			if (message.type === "activity") { this.hooks.activity?.(message.text); return; }
			const waiter = this.waiting.get(message.id);
			if (!waiter) return;
			this.waiting.delete(message.id);
			if (message.type === "done") waiter.resolve(message.text);
			else if (message.type === "failed") waiter.reject(new Error(message.diagnostic));
		});
		child.on("exit", (code, signal) => {
			this.exit = { code, signal };
			this.child = undefined;
			const reason = new Error(`Worker stopped before finishing (code ${code ?? "none"}, signal ${signal ?? "none"}). Inspect the checkout and durable progress before continuing.${diagnostic ? `\n${diagnostic}` : ""}`);
			ready.reject(reason);
			for (const [id, waiter] of this.waiting) { this.waiting.delete(id); waiter.reject(reason); }
		});
		child.on("error", (error) => ready.reject(error));
		return ready.promise;
	}

	/**
	 * Run one stage turn under that stage's own instructions and tools.
	 *
	 * `context` carries the stage checklist and the evidence it needs, because a
	 * reset leaves the worker no conversation to recover them from. Rejection
	 * never means the stage completed.
	 */
	async run(stage: WorkerStage, instructions: string, context: string, signal?: AbortSignal): Promise<string> {
		return this.request({ type: "run", id: this.nextId(), stage, instructions, context }, signal);
	}

	/** Reset context at a settled boundary, to the worker's own root anchor. */
	async reset(signal?: AbortSignal): Promise<void> {
		const anchor = this.identity?.anchor;
		if (!anchor) throw new Error("The worker recorded no reset anchor; restart it from its saved session instead");
		await this.request({ type: "reset", id: this.nextId(), anchor }, signal);
	}

	/**
	 * Stop the worker: ask, wait a bounded time, then terminate the tree.
	 *
	 * Termination is how a stuck process is stopped, so the caller inspects the
	 * checkout afterwards rather than assuming the interrupted work finished.
	 */
	async stop(): Promise<"stopped" | "terminated" | "already-stopped"> {
		const child = this.child;
		if (!child) { await this.cleanup(); return "already-stopped"; }
		const stopped = new Promise<void>((resolve) => child.once("exit", () => resolve()));
		try { this.post({ type: "shutdown", id: this.nextId() }); } catch { /* already disconnected */ }
		let timer: NodeJS.Timeout | undefined;
		const graceful = await Promise.race([
			stopped.then(() => true),
			new Promise<false>((resolve) => { timer = setTimeout(() => resolve(false), this.graceMs); }),
		]);
		clearTimeout(timer);
		if (!graceful && child.pid) {
			killProcessTree(child.pid);
			await stopped;
		}
		await this.cleanup();
		return graceful ? "stopped" : "terminated";
	}

	private async cleanup() {
		const directory = this.configDirectory;
		this.configDirectory = undefined;
		if (directory) await rm(directory, { recursive: true, force: true });
	}

	private nextId() { return `stage-${this.sequence++}`; }

	private post(message: ToWorker) {
		if (!this.child?.connected) throw new Error("Worker is not connected");
		this.child.send(message);
	}

	private async request(message: ToWorker, signal?: AbortSignal): Promise<string> {
		if (!this.running) throw new Error("No worker is running; start or resume it first");
		if (signal?.aborted) throw new Error("Worker request was cancelled before it was sent");
		const result = Promise.withResolvers<string>();
		this.waiting.set(message.id, result);
		const cancel = () => {
			this.waiting.delete(message.id);
			// Cancellation stops waiting, not the worker: its partial work and
			// its own checkpointing continue until it is explicitly stopped.
			result.reject(new Error("Stopped waiting for the worker; it keeps running until it is paused"));
		};
		signal?.addEventListener("abort", cancel, { once: true });
		try {
			this.post(message);
			return await result.promise;
		} finally {
			this.waiting.delete(message.id);
			signal?.removeEventListener("abort", cancel);
		}
	}
}

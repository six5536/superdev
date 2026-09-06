import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";

const PROTOCOL_VERSION = "2025-06-18";
const MAX_RESPONSE_BYTES = 16 * 1024 * 1024;
const MAX_STDERR_BYTES = 64 * 1024;
const SHUTDOWN_GRACE_MS = 1000;

type JsonObject = Record<string, unknown>;
type Pending = {
	accept: (value: JsonObject) => void;
	reject: (error: Error) => void;
	removeAbort?: () => void;
};

export type McpToolResult = {
	content?: Array<{ type: string; text?: string }>;
	structuredContent?: unknown;
	isError?: boolean;
};

export type McpProcessOptions = {
	command?: string;
	args?: string[];
};

/** A narrow, serial MCP client for one repository's SOKF server. */
export class SokfMcpClient {
	private child?: ChildProcessWithoutNullStreams;
	private nextId = 1;
	private pending = new Map<number, Pending>();
	private stdout = "";
	private stderr = "";
	private initialized?: Promise<void>;
	private tail: Promise<void> = Promise.resolve();
	private closing = false;

	private readonly repository: string;
	private readonly processOptions: McpProcessOptions;

	constructor(repository: string, processOptions: McpProcessOptions = {}) {
		this.repository = repository;
		this.processOptions = processOptions;
	}

	callTool(name: string, args: JsonObject, signal?: AbortSignal): Promise<McpToolResult> {
		const run = this.tail.then(async () => {
			if (signal?.aborted) throw abortError();
			await this.ensureInitialized(signal);
			const result = await this.request("tools/call", { name, arguments: args }, signal);
			return result as McpToolResult;
		});
		this.tail = run.then(
			() => undefined,
			() => undefined,
		);
		return run;
	}

	async close(): Promise<void> {
		this.closing = true;
		const child = this.child;
		if (!child) return;
		const closed = new Promise<void>((accept) => child.once("close", () => accept()));
		child.stdin.end();
		const graceful = await Promise.race([
			closed.then(() => true),
			new Promise<false>((accept) => setTimeout(() => accept(false), SHUTDOWN_GRACE_MS)),
		]);
		if (!graceful && child.exitCode === null) {
			child.kill("SIGKILL");
			await closed;
		}
		this.child = undefined;
		this.initialized = undefined;
	}

	private async ensureInitialized(signal?: AbortSignal): Promise<void> {
		if (this.initialized) return this.initialized;
		this.start();
		this.initialized = (async () => {
			const result = await this.request(
				"initialize",
				{
					protocolVersion: PROTOCOL_VERSION,
					capabilities: {},
					clientInfo: { name: "superdev-pi-sokf", version: "1" },
				},
				signal,
			);
			if (result.protocolVersion !== PROTOCOL_VERSION) {
				throw new Error(
					`Incompatible SOKF MCP protocol: expected ${PROTOCOL_VERSION}, got ${String(result.protocolVersion)}`,
				);
			}
			this.notify("notifications/initialized");
		})().catch((error) => {
			this.terminate(asError(error));
			throw error;
		});
		return this.initialized;
	}

	private start(): void {
		if (this.closing) throw new Error("SOKF MCP client is closing");
		if (this.child) return;
		const command = this.processOptions.command ?? "superdev";
		const args = this.processOptions.args ?? ["mcp", "sokf"];
		const child = spawn(command, args, {
			cwd: this.repository,
			stdio: ["pipe", "pipe", "pipe"],
		});
		this.child = child;
		this.stdout = "";
		this.stderr = "";
		child.stdout.setEncoding("utf8");
		child.stderr.setEncoding("utf8");
		child.stdout.on("data", (chunk: string) => this.receive(chunk));
		child.stderr.on("data", (chunk: string) => {
			const bytes = Buffer.from(this.stderr + chunk, "utf8");
			this.stderr = bytes.subarray(Math.max(0, bytes.length - MAX_STDERR_BYTES)).toString("utf8");
		});
		child.on("error", (error) => this.terminate(error));
		child.on("close", (code, signal) => {
			if (this.child !== child) return;
			const suffix = this.stderr.trim();
			this.failPending(
				new Error(
					`SOKF MCP server exited ${code === null ? `on ${String(signal)}` : String(code)}${suffix ? `: ${suffix}` : ""}`,
				),
			);
			this.child = undefined;
			this.initialized = undefined;
		});
	}

	private receive(chunk: string): void {
		this.stdout += chunk;
		if (Buffer.byteLength(this.stdout, "utf8") > MAX_RESPONSE_BYTES && !this.stdout.includes("\n")) {
			this.terminate(new Error(`SOKF MCP response exceeded ${MAX_RESPONSE_BYTES} bytes`));
			return;
		}
		while (true) {
			const newline = this.stdout.indexOf("\n");
			if (newline < 0) return;
			const line = this.stdout.slice(0, newline).trim();
			this.stdout = this.stdout.slice(newline + 1);
			if (!line) continue;
			if (Buffer.byteLength(line, "utf8") > MAX_RESPONSE_BYTES) {
				this.terminate(new Error(`SOKF MCP response exceeded ${MAX_RESPONSE_BYTES} bytes`));
				return;
			}
			let message: JsonObject;
			try {
				message = JSON.parse(line) as JsonObject;
			} catch (error) {
				this.terminate(new Error(`SOKF MCP returned malformed JSON: ${String(error)}`));
				return;
			}
			if (typeof message.id !== "number") continue;
			const pending = this.pending.get(message.id);
			if (!pending) continue;
			this.pending.delete(message.id);
			pending.removeAbort?.();
			if (message.error && typeof message.error === "object") {
				const error = message.error as JsonObject;
				pending.reject(new Error(String(error.message ?? "SOKF MCP request failed")));
			} else if (message.result && typeof message.result === "object") {
				pending.accept(message.result as JsonObject);
			} else {
				pending.reject(new Error("SOKF MCP response carried neither result nor error"));
			}
		}
	}

	private request(method: string, params: JsonObject, signal?: AbortSignal): Promise<JsonObject> {
		const child = this.child;
		if (!child) return Promise.reject(new Error("SOKF MCP server is not running"));
		if (signal?.aborted) {
			const error = abortError();
			this.terminate(error);
			return Promise.reject(error);
		}
		const id = this.nextId++;
		return new Promise<JsonObject>((accept, reject) => {
			const onAbort = () => this.terminate(abortError());
			if (signal) signal.addEventListener("abort", onAbort, { once: true });
			this.pending.set(id, {
				accept,
				reject,
				removeAbort: signal ? () => signal.removeEventListener("abort", onAbort) : undefined,
			});
			child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`, (error) => {
				if (error) this.terminate(error);
			});
		});
	}

	private notify(method: string): void {
		this.child?.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method })}\n`);
	}

	private terminate(error: Error): void {
		const child = this.child;
		this.child = undefined;
		this.initialized = undefined;
		this.failPending(error);
		if (child && child.exitCode === null) child.kill();
	}

	private failPending(error: Error): void {
		for (const pending of this.pending.values()) {
			pending.removeAbort?.();
			pending.reject(error);
		}
		this.pending.clear();
	}
}

function abortError(): Error {
	const error = new Error("SOKF MCP request aborted");
	error.name = "AbortError";
	return error;
}

function asError(value: unknown): Error {
	return value instanceof Error ? value : new Error(String(value));
}

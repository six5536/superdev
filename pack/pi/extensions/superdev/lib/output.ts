import { createWriteStream, type WriteStream } from "node:fs";
import { chmod, mkdtemp, readdir, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import { truncateHead, truncateTail } from "@earendil-works/pi-coding-agent";

export type OutputPolicy = {
	timeoutSeconds?: number;
	maxContextBytes: number;
	maxContextLines: number;
	maxReviewStateBytes?: number;
	maxReviewFindings?: number;
	maxArtifactBytes: number;
	maxArtifacts: number;
	retentionHours: number;
};

function safeToken(value: string): string {
	return value.replace(/[^a-zA-Z0-9_.-]/g, "_").slice(0, 80) || "session";
}

class BoundedArtifactStream {
	readonly path: string;
	private readonly stream: WriteStream;
	private keptBytes = 0;
	private discardedBytes = 0;
	private streamError: unknown;

	constructor(path: string, private readonly limit: number) {
		this.path = path;
		this.stream = createWriteStream(path, { flags: "wx", mode: 0o600 });
		this.stream.on("error", (error) => { this.streamError = error; });
	}

	write(chunk: Buffer | string): boolean {
		const bytes = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
		if (this.streamError) {
			this.discardedBytes += bytes.length;
			return true;
		}
		const remaining = Math.max(0, this.limit - this.keptBytes);
		let writable = true;
		if (remaining) {
			const kept = bytes.subarray(0, remaining);
			writable = this.stream.write(kept);
			this.keptBytes += kept.length;
		}
		this.discardedBytes += Math.max(0, bytes.length - remaining);
		return writable;
	}

	onDrain(resume: () => void): void {
		const done = () => {
			this.stream.off("drain", done);
			this.stream.off("error", done);
			resume();
		};
		this.stream.once("drain", done);
		this.stream.once("error", done);
	}

	async finish(): Promise<void> {
		if (this.streamError) throw this.streamError;
		if (!this.stream.closed) {
			await new Promise<void>((resolve, reject) => {
				this.stream.once("error", reject);
				this.stream.once("finish", resolve);
				this.stream.end();
			});
		}
		if (this.streamError) throw this.streamError;
	}

	summary(): { bytes: number; discardedBytes: number; path: string } {
		return { bytes: this.keptBytes, discardedBytes: this.discardedBytes, path: this.path };
	}
}

export class IsolatedArtifact {
	readonly directory: string;
	readonly stdoutPath: string;
	readonly stderrPath: string;
	readonly resultPath: string;
	readonly diagnosticPath: string;
	private readonly stdout: BoundedArtifactStream;
	private readonly stderr: BoundedArtifactStream;
	private constructor(directory: string, private readonly limit: number) {
		this.directory = directory;
		this.stdoutPath = join(directory, "events.jsonl");
		this.stderrPath = join(directory, "stderr.log");
		this.resultPath = join(directory, "result.json");
		this.diagnosticPath = join(directory, "diagnostic.json");
		this.stdout = new BoundedArtifactStream(this.stdoutPath, limit);
		this.stderr = new BoundedArtifactStream(this.stderrPath, limit);
	}

	static async create(session: string, role: string, limit: number): Promise<IsolatedArtifact> {
		const directory = await mkdtemp(join(tmpdir(), `superdev-isolated-${safeToken(session)}-${safeToken(role)}-`));
		await chmod(directory, 0o700);
		return new IsolatedArtifact(directory, limit);
	}

	writeStdout(chunk: Buffer | string): boolean { return this.stdout.write(chunk); }
	onStdoutDrain(resume: () => void): void { this.stdout.onDrain(resume); }
	async finishStdout(): Promise<void> { await this.stdout.finish(); }
	stdoutSummary(): { bytes: number; discardedBytes: number; path: string } { return this.stdout.summary(); }
	writeStderr(chunk: Buffer | string): boolean { return this.stderr.write(chunk); }
	onStderrDrain(resume: () => void): void { this.stderr.onDrain(resume); }
	async finishStderr(): Promise<void> { await this.stderr.finish(); }

	async writeDiagnostic(value: unknown): Promise<number> {
		const bytes = Buffer.from(JSON.stringify(value, null, 2));
		const diagnosticLimit = Math.min(this.limit, 65_536);
		if (bytes.length > diagnosticLimit) throw new Error(`isolated-output-overflow: diagnostic exceeded ${diagnosticLimit} bytes`);
		await writeFile(this.diagnosticPath, bytes, { flag: "wx", mode: 0o600 });
		return bytes.length;
	}

	async writeResult(value: unknown): Promise<number> {
		const bytes = Buffer.from(JSON.stringify(value));
		if (bytes.length > this.limit) throw new Error(`isolated-output-overflow: final result exceeded ${this.limit} bytes`);
		await writeFile(this.resultPath, bytes, { flag: "wx", mode: 0o600 });
		return bytes.length;
	}

	stderrSummary(): { bytes: number; discardedBytes: number; path: string } {
		return this.stderr.summary();
	}
}

export function boundedText(text: string, policy: OutputPolicy, direction: "head" | "tail" = "tail", artifactPath?: string): string {
	const result = direction === "head"
		? truncateHead(text, { maxBytes: policy.maxContextBytes, maxLines: policy.maxContextLines })
		: truncateTail(text, { maxBytes: policy.maxContextBytes, maxLines: policy.maxContextLines });
	if (!result.truncated) return result.content;
	const location = artifactPath ? ` Full output saved to: ${artifactPath}` : "";
	return `${result.content}\n\n[Showing ${result.outputLines} of ${result.totalLines} lines (${result.outputBytes} of ${result.totalBytes} bytes).${location}]`;
}

export async function cleanupArtifacts(session: string, policy: OutputPolicy): Promise<void> {
	const prefix = `superdev-isolated-${safeToken(session)}-`;
	const entries = (await readdir(tmpdir(), { withFileTypes: true }))
		.filter((entry) => entry.isDirectory() && entry.name.startsWith(prefix));
	const records = await Promise.all(entries.map(async (entry) => {
		const path = join(tmpdir(), entry.name);
		return { path, name: basename(path), modified: (await stat(path)).mtimeMs };
	}));
	const cutoff = Date.now() - policy.retentionHours * 60 * 60 * 1_000;
	records.sort((left, right) => right.modified - left.modified);
	await Promise.all(records.map(async (record, index) => {
		if (record.modified < cutoff || index >= policy.maxArtifacts) await rm(record.path, { recursive: true, force: true });
	}));
}

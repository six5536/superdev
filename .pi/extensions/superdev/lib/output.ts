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

export class IsolatedArtifact {
	readonly directory: string;
	readonly stderrPath: string;
	readonly resultPath: string;
	private stream: WriteStream;
	private stderrBytes = 0;
	private discardedStderrBytes = 0;
	private streamError: unknown;
	private readonly limit: number;
	private constructor(directory: string, limit: number) {
		this.directory = directory;
		this.limit = limit;
		this.stderrPath = join(directory, "stderr.log");
		this.resultPath = join(directory, "result.json");
		this.stream = createWriteStream(this.stderrPath, { flags: "wx", mode: 0o600 });
		this.stream.on("error", (error) => { this.streamError = error; });
	}

	static async create(session: string, role: string, limit: number): Promise<IsolatedArtifact> {
		const directory = await mkdtemp(join(tmpdir(), `superdev-isolated-${safeToken(session)}-${safeToken(role)}-`));
		await chmod(directory, 0o700);
		return new IsolatedArtifact(directory, limit);
	}

	writeStderr(chunk: Buffer | string): boolean {
		const bytes = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
		if (this.streamError) {
			this.discardedStderrBytes += bytes.length;
			return true;
		}
		const remaining = Math.max(0, this.limit - this.stderrBytes);
		let writable = true;
		if (remaining) {
			const kept = bytes.subarray(0, remaining);
			writable = this.stream.write(kept);
			this.stderrBytes += kept.length;
		}
		this.discardedStderrBytes += Math.max(0, bytes.length - remaining);
		return writable;
	}

	onStderrDrain(resume: () => void): void {
		const done = () => {
			this.stream.off("drain", done);
			this.stream.off("error", done);
			resume();
		};
		this.stream.once("drain", done);
		this.stream.once("error", done);
	}

	async finishStderr(): Promise<void> {
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

	async writeResult(value: unknown): Promise<number> {
		const bytes = Buffer.from(JSON.stringify(value));
		if (bytes.length > this.limit) throw new Error(`isolated-output-overflow: final result exceeded ${this.limit} bytes`);
		await writeFile(this.resultPath, bytes, { flag: "wx", mode: 0o600 });
		return bytes.length;
	}

	stderrSummary(): { bytes: number; discardedBytes: number; path?: string } {
		return {
			bytes: this.stderrBytes,
			discardedBytes: this.discardedStderrBytes,
			...(this.stderrBytes ? { path: this.stderrPath } : {}),
		};
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

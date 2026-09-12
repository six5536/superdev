import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, mkdtemp, open, rm, type FileHandle } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

export function killProcessTree(pid: number) {
	if (process.platform === "win32") {
		const result = spawnSync(`${process.env.SystemRoot ?? "C:\\Windows"}\\System32\\taskkill.exe`, ["/F", "/T", "/PID", String(pid)], { windowsHide: true, stdio: "ignore" });
		if (result.error) throw result.error;
		return;
	}
	try { process.kill(-pid, "SIGKILL"); }
	catch (error) {
		if ((error as NodeJS.ErrnoException).code !== "ESRCH") throw error;
		try { process.kill(pid, "SIGKILL"); }
		catch (error) { if ((error as NodeJS.ErrnoException).code !== "ESRCH") throw error; }
	}
}

export function stopProcess(child: ChildProcess) { if (child.pid) killProcessTree(child.pid); }

/** Execute a verified private native snapshot, never a mutable checkout launcher. */
export async function runPinnedSuperdev(path: string, digest: string | undefined, args: string[], cwd: string,
	signal?: AbortSignal, environment?: Record<string, string>, input?: string) {
	if (process.platform !== "linux") throw new Error("Pinned service execution requires Linux file descriptors");
	if (signal?.aborted) throw new Error("Service execution was cancelled before startup");
	const source = await open(path, "r");
	let bytes: Buffer;
	try { bytes = await source.readFile(); } finally { await source.close(); }
	const observed = createHash("sha256").update(bytes).digest("hex");
	if (digest && observed !== digest) throw new Error("Service executable changed after it was pinned");
	const directory = await mkdtemp(join(tmpdir(), "superdev-exec-"));
	const privatePath = join(directory, "service");
	let executable: FileHandle | undefined;
	try {
		const writer = await open(privatePath, "wx", 0o600);
		try { await writer.writeFile(bytes); await writer.sync(); } finally { await writer.close(); }
		await chmod(privatePath, 0o500);
		executable = await open(privatePath, "r");
		await rm(privatePath);
		const result = await new Promise<{ code: number; stdout: string; stderr: string }>((accept, reject) => {
			const env = { ...process.env };
			delete env.SUPERDEV_UI_AUTHORITY;
			Object.assign(env, environment);
			const child = spawn("/proc/self/fd/3", args, { cwd, shell: false, detached: true,
				stdio: ["pipe", "pipe", "pipe", executable!.fd], env });
			let stdout = "", stderr = "", failure: Error | undefined;
			const stop = () => { if (child.pid) killProcessTree(child.pid); };
			const abort = () => { failure = new Error("Service execution cancelled; inspect durable state before retrying"); stop(); };
			const timeout = setTimeout(() => { failure = new Error("Service deadline exceeded; inspect durable state before retrying"); stop(); }, 120_000);
			const cleanup = () => { clearTimeout(timeout); signal?.removeEventListener("abort", abort); };
			child.stdout.setEncoding("utf8"); child.stderr.setEncoding("utf8");
			child.stdout.on("data", (chunk: string) => {
				if (failure) return;
				stdout += chunk;
				if (Buffer.byteLength(stdout) > 4 * 1024 * 1024) { failure = new Error("Service response exceeded its output limit"); stop(); }
			});
			child.stderr.on("data", (chunk: string) => { stderr = (stderr + chunk).slice(-8_192); });
			child.stdin.on("error", (error) => { failure = new Error("Service request pipe failed", { cause: error }); stop(); });
			child.on("error", (error) => { cleanup(); reject(error); });
			child.on("close", (code) => { cleanup(); failure ? reject(failure) : accept({ code: code ?? 2, stdout, stderr }); });
			signal?.addEventListener("abort", abort, { once: true });
			child.stdin.end(input);
		});
		return { ...result, digest: observed };
	} finally {
		await executable?.close();
		await rm(directory, { recursive: true, force: true });
	}
}

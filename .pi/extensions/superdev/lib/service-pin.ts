import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { rmSync } from "node:fs";
import { chmod, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";
import { promisify } from "node:util";

/** Bootstrap a trusted PATH launcher once, then keep its native service independent
 * of checkout switches and cargo rebuilds for the lifetime of this Pi process. */
export async function pinService(launcher: string, cwd: string) {
	const environment = { ...process.env };
	delete environment.SUPERDEV_UI_AUTHORITY;
	const { stdout } = await promisify(execFile)(launcher, ["workflow", "status", "--json"], {
		cwd, env: environment, timeout: 120_000, maxBuffer: 1_048_576, encoding: "utf8",
	});
	const response = JSON.parse(stdout);
	if (response.protocol !== "superdev-workflow/v3") throw new Error(`Unsupported workflow service protocol: ${response.protocol}`);
	const executable = response.result?.executable;
	if (typeof executable !== "string" || !isAbsolute(executable)) throw new Error("Workflow status omitted its absolute native executable path");
	const bytes = await readFile(executable);
	if (bytes.subarray(0, 4).toString("hex") !== "7f454c46") throw new Error("Pinned workflow service must be a native Linux executable, not a launcher script");
	const directory = await mkdtemp(join(tmpdir(), "superdev-parent-"));
	const path = join(directory, "superdev");
	try {
		await writeFile(path, bytes, { flag: "wx", mode: 0o500 });
		await chmod(directory, 0o700);
	} catch (error) {
		await rm(directory, { recursive: true, force: true });
		throw error;
	}
	const cleanup = () => rmSync(directory, { recursive: true, force: true });
	process.once("exit", cleanup);
	return {
		path,
		digest: createHash("sha256").update(bytes).digest("hex"),
		dispose() { process.removeListener("exit", cleanup); cleanup(); },
	};
}

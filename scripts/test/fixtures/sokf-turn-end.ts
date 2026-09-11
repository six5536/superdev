import assert from "node:assert/strict";
import childProcess from "node:child_process";
import { EventEmitter } from "node:events";
import { mkdtempSync, mkdirSync, readFileSync, realpathSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { syncBuiltinESMExports } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import sokf, { findRepository } from "../../../.pi/extensions/sokf.ts";

// Exercise the real handler, substituting only the validator process so a
// write between validation and reporting is deterministic, not a timed race.
export default async function () {
	const base = realpathSync(mkdtempSync(join(tmpdir(), "sokf-turn-end-")));
	const roots = [join(base, "first"), join(base, "second")];
	for (const root of roots) {
		mkdirSync(join(root, "knowledge"), { recursive: true });
		writeFileSync(join(root, ".git"), "gitdir: /unused/worktree-pointer\n");
		writeFileSync(join(root, "knowledge", "finding.txt"), "");
	}
	const calls: Array<{ cwd: string; args: string[] }> = [];
	const messages: Array<{ message: any; options: any }> = [];
	const handlers = new Map<string, any>();
	let settleAfterFix = false;
	const originalSpawn = childProcess.spawn;
	childProcess.spawn = ((command: string, args: string[], options: any) => {
		assert.equal(command, "superdev");
		calls.push({ cwd: options.cwd, args });
		const child: any = new EventEmitter();
		child.stdout = new EventEmitter();
		child.stderr = new EventEmitter();
		child.stdout.setEncoding = child.stderr.setEncoding = () => {};
		child.stdin = { end() {} };
		queueMicrotask(() => {
			const path = join(options.cwd, "knowledge", "finding.txt");
			const report = readFileSync(path, "utf8");
			if (settleAfterFix && args.includes("--fix")) writeFileSync(path, "");
			child.stdout.emit("data", report);
			child.emit("close", report ? 1 : 0);
		});
		return child;
	}) as typeof childProcess.spawn;
	syncBuiltinESMExports();
	try {
		sokf({
			registerTool() {},
			on(name: string, handler: any) { handlers.set(name, handler); },
			sendMessage(message: any, options: any) { messages.push({ message, options }); },
		} as any);
		const end = (root: string) => handlers.get("turn_end")({}, { cwd: join(root, "knowledge") });
		const finding = (root: string, text: string) => writeFileSync(join(root, "knowledge", "finding.txt"), text);
		const first = roots[0];
		const second = roots[1];

		// A clean turn invokes exactly one repair pass, even without tool calls.
		await end(first);
		assert.deepEqual(calls.splice(0), [{ cwd: first, args: ["validate", "--fix"] }]);
		assert.equal(messages.length, 0);
		assert.equal(readFileSync(join(first, "knowledge", "finding.txt"), "utf8"), "");

		// Re-read the current tree: do not send a captured finding already fixed.
		finding(first, "knowledge/a.md:3: stale finding; add its target");
		settleAfterFix = true;
		await end(first);
		assert.deepEqual(calls.splice(0), [
			{ cwd: first, args: ["validate", "--fix"] },
			{ cwd: first, args: ["validate"] },
		]);
		assert.equal(messages.length, 0);
		settleAfterFix = false;

		finding(first, "knowledge/a.md:3: missing target; add its concept");
		await end(first);
		assert.equal(messages.at(-1)?.options.triggerTurn, true);
		assert.equal(messages.at(-1)?.message.display, true);
		finding(second, "knowledge/b.md:4: missing target; add its concept");
		await end(second);
		assert.equal(messages.at(-1)?.options.triggerTurn, true, "roots share follow-up state");
		await end(first);
		assert.notEqual(messages.at(-1)?.options.triggerTurn, true);

		// Symlink spellings of one root share both its process and its sequence.
		if (process.platform !== "win32") {
			const alias = join(base, "alias");
			symlinkSync(first, alias, "dir");
			assert.equal(findRepository(join(alias, "knowledge")), first);
			await end(alias);
			assert.notEqual(messages.at(-1)?.options.triggerTurn, true, "alias restarted the sequence");
		}
		// A nearer config does not beat a Git root; a non-Git config is a fallback.
		mkdirSync(join(first, "knowledge", ".superdev"));
		writeFileSync(join(first, "knowledge", ".superdev", "config.toml"), "");
		assert.equal(findRepository(join(first, "knowledge")), first);
		const configured = join(base, "configured");
		mkdirSync(join(configured, ".superdev"), { recursive: true });
		writeFileSync(join(configured, ".superdev", "config.toml"), "");
		assert.equal(findRepository(configured), configured);

		// The cap includes the heading, blank lines, and truncation notice.
		for (const report of ["knowledge/a.md:3: add target\n".repeat(220), "é".repeat(5_000)]) {
			finding(first, report);
			await end(first);
			const text = messages.at(-1)?.message.content;
			assert.ok(text.split("\n").length <= 200, `report carries ${text.split("\n").length} lines`);
			assert.ok(Buffer.byteLength(text) <= 8 * 1024);
			assert.match(text, /truncated.*superdev validate/);
		}
	} finally {
		childProcess.spawn = originalSpawn;
		syncBuiltinESMExports();
		await handlers.get("session_shutdown")?.();
		rmSync(base, { recursive: true, force: true });
	}
}

import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { promisify } from "node:util";
import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import { WorkflowClient, type ScopeRecord } from "../../../.pi/extensions/superdev/lib/client.ts";

const exec = promisify(execFile), binary = resolve(process.argv[2]);
const root = await mkdtemp(join(tmpdir(), "superdev-checkout-smoke-"));
const launcher = join(root, "superdev"), client = new WorkflowClient(launcher);
const ctx = { cwd: root, sessionManager: { getSessionId: () => "checkout-test" } } as ExtensionContext;
const issue = "issue-001-canonical-recovery", plan = "plan-042-canonical-recovery";
const git = (...args: string[]) => exec("git", args, { cwd: root });
const input = (event: string) => ({ session: "checkout-test", event, text: `Explicit fixture permission: ${event}` });
const change = (record: ScopeRecord, action: Record<string, unknown>) => client.apply({ operation: "change", id: record.id, expected_revision: record.revision, change: action }, ctx);
async function approve(record: ScopeRecord, kind: string, id: string) {
	const bytes = await readFile(join(root, `knowledge/${kind}s/open/${id}.md`));
	return change(record, { action: "approve", document: kind, expected_hash: createHash("sha256").update(bytes).digest("hex"), input: input(kind), indexes: [] });
}
try {
	await git("init", "-q", "-b", "trunk"); await git("config", "user.name", "Test");
	await git("config", "user.email", "test@example.com"); await git("config", "commit.gpgsign", "false");
	await exec(binary, ["init", "--no-frontend", "--no-code-index"], { cwd: root });
	await writeFile(launcher, `#!/bin/sh\nexec '${binary.replaceAll("'", "'\\''")}' "$@"\n`); await chmod(launcher, 0o700);
	await git("add", "-A"); await git("commit", "-qm", "initial scaffold");
	await client.status(root); // Pins the native executable, not the launcher.
	let record = await client.apply({ operation: "create", issue }, ctx);
	for (const [kind, id] of [["issue", issue], ["plan", plan]]) {
		await mkdir(join(root, `knowledge/${kind}s/open`), { recursive: true });
		const text = await readFile(new URL(`../../../crates/app/superdev/tests/fixtures/workflow-${kind}.md`, import.meta.url), "utf8");
		await writeFile(join(root, `knowledge/${kind}s/open/${id}.md`), text);
	}
	record = await approve(record, "issue", issue);
	record = await change(record, { action: "attach-plan", plan });
	record = await approve(record, "plan", plan);
	record = await change(record, { action: "start-build", mode: "current", input: input("startup") });
	await writeFile(launcher, "#!/bin/sh\necho obsolete-work-branch-service >&2\nexit 99\n");
	await git("add", "superdev"); await git("commit", "-qm", "old branch launcher");
	await client.apply({ operation: "pause", id: record.id }, ctx);
	await git("switch", "trunk");
	const defaultBefore = (await git("rev-parse", "trunk")).stdout.trim();
	const resumed = await client.apply({ operation: "resume", id: record.id }, ctx);
	assert.equal(resumed.work_branch, "work/001-canonical-recovery");
	assert.equal((await git("branch", "--show-current")).stdout.trim(), "trunk", "resume changed branches without permission");
	await git("switch", resumed.work_branch!);
	await assert.rejects(exec(launcher, ["workflow", "status", "--json"], { cwd: root }), (error: any) => error.code === 99);
	const status = await client.status(root);
	assert.equal(status.workflows[0].record.id, record.id);
	assert.equal(status.workflows[0].record.phase, "build");
	assert.equal((await git("rev-parse", "trunk")).stdout.trim(), defaultBefore);
	assert.equal((await git("status", "--porcelain")).stdout.trim(), "");
	await client.apply({ operation: "pause", id: record.id }, ctx);
	console.log("SUPERDEV_SERVICE_CHECKOUT_PASS");
} finally { await client.dispose(); await rm(root, { recursive: true, force: true }); }

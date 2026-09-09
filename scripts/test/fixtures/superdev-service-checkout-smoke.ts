import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { chmod, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, join, resolve } from "node:path";
import { promisify } from "node:util";
import { ensureParentService, runSuperdev } from "../../../.pi/extensions/superdev/lib/process.ts";

const exec = promisify(execFile);
const binary = resolve(process.argv[2]);
const root = await mkdtemp(join(tmpdir(), "superdev-checkout-smoke-"));
const authority = "0123456789abcdef0123456789abcdef";
const identity = ["--session", "checkout-test", "--issue", "issue-001-canonical-recovery",
	"--plan", "plan-042-canonical-recovery", "--work-branch", "work/001-canonical-recovery"];
const git = (...args: string[]) => exec("git", args, { cwd: root });
const service = async (...args: string[]): Promise<any> => runSuperdev(args, root, authority);
try {
	await git("init", "-q", "-b", "trunk");
	await git("config", "user.name", "Test");
	await git("config", "user.email", "test@example.com");
	await git("config", "commit.gpgsign", "false");
	await exec(binary, ["init", "--no-frontend", "--no-code-index"], { cwd: root });
	// A checkout-relative launcher is present when the Pi process initializes.
	const launcher = join(root, "superdev");
	await writeFile(launcher, `#!/bin/sh\nexec '${binary.replaceAll("'", "'\\''")}' "$@"\n`);
	await chmod(launcher, 0o700);
	process.env.PATH = `${root}${delimiter}${process.env.PATH ?? ""}`;
	await git("add", "-A");
	await git("commit", "-qm", "initial scaffold");
	const pinned = await ensureParentService(root);
	const issue = await readFile(new URL("../../../crates/app/superdev/tests/fixtures/workflow-issue.md", import.meta.url), "utf8");
	await mkdir(join(root, "knowledge/issues/open"), { recursive: true });
	await writeFile(join(root, "knowledge/issues/open/issue-001-canonical-recovery.md"), issue);
	await git("add", "knowledge");
	await git("commit", "-qm", "docs: file canonical recovery");
	const plan = await readFile(new URL("../../../crates/app/superdev/tests/fixtures/workflow-plan.md", import.meta.url), "utf8");
	await mkdir(join(root, "knowledge/plans/open"), { recursive: true });
	await writeFile(join(root, "knowledge/plans/open/plan-042-canonical-recovery.md"), plan);
	await service("workflow", "start", ...identity);
	// Emulate an older work branch whose launcher cannot serve the loaded adapter.
	await writeFile(launcher, "#!/bin/sh\necho obsolete-work-branch-service >&2\nexit 99\n");
	await git("add", "superdev");
	await git("commit", "-qm", "old branch launcher");
	await service("workflow", "cancel", "--session", "checkout-test");
	await git("switch", "trunk");
	const defaultBefore = (await git("rev-parse", "trunk")).stdout.trim();
	await service("workflow", "resume", ...identity);
	await assert.rejects(exec(launcher, ["workflow", "status", "--json"], { cwd: root }),
		(error: any) => error.code === 99);
	assert.deepEqual(await ensureParentService(root), pinned);
	const status = await service("workflow", "status", "--json");
	const baseline = (await git("rev-parse", "HEAD")).stdout.trim();
	assert.equal((await git("branch", "--show-current")).stdout.trim(), "work/001-canonical-recovery");
	assert.equal((await git("rev-parse", "trunk")).stdout.trim(), defaultBefore);
	assert.equal((await git("status", "--porcelain")).stdout.trim(), "");
	// An unchanged proposal must still publish an immutable checkpoint through the
	// same pinned service even though the branch-local launcher is unusable.
	const planPath = join(root, "knowledge/plans/open/plan-042-canonical-recovery.md");
	const planBefore = await readFile(planPath, "utf8");
	const checkpoint = await service("workflow", "commit", "--session", "checkout-test",
		"--expected-revision", status.result.owner.last_plan_revision,
		"--message", "docs(workflow): checkpoint scope proposal");
	assert.equal(checkpoint.result.revision, status.result.owner.last_plan_revision);
	assert.equal(await readFile(planPath, "utf8"), planBefore);
	assert.notEqual(checkpoint.result.commit, baseline);
	assert.equal((await git("diff", "--name-only", baseline, checkpoint.result.commit)).stdout.trim(), "");
	assert.equal((await git("status", "--porcelain")).stdout.trim(), "");
	assert.equal((await git("rev-parse", "trunk")).stdout.trim(), defaultBefore);
	await service("workflow", "cancel", "--session", "checkout-test");
	console.log("SUPERDEV_SERVICE_CHECKOUT_PASS");
} finally {
	await rm(root, { recursive: true, force: true });
}

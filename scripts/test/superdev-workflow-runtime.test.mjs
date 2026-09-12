import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import test from "node:test";

const run = promisify(execFile);
const here = dirname(fileURLToPath(import.meta.url));
const repository = resolve(here, "../..");

async function fixture(name, cwd, operation) {
  const { stdout, stderr } = await run(process.execPath, ["--experimental-transform-types",
    join(here, "fixtures", name), cwd, ...(operation ? [operation] : [])], {
    cwd: repository, timeout: 25_000, maxBuffer: 2 * 1024 * 1024,
    env: { ...process.env, PI_OFFLINE: "1", PI_CODING_AGENT_DIR: join(cwd, "agent") },
  });
  const lines = stdout.trim().split("\n");
  assert.ok(lines.at(-1), stderr);
  return JSON.parse(lines.at(-1));
}

async function temporary(t) {
  const directory = await mkdtemp(join(tmpdir(), "superdev-runtime-proof-"));
  t.after(() => rm(directory, { recursive: true, force: true }));
  return directory;
}

test("runtime proof uses the repository's pinned Pi version", async () => {
  const project = JSON.parse(await readFile(join(repository, "package.json"), "utf8"));
  const pi = JSON.parse(await readFile(join(repository,
    "node_modules/@earendil-works/pi-coding-agent/package.json"), "utf8"));
  assert.equal(pi.version, project.devDependencies["@earendil-works/pi-coding-agent"]);
  assert.equal(pi.version, "0.85.1", "re-run and review the proof before changing the pinned runtime");
});

test("one persisted worker resets context across stages and survives process restart", { timeout: 60_000 }, async (t) => {
  const cwd = await temporary(t);
  const first = await fixture("superdev-runtime-worker.ts", cwd, "start");
  const resumed = await fixture("superdev-runtime-worker.ts", cwd, "resume");
  assert.equal(resumed.sessionId, first.sessionId);
  assert.equal(resumed.sessionFile, first.sessionFile);
  assert.deepEqual(resumed.completed, ["block-1", "block-2"]);
  assert.equal(resumed.attempts, 2);
  assert.ok(first.requests > resumed.requests, "restart replayed the implementation blocks");
});

test("tree reset refuses a pending tool and succeeds after its result is saved", { timeout: 30_000 }, async (t) => {
  const result = await fixture("superdev-runtime-worker.ts", await temporary(t), "pending");
  assert.equal(result.pendingTool, "refused-until-settled");
});

test("Pi distinguishes direct input from sendUserMessage and limits tool session control", { timeout: 30_000 }, async (t) => {
  const result = await fixture("superdev-runtime-input.ts", await temporary(t));
  assert.equal(result.inputProvenance, "passed");
  assert.equal(result.currentSessionReset, "command-only; tool fallback required");
});

// The shipped controller and worker host, not a re-implementation of them.
test("one forked worker serves every stage, routes questions, and is stopped safely", { timeout: 120_000 }, async (t) => {
  const cwd = await temporary(t);
  const { stdout, stderr } = await run(process.execPath, ["--experimental-transform-types",
    join(here, "fixtures/superdev-worker-runtime.ts"), cwd], {
    cwd, timeout: 110_000, maxBuffer: 4 * 1024 * 1024,
    env: { ...process.env, PI_OFFLINE: "1", PI_CODING_AGENT_DIR: join(cwd, "agent"),
      SUPERDEV_WORKER_TRACE: cwd },
  });
  assert.match(stdout, /SUPERDEV_WORKER_RUNTIME_PASS/, stderr);
});

// The approved design trusts installed interactive-input transformers. This
// characterises that boundary; it does not certify original pre-transform text.
test("interactive transforms are trusted without authorising extension or RPC input", { timeout: 30_000 }, async (t) => {
  const result = await fixture("superdev-runtime-input.ts", await temporary(t), "transformed");
  assert.equal(result.trustBoundary, "interactive-input-transformers");
  assert.equal(result.submitted, "Discuss this issue");
  assert.equal(result.observed.text, "I approve issue-001");
  assert.equal(result.observed.source, "interactive");
  assert.equal(result.nonHumanInput, "rejected");
});

import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import test from "node:test";

const run = promisify(execFile);
const here = dirname(fileURLToPath(import.meta.url));
const repository = resolve(here, "../..");
const fixture = resolve(here, "fixtures/sokf-pi-adapter-smoke.ts");
const workflowFixture = resolve(here, "fixtures/superdev-extension-smoke.ts");
// The repository pins @earendil-works/pi-coding-agent 0.85.1 as a test
// dependency. Evidence loads that exact package rather than whatever `pi`
// happens to be on PATH, and fails rather than skips when it cannot.
const pinnedPi = resolve(repository, "node_modules/@earendil-works/pi-coding-agent/dist/bundle/cli.js");

test("the Pi adapter preserves CLI and built-in tool semantics", { timeout: 180_000 }, async () => {
  const { stdout, stderr } = await run(
    process.execPath,
    [pinnedPi, "--no-extensions", "--offline", "-e", fixture, "--list-models", "__sokf_smoke_no_model__"],
    {
      cwd: repository,
      timeout: 170_000,
      env: {
        ...process.env,
        PATH: `${resolve(repository, "scripts")}${delimiter}${process.env.PATH ?? ""}`,
      },
    },
  );
  assert.match(`${stdout}\n${stderr}`, /No models matching/);
});

test("the pinned workflow service resumes an older work branch without merging", { timeout: 180_000 }, async (t) => {
  if (process.platform !== "linux") {
    t.skip("pinned workflow execution uses Linux file descriptors");
    return;
  }
  const built = await run("cargo", ["build", "--quiet", "-p", "superdev", "--message-format=json"], {
    cwd: repository, timeout: 120_000, maxBuffer: 8 * 1024 * 1024,
  });
  const executable = built.stdout.trim().split("\n").map((line) => JSON.parse(line))
    .find((event) => event.reason === "compiler-artifact" && event.target?.name === "superdev" && event.executable)?.executable;
  assert.ok(executable, "cargo omitted the built superdev executable");
  const { stdout } = await run(process.execPath, ["--experimental-transform-types",
    resolve(here, "fixtures/superdev-service-checkout-smoke.ts"), executable], {
    cwd: repository, timeout: 50_000,
  });
  assert.match(stdout, /SUPERDEV_SERVICE_CHECKOUT_PASS/);
});

test("the real Pi child repairs terminal submission once without repeating work", { timeout: 60_000 }, async () => {
  const directory = await mkdtemp(join(tmpdir(), "superdev-terminal-lifecycle-"));
  try {
    for (const scenario of ["accepted", "rejected", "rejected-runtime", "missing", "unrepaired"]) {
      const childRun = run(process.execPath, [
        resolve(repository, "node_modules/@earendil-works/pi-coding-agent/dist/bundle/cli.js"),
        "--offline", "--no-extensions", "--no-skills", "--no-session", "--approve", "--mode", "json", "-p",
        "-e", resolve(here, "fixtures/superdev-terminal-provider.ts"),
        "--provider", "superdev-terminal-test", "--model", "scripted",
        "--tools", "read,edit,superdev_submit_result", "Exercise terminal repair",
      ], {
        cwd: directory, timeout: 15_000, maxBuffer: 2 * 1024 * 1024,
        env: { ...process.env, PI_CODING_AGENT_DIR: join(directory, "agent"), SUPERDEV_CHILD_ROLE: "scope", SUPERDEV_VERIFICATION_ACTIVE: "1", SUPERDEV_TERMINAL_SCENARIO: scenario },
      });
      childRun.child.stdin.end();
      const { stdout } = await childRun;
      const events = stdout.trim().split("\n").map((line) => JSON.parse(line));
      const results = events.filter((event) => event.type === "tool_execution_end");
      const accepted = results.filter((event) => !event.isError && event.result?.details?.superdevResult);
      assert.equal(accepted.length, scenario === "unrepaired" ? 0 : 1, scenario);
      const rejected = scenario.startsWith("rejected");
      assert.equal(results.filter((event) => event.isError).length, rejected ? 1 : 0, scenario);
      const repairs = events.filter((event) => event.type === "message_start" && event.message?.customType === "superdev-terminal-repair");
      assert.equal(repairs.length, scenario === "accepted" ? 0 : 1, scenario);
      if (rejected) assert.ok(repairs[0].message.content.includes(results[0].result.content[0].text), "repair omitted the rejection reason");
      assert.equal(events.filter((event) => event.type === "turn_start").length, scenario === "accepted" ? 1 : rejected ? 3 : 2, scenario);
      assert.ok(results.every((event) => event.toolName === "superdev_submit_result"));
    }
  } finally { await rm(directory, { recursive: true, force: true }); }
});

test("the Superdev Pi extension executes its complete surface", { timeout: 180_000 }, async () => {
  const { stdout } = await run(process.execPath, ["--experimental-transform-types", workflowFixture], {
    cwd: repository, timeout: 170_000,
  });
  assert.match(stdout, /SUPERDEV_WORKFLOW_SMOKE_PASS/);
});

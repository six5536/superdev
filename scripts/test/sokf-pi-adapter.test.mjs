import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { promisify } from "node:util";
import test from "node:test";

const run = promisify(execFile);
const here = dirname(fileURLToPath(import.meta.url));
const repository = resolve(here, "../..");
const fixture = resolve(here, "fixtures/sokf-pi-adapter-smoke.ts");
const workflowFixture = resolve(here, "fixtures/superdev-extension-smoke.ts");
// Each fixture imports the repository's pinned Pi package. Execute the
// factory explicitly: --list-models can exit without ever running it.
function fixtureArguments(path) {
  return ["--experimental-transform-types", "--input-type=module", "--eval",
    `const fixture = await import(${JSON.stringify(pathToFileURL(path).href)}); await fixture.default(); console.log("SOKF_FIXTURE_PASS");`];
}

test("the Pi adapter preserves CLI and built-in tool semantics", { timeout: 180_000 }, async () => {
  const { stdout, stderr } = await run(
    process.execPath,
    fixtureArguments(fixture),
    {
      cwd: repository,
      timeout: 170_000,
      env: {
        ...process.env,
        PATH: `${resolve(repository, "scripts")}${delimiter}${process.env.PATH ?? ""}`,
      },
    },
  );
  assert.match(stdout, /SOKF_FIXTURE_PASS/, stderr);
});

test("turn-end reports are fresh, bounded, and keyed by canonical root", { timeout: 30_000 }, async () => {
  const { stdout } = await run(process.execPath,
    fixtureArguments(resolve(here, "fixtures/sokf-turn-end.ts")),
    { cwd: repository, timeout: 25_000 });
  assert.match(stdout, /SOKF_FIXTURE_PASS/);
});

async function reportSession(scenario) {
  const directory = await mkdtemp(join(tmpdir(), "sokf-report-delivery-"));
  try {
    await mkdir(join(directory, ".git"));
    await mkdir(join(directory, "knowledge"));
    await writeFile(join(directory, "knowledge/manifest.sokf.yaml"), 'sokf: "0.1"\nname: report-test\n');
    await writeFile(join(directory, "knowledge/a.md"), "---\ntype: T\nid: alpha\n---\n[Missing](missing.md)\n");
    const child = run(process.execPath, [
      resolve(repository, "node_modules/@earendil-works/pi-coding-agent/dist/bundle/cli.js"),
      "--offline", "--no-extensions", "--no-skills", "--no-session", "--approve", "--mode", "json", "-p",
      "-e", resolve(here, "fixtures/sokf-report-provider.ts"),
      "--provider", "sokf-report-test", "--model", "scripted", "--tools", "read,write", "Check reporting",
    ], {
      cwd: directory, timeout: 50_000, maxBuffer: 2 * 1024 * 1024,
      env: { ...process.env, PI_CODING_AGENT_DIR: join(directory, "agent"), SOKF_REPORT_SCENARIO: scenario,
        PATH: `${resolve(repository, "scripts")}${delimiter}${process.env.PATH ?? ""}` },
    });
    child.child.stdin.end();
    const { stdout } = await child;
    return stdout.trim().split("\n").map((line) => JSON.parse(line));
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

test("Pi displays later reports immediately without starting another turn", { timeout: 60_000 }, async () => {
  const events = await reportSession("unresolved");
  assert.equal(events.filter((event) => event.type === "turn_start").length, 2,
    "only the first failure triggers another turn");
  const reports = events.filter((event) => event.type === "message_end"
    && event.message?.customType === "sokf-validation");
  assert.equal(reports.length, 2, "the second report is hidden until a new user prompt");
  assert.ok(reports.every((event) => event.message.display && event.message.content.includes("missing.md")));
});

test("the first report reaches Pi before further tool turns can make it stale", { timeout: 60_000 }, async () => {
  const events = await reportSession("repair");
  const report = events.findIndex((event) => event.type === "message_end"
    && event.message?.customType === "sokf-validation");
  const repair = events.findIndex((event) => event.type === "tool_execution_start" && event.toolName === "write");
  assert.ok(report >= 0 && repair > report, "the initial report arrived after the finding was repaired");
  assert.equal(events.filter((event) => event.type === "turn_start").length, 3,
    "a queued stale report triggered an extra turn after repair");
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

test("the Superdev Pi extension executes its complete surface", { timeout: 180_000 }, async () => {
  const { stdout } = await run(process.execPath, ["--experimental-transform-types", workflowFixture], {
    cwd: repository, timeout: 170_000,
  });
  assert.match(stdout, /SUPERDEV_WORKFLOW_SMOKE_PASS/);
});

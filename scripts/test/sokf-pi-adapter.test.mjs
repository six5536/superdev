import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { access } from "node:fs/promises";
import { delimiter, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import test from "node:test";

const run = promisify(execFile);
const here = dirname(fileURLToPath(import.meta.url));
const repository = resolve(here, "../..");
const fixture = resolve(here, "fixtures/sokf-pi-adapter-smoke.ts");
const workflowFixture = resolve(here, "fixtures/superdev-extension-smoke.ts");

async function commandExists(command) {
  const candidates = (process.env.PATH ?? "").split(delimiter);
  for (const candidate of candidates) {
    try {
      await access(resolve(candidate, command));
      return true;
    } catch {
      // Continue through PATH.
    }
  }
  return false;
}

test("the Pi adapter preserves CLI and built-in tool semantics", { timeout: 180_000 }, async (t) => {
  if (!(await commandExists("pi"))) {
    t.skip("pi is not installed");
    return;
  }

  const { stdout, stderr } = await run(
    "pi",
    ["--no-extensions", "--offline", "-e", fixture, "--list-models", "__sokf_smoke_no_model__"],
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

test("the Superdev Pi extension executes its complete surface", { timeout: 180_000 }, async () => {
  const { stdout } = await run(process.execPath, ["--experimental-transform-types", workflowFixture], {
    cwd: repository, timeout: 170_000,
  });
  assert.match(stdout, /SUPERDEV_WORKFLOW_SMOKE_PASS/);
});

import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { promisify } from "node:util";

const run = promisify(execFile);
const here = dirname(fileURLToPath(import.meta.url));
const repository = resolve(here, "../..");

test("real Pi and Rust run BUILD and ACCEPT with bounded commits and closure", { timeout: 240_000 }, async (t) => {
  if (process.platform !== "linux") { t.skip("native service pinning requires Linux"); return; }
  const build = await run("cargo", ["build", "--quiet", "-p", "superdev", "--message-format=json"], { cwd: repository, timeout: 120_000, maxBuffer: 8 * 1024 * 1024 });
  const executable = build.stdout.trim().split("\n").map((line) => JSON.parse(line))
    .find((event) => event.reason === "compiler-artifact" && event.target?.name === "superdev" && event.executable)?.executable;
  assert.ok(executable);
  const directory = await mkdtemp(join(tmpdir(), "superdev-execution-sdk-"));
  try {
    const { stdout } = await run(process.execPath, ["--experimental-transform-types", join(here, "fixtures/superdev-execution-sdk.ts"), directory], {
      cwd: repository, timeout: 180_000, maxBuffer: 4 * 1024 * 1024,
      env: { ...process.env, PATH: `${dirname(executable)}${delimiter}${process.env.PATH ?? ""}` },
    });
    assert.match(stdout, /SUPERDEV_EXECUTION_SDK_PASS/);
  } finally { await rm(directory, { recursive: true, force: true }); }
});

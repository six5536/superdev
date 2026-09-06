import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { SokfMcpClient } from "../../.pi/extensions/sokf-mcp.ts";

const here = dirname(fileURLToPath(import.meta.url));
const fake = resolve(here, "fixtures/sokf-mcp-fake.mjs");

async function setup() {
  const root = await mkdtemp(join(tmpdir(), "sokf-mcp-client-"));
  const repo = join(root, "repo");
  await mkdir(repo);
  return { root, repo, log: join(root, "starts.log"), marker: join(root, "marker") };
}

function client(repo, mode, log, marker = "") {
  return new SokfMcpClient(repo, { command: process.execPath, args: [fake, mode, log, marker] });
}

test("one initialized child serves serial tool calls and chunked messages", async () => {
  const { repo, log } = await setup();
  const mcp = client(repo, "chunked", log);
  const first = await mcp.callTool("sokf_search", { query: "one" });
  const second = await mcp.callTool("sokf_graph", {});
  assert.equal(first.content?.[0]?.text, "sokf_search:1");
  assert.equal(second.content?.[0]?.text, "sokf_graph:2");
  await mcp.close();
  assert.equal((await readFile(log, "utf8")).trim().split("\n").length, 1);
});

test("an incompatible protocol and malformed output fail clearly", async () => {
  const incompatible = await setup();
  const wrong = client(incompatible.repo, "incompatible", incompatible.log);
  await assert.rejects(wrong.callTool("sokf_graph", {}), /Incompatible SOKF MCP protocol/);
  await wrong.close();

  const malformed = await setup();
  const broken = client(malformed.repo, "malformed", malformed.log);
  await assert.rejects(broken.callTool("sokf_graph", {}), /malformed JSON/);
  await broken.close();
});

test("process diagnostics are bounded and the next call restarts after abort", async () => {
  const exited = await setup();
  const dead = client(exited.repo, "exit", exited.log);
  await assert.rejects(dead.callTool("sokf_graph", {}), (error) => {
    assert.match(error.message, /exited 7/);
    assert.ok(Buffer.byteLength(error.message) < 65 * 1024);
    return true;
  });
  await dead.close();

  const restarted = await setup();
  const hanging = client(restarted.repo, "hang-once", restarted.log, restarted.marker);
  const controller = new AbortController();
  const pending = hanging.callTool("sokf_search", { query: "hang" }, controller.signal);
  setTimeout(() => controller.abort(), 30);
  await assert.rejects(pending, { name: "AbortError" });
  const recovered = await hanging.callTool("sokf_search", { query: "works" });
  assert.equal(recovered.content?.[0]?.text, "sokf_search:1");
  await hanging.close();
  assert.equal((await readFile(restarted.log, "utf8")).trim().split("\n").length, 2);
});

test("an abort between initialization and a tool request restarts cleanly", async () => {
  const interrupted = await setup();
  const mcp = client(interrupted.repo, "normal", interrupted.log);
  let checks = 0;
  const signal = {
    get aborted() {
      checks += 1;
      return checks >= 3;
    },
    addEventListener() {},
    removeEventListener() {},
  };
  await assert.rejects(mcp.callTool("sokf_graph", {}, signal), { name: "AbortError" });
  const recovered = await mcp.callTool("sokf_graph", {});
  assert.equal(recovered.content?.[0]?.text, "sokf_graph:1");
  await mcp.close();
  assert.equal((await readFile(interrupted.log, "utf8")).trim().split("\n").length, 2);
});

test("different repositories receive different children", async () => {
  const first = await setup();
  const secondRepo = join(first.root, "other");
  await mkdir(secondRepo);
  const a = client(first.repo, "normal", first.log);
  const b = client(secondRepo, "normal", first.log);
  await Promise.all([a.callTool("sokf_graph", {}), b.callTool("sokf_graph", {})]);
  await Promise.all([a.close(), b.close()]);
  const starts = (await readFile(first.log, "utf8")).trim().split("\n");
  assert.equal(starts.length, 2);
  assert.notEqual(starts[0], starts[1]);
});

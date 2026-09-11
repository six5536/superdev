import assert from "node:assert/strict";
import { chmodSync, existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { checkRequest, checkRetrieval, checkSnapshot, guidanceFrom, identities, parseReply, score, snapshotInputs, validateFixtures } from "../sokf-search-eval.mjs";

test("search experiment validates its fixtures without making model calls", () => {
  const fixtures = JSON.parse(readFileSync(new URL("../../evals/sokf/search.json", import.meta.url), "utf8"));
  validateFixtures(fixtures);
  assert.throws(() => validateFixtures({ ...fixtures, cases: [...fixtures.cases, fixtures.cases[0]] }));
});

test("search guidance is taken from the shipped tool and request fields", () => {
  const source = readFileSync(new URL("../../crates/lib/superdev-core/src/sokf/mcp.rs", import.meta.url), "utf8");
  const guidance = guidanceFrom(source);
  for (const field of ["query", "limit", "types", "tags", "lifecycle"]) assert(guidance.includes(`pub ${field}:`));
  assert(guidance.includes("semantic"));
  // The first LLM trial caught an invented ADR type in the proposed example.
  // Keep that example tied to the actual document type, not its filename.
  const decision = readFileSync(new URL("../../knowledge/adrs/active/adr-055-sokf-does-not-intercept-file-tools.md", import.meta.url), "utf8");
  assert.match(decision, /^type: Decision$/m);
  assert(guidance.includes('["Decision"]'));
  assert.throws(() => guidanceFrom("no definitions"));
});

test("a successful CLI exit cannot hide a model error or malformed response", () => {
  const event = (message) => JSON.stringify({ type: "message_end", message: { role: "assistant", ...message } });
  assert.throws(() => parseReply(event({ stopReason: "error", errorMessage: "unsupported model", content: [] })), /unsupported model/);
  assert.throws(() => parseReply(""));
  assert.throws(() => parseReply(event({ stopReason: "stop", content: [{ type: "text", text: "not json" }] })));
  const reply = parseReply(event({ stopReason: "stop", content: [{ type: "text", text: '```json\n{"query":"testing"}\n```' }] }));
  assert.equal(reply.value.query, "testing");
});

test("paid malformed responses still contribute their reported usage", () => {
  let cost = 0;
  const stdout = JSON.stringify({ type: "message_end", message: {
    role: "assistant", stopReason: "stop", content: [{ type: "text", text: "invalid JSON" }],
    usage: { totalTokens: 12, cost: { total: 0.01 } },
  } });
  assert.throws(() => parseReply(stdout, (message) => { cost += message.usage.cost.total; }));
  assert.equal(cost, 0.01);
});

test("a frozen comparison detects knowledge, binary and configuration drift", () => {
  const directory = mkdtempSync(join(tmpdir(), "sokf-eval-snapshot-"));
  try {
    mkdirSync(join(directory, "knowledge"));
    mkdirSync(join(directory, ".superdev"));
    const inputs = ["knowledge/note.md", ".superdev/config.toml", "binary"];
    for (const path of inputs) writeFileSync(join(directory, path), "original");
    const original = snapshotInputs(directory, join(directory, "binary"));
    checkSnapshot(original, snapshotInputs(directory, join(directory, "binary")));
    for (const path of inputs) {
      writeFileSync(join(directory, path), "changed");
      assert.throws(() => checkSnapshot(original, snapshotInputs(directory, join(directory, "binary"))), /changed during the run/);
      writeFileSync(join(directory, path), "original");
    }
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test("the runner stops before search if a model call changes its configuration", { skip: process.platform === "win32" && "uses POSIX executable shims" }, () => {
  const directory = mkdtempSync(join(tmpdir(), "sokf-eval-drift-"));
  try {
    mkdirSync(join(directory, "knowledge"));
    mkdirSync(join(directory, ".superdev"));
    writeFileSync(join(directory, "knowledge/note.md"), "# Note\n");
    writeFileSync(join(directory, ".superdev/config.toml"), "# frozen\n");
    const programs = {
      pi: `import { writeFileSync } from 'node:fs';
        writeFileSync('.superdev/config.toml', '# changed');
        console.log(JSON.stringify({type:'message_end',message:{role:'assistant',model:'gpt-5.6-luna',stopReason:'stop',usage:{totalTokens:12,cost:{total:0.01}},content:[{type:'text',text:'{"query":"contract-002"}'}]}}));`,
      superdev: `import { writeFileSync } from 'node:fs'; writeFileSync('search-ran', 'unexpected');`,
    };
    for (const [name, source] of Object.entries(programs)) {
      writeFileSync(join(directory, name), `#!${process.execPath}\n${source}\n`);
      chmodSync(join(directory, name), 0o755);
    }
    const output = join(directory, "report.json");
    const child = spawnSync(process.execPath, [
      fileURLToPath(new URL("../sokf-search-eval.mjs", import.meta.url)), "--run",
      `--corpus=${directory}`, `--binary=${join(directory, "superdev")}`, `--output=${output}`,
    ], { env: { ...process.env, PATH: directory }, encoding: "utf8", timeout: 10_000 });
    assert.equal(child.status, 1, child.stderr);
    const report = JSON.parse(readFileSync(output, "utf8"));
    assert.match(report.error, /configHash changed during the run/);
    assert.equal(report.calls, 1);
    assert.equal(report.estimatedCost, 0.01);
    assert(!existsSync(join(directory, "search-ran")));
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test("retrieval cannot silently become lexical-only in a replay or midway through a run", () => {
  const report = {};
  const manifest = { model_id: "local-embedding", schema_version: 3 };
  checkRetrieval(report, manifest, null);
  assert.equal(report.retrievalModel, "local-embedding");
  assert.throws(() => checkRetrieval(report, { ...manifest, model_id: null }, null), /retrieval model changed/);
  assert.throws(() => checkRetrieval({}, manifest, { retrievalModel: "other-model" }), /retrieval model changed/);
});

test("requests stay bounded and do not accept invented parameter names", () => {
  checkRequest({ query: "contract-002", types: ["Contract"], lifecycle: null, limit: 8 });
  for (const request of [{ query: "" }, { query: "q", limit: 9 }, { query: "q", typo: true }, { query: "q", types: "Contract" }]) {
    assert.throws(() => checkRequest(request));
  }
});

test("evidence scoring distinguishes missing retrieval from correct abstention", () => {
  const found = identities("8 sections for `q`\n\nissue-096-sample — Description\n  issues/file.md:1-9  [Summary] text\n\nother — Other\n");
  assert.deepEqual(found, ["issue-096-sample", "other"]);
  assert.equal(score(["issue-096-sample"], found, "issue-096").selectedCorrectly, true);
  assert.equal(score(["issue-096-sample"], [], "issue-096").selectedCorrectly, false);
  assert.equal(score(["issue-096-sample"], [...found, "issue-096-ambiguous"], "issue-096").selectedCorrectly, false);
  assert.equal(score(["issue-096-sample"], [], null).selectedCorrectly, false);
  assert.equal(score([], found, null).selectedCorrectly, true);
  assert.equal(score([], found, "other").falseSelection, true);
});

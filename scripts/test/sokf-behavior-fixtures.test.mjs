import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const fixtureUrl = new URL("../../evals/sokf/behavioral.json", import.meta.url);
const fixture = JSON.parse(await readFile(fixtureUrl, "utf8"));

const requiredScenarios = [
  "known-project-question",
  "architecture-decision",
  "direct-concept-reference",
  "code-local-task",
  "concept-edit",
  "multi-step-knowledge-change",
  "knowledge-producing-code-change",
  "new-project-decision",
  "outward-facing-documentation",
  "format-question",
  "unknown-concept-id",
  "missing-sokf-store",
];

test("SOKF behavioral fixtures cover every planned scenario", () => {
  assert.equal(fixture.protocol, "sokf-behavior/v1");
  assert.equal(fixture.acceptance.trials, 3);
  assert.equal(fixture.acceptance.minimumMechanicalPassRate, 0.9);
  assert.deepEqual(fixture.acceptance.safetyCriticalScenarios, [
    "code-local-task",
    "concept-edit",
    "multi-step-knowledge-change",
    "knowledge-producing-code-change",
    "new-project-decision",
    "outward-facing-documentation",
    "format-question",
    "missing-sokf-store",
  ]);
  assert.deepEqual(
    fixture.scenarios.map((scenario) => scenario.id),
    requiredScenarios,
  );
  assert.ok(
    fixture.acceptance.safetyCriticalScenarios.every((id) => requiredScenarios.includes(id)),
    "acceptance names an unknown safety-critical scenario",
  );

  for (const scenario of fixture.scenarios) {
    assert.match(scenario.prompt, /\S/);
    assert.equal(scenario.sandbox, scenario.id === "missing-sokf-store"
      ? "repository-without-knowledge" : "repository-copy");
    assert.ok(scenario.expect.orderedCalls.length > 0, `${scenario.id} has no expected calls`);
    assert.ok(scenario.expect.outcomes.length > 0, `${scenario.id} has no expected outcomes`);
    for (const call of [...(scenario.expect.prerequisiteCalls ?? []), ...scenario.expect.orderedCalls]) {
      assert.match(call.tool, /^[a-z][a-z0-9_]*$/);
      if (call.path !== undefined) {
        const paths = Array.isArray(call.path) ? call.path : [call.path];
        assert.ok(paths.length > 0);
        for (const path of paths) {
          assert.match(path, /\S/);
          if (["read", "edit", "write"].includes(call.tool)) {
            assert.ok(!path.startsWith("sokf:"), `${scenario.id} routes a file tool: ${path}`);
          }
        }
      }
    }
  }
});

test("physical paths preserve direct addressing and schema-first editing", () => {
  const byId = new Map(fixture.scenarios.map((scenario) => [scenario.id, scenario]));
  const direct = byId.get("direct-concept-reference");
  assert.match(direct.prompt, /knowledge\/architecture\.md/);
  assert.deepEqual(direct.expect.orderedCalls, [{ tool: "read", path: "knowledge/architecture.md" }]);
  assert.ok(direct.expect.forbiddenTools.includes("sokf_search"));
  const edit = byId.get("concept-edit");
  assert.deepEqual(edit.expect.prerequisiteCalls, [
    { tool: "read", path: "knowledge/schemas/*.md" },
    { tool: "read", path: "knowledge/architecture.md" },
  ]);
  assert.deepEqual(edit.expect.orderedCalls, [{ tool: "edit", path: "knowledge/architecture.md" }]);
  assert.deepEqual(byId.get("multi-step-knowledge-change").expect.orderedCalls[0],
    { tool: "read", path: ["knowledge/schemas/*.md"] });
});

test("a missing knowledge path requires semantic recovery, not a filesystem scan", async () => {
  const scenario = fixture.scenarios.find(({ id }) => id === "unknown-concept-id");
  const missing = "knowledge/command-router-policy.md";
  assert.ok(scenario.prompt.includes(missing));
  await assert.rejects(readFile(new URL(`../../${missing}`, import.meta.url)), { code: "ENOENT" });
  assert.deepEqual(scenario.expect.orderedCalls, [
    { tool: "read", path: missing }, { tool: "sokf_search" },
  ]);
  assert.deepEqual(scenario.expect.outcomes, ["search-recovery", "no-broad-filesystem-scan"]);
  assert.deepEqual(scenario.expect.forbiddenTools, ["edit", "write"]);
});

test("evaluation instructions use AGENTS.md without duplicate prompt injection", async () => {
  const runner = await readFile(new URL("../sokf-eval.mjs", import.meta.url), "utf8");
  assert.ok(runner.includes('writeFileSync(join(destination, "AGENTS.md"), withoutInstruction ? instructionsWithoutKnowledge : agentInstructions)'));
  assert.ok(!runner.includes("@.agents/superdev.md"));
  assert.ok(!runner.includes("--append-system-prompt"));
});

test("fixtures distinguish SOKF-worthy work from a code-local task", () => {
  const byId = new Map(fixture.scenarios.map((scenario) => [scenario.id, scenario]));
  assert.equal(byId.get("known-project-question").expect.orderedCalls[0].tool, "sokf_search");
  assert.deepEqual(byId.get("code-local-task").expect.forbiddenTools, ["sokf_search", "sokf_graph"]);
  assert.equal(byId.get("format-question").expect.orderedCalls[0].path, ".agents/sokf/SPEC.md");
  assert.equal(byId.get("missing-sokf-store").sandbox, "repository-without-knowledge");
});

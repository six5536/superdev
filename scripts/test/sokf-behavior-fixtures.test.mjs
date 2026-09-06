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
  assert.deepEqual(
    fixture.scenarios.map((scenario) => scenario.id),
    requiredScenarios,
  );

  for (const scenario of fixture.scenarios) {
    assert.match(scenario.prompt, /\S/);
    assert.ok(["repository-copy", "repository-without-knowledge"].includes(scenario.sandbox));
    assert.ok(scenario.expect.orderedCalls.length > 0, `${scenario.id} has no expected calls`);
    assert.ok(scenario.expect.outcomes.length > 0, `${scenario.id} has no expected outcomes`);
    for (const call of scenario.expect.orderedCalls) {
      assert.match(call.tool, /^[a-z][a-z0-9_]*$/);
      if (call.path !== undefined) assert.match(call.path, /\S/);
    }
  }
});

test("fixtures distinguish SOKF-worthy work from a code-local task", () => {
  const byId = new Map(fixture.scenarios.map((scenario) => [scenario.id, scenario]));
  assert.equal(byId.get("known-project-question").expect.orderedCalls[0].tool, "sokf_search");
  assert.deepEqual(byId.get("code-local-task").expect.forbiddenTools, ["sokf_search", "sokf_graph"]);
  assert.equal(byId.get("format-question").expect.orderedCalls[0].path, ".agents/sokf/SPEC.md");
  assert.equal(byId.get("missing-sokf-store").sandbox, "repository-without-knowledge");
});

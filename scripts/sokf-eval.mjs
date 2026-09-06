#!/usr/bin/env node
import { spawn } from "node:child_process";
import { closeSync, cpSync, existsSync, mkdtempSync, mkdirSync, openSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
const repository = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const fixtures = JSON.parse(readFileSync(join(repository, "evals/sokf/behavioral.json"), "utf8"));
const agentInstructions = readFileSync(join(repository, ".agents/superdev.md"), "utf8");
const instructionsWithoutKnowledge = agentInstructions.replace(/<knowledge\b[\s\S]*?<\/knowledge>\n*/u, "");
if (instructionsWithoutKnowledge === agentInstructions) throw new Error("The standing SOKF instruction block is missing");
const arguments_ = new Set(process.argv.slice(2));
const execute = arguments_.has("--run");
const withoutInstruction = arguments_.has("--without-instruction");
const keepSandboxes = arguments_.has("--keep-sandboxes");
const selected = process.argv.find((argument) => argument.startsWith("--scenario="))?.slice("--scenario=".length);
const trials = Number(process.argv.find((argument) => argument.startsWith("--trials="))?.slice("--trials=".length) ?? "1");
if (!Number.isInteger(trials) || trials < 1) throw new Error("--trials must be a positive integer");
const provider = process.env.PI_PROVIDER;
const model = process.env.PI_MODEL;

function copyRepository(destination, withoutKnowledge) {
  cpSync(repository, destination, {
    recursive: true,
    filter(source) {
      const relative = source.slice(repository.length).replace(/^\//, "");
      if (!relative) return true;
      if ([".git", ".codegraph", "target", "node_modules", ".superdev/cache"].some((path) => relative === path || relative.startsWith(`${path}/`))) {
        return false;
      }
      if (relative === ".pi/current-system-prompt.md") return false;
      if (withoutKnowledge && (relative === "knowledge" || relative.startsWith("knowledge/"))) return false;
      return true;
    },
  });
  mkdirSync(join(destination, ".git"), { recursive: true });
  writeFileSync(join(destination, "AGENTS.md"), "@.agents/superdev.md\n");
}

function globMatches(pattern, value) {
  const expression = pattern
    .split("*")
    .map((part) => part.replace(/[|\\{}()[\]^$+?.]/g, "\\$&"))
    .join(".*");
  return new RegExp(`^${expression}$`).test(value);
}

function callMatches(expected, actual) {
  if (expected.tool !== actual.tool) return false;
  if (expected.path === undefined) return true;
  const patterns = Array.isArray(expected.path) ? expected.path : [expected.path];
  return typeof actual.args.path === "string" && patterns.some((pattern) => globMatches(pattern, actual.args.path));
}

function scoreScenario(scenario, calls) {
  let cursor = 0;
  const missing = [];
  const firstMutation = calls.findIndex((call) => call.tool === "edit" || call.tool === "write");
  const prerequisiteBoundary = firstMutation < 0 ? calls.length : firstMutation;
  for (const expected of scenario.expect.prerequisiteCalls ?? []) {
    if (!calls.some((call, index) => index < prerequisiteBoundary && callMatches(expected, call))) {
      missing.push(expected);
    }
  }
  for (const expected of scenario.expect.orderedCalls) {
    const found = calls.findIndex((call, index) => index >= cursor && callMatches(expected, call));
    if (found < 0) missing.push(expected);
    else cursor = found + 1;
  }
  const forbidden = calls.filter((call) => {
    if (scenario.expect.forbiddenTools?.includes(call.tool)) return true;
    const path = call.args.path;
    return typeof path === "string" && scenario.expect.forbiddenPaths?.some((pattern) => globMatches(pattern, path));
  });
  return { passed: missing.length === 0 && forbidden.length === 0, missing, forbidden };
}

function parseEvents(stdout) {
  const events = stdout
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line));
  const calls = events
    .filter((event) => event.type === "tool_execution_start")
    .map((event) => ({ tool: event.toolName, args: event.args ?? {} }));
  const assistantMessages = events.filter((event) => event.type === "message_end" && event.message?.role === "assistant");
  const usage = assistantMessages.reduce(
    (total, event) => {
      total.tokens += event.message.usage?.totalTokens ?? 0;
      total.cost += event.message.usage?.cost?.total ?? 0;
      return total;
    },
    { tokens: 0, cost: 0 },
  );
  const finalText = assistantMessages
    .at(-1)
    ?.message.content.filter((content) => content.type === "text")
    .map((content) => content.text)
    .join("\n");
  const toolErrors = events
    .filter((event) => event.type === "tool_execution_end" && event.result?.isError)
    .map((event) => ({ tool: event.toolName, text: event.result.content?.[0]?.text ?? "tool error" }));
  return {
    calls,
    usage,
    initialInputTokens: assistantMessages[0]?.message.usage?.input ?? 0,
    finalText: finalText ?? "",
    toolErrors,
  };
}

async function runPi(args, options, sandbox) {
  const stdoutPath = join(sandbox, ".sokf-evaluation.jsonl");
  const stderrPath = join(sandbox, ".sokf-evaluation.stderr");
  const stdout = openSync(stdoutPath, "w");
  const stderr = openSync(stderrPath, "w");
  try {
    const code = await new Promise((accept, reject) => {
      const child = spawn("pi", args, { ...options, stdio: ["ignore", stdout, stderr], timeout: 600_000 });
      child.on("error", reject);
      child.on("exit", accept);
    });
    if (code !== 0) {
      throw new Error(readFileSync(stderrPath, "utf8") || `pi exited ${String(code)}`);
    }
  } finally {
    closeSync(stdout);
    closeSync(stderr);
  }
  return readFileSync(stdoutPath, "utf8");
}

async function runScenario(scenario) {
  const sandbox = mkdtempSync(join(tmpdir(), `sokf-eval-${scenario.id}-`));
  copyRepository(sandbox, scenario.sandbox === "repository-without-knowledge");
  const args = [
    "--approve",
    "--no-session",
    "--no-extensions",
    "-e",
    join(sandbox, ".pi/extensions/sokf.ts"),
    "--no-skills",
    "--skill",
    join(sandbox, ".pi/skills/sokf-authoring"),
    "--mode",
    "json",
    "--tools",
    "read,edit,write,sokf_search,sokf_graph",
    "--thinking",
    "minimal",
  ];
  if (provider) args.push("--provider", provider);
  if (model) args.push("--model", model);
  args.push("--append-system-prompt", withoutInstruction ? instructionsWithoutKnowledge : agentInstructions);
  args.push("-p", scenario.prompt);

  try {
    const stdout = await runPi(
      args,
      {
        cwd: sandbox,
        env: {
          ...process.env,
          PATH: `${join(repository, "scripts")}${delimiter}${process.env.PATH ?? ""}`,
        },
      },
      sandbox,
    );
    const parsed = parseEvents(stdout);
    const calls = parsed.calls.map((call) => ({
      ...call,
      args: {
        ...call.args,
        path:
          typeof call.args.path === "string" && call.args.path.startsWith(`${sandbox}/`)
            ? call.args.path.slice(sandbox.length + 1)
            : call.args.path,
      },
    }));
    return {
      id: scenario.id,
      ...scoreScenario(scenario, calls),
      calls,
      usage: parsed.usage,
      initialInputTokens: parsed.initialInputTokens,
      finalText: parsed.finalText,
      toolErrors: parsed.toolErrors,
      manualOutcomes: scenario.expect.outcomes,
      sandbox: keepSandboxes ? sandbox : undefined,
    };
  } finally {
    if (!keepSandboxes) rmSync(sandbox, { recursive: true, force: true });
  }
}

const scenarios = fixtures.scenarios.filter((scenario) => !selected || scenario.id === selected);
if (selected && scenarios.length === 0) throw new Error(`Unknown scenario: ${selected}`);
if (!execute) {
  console.log(`Validated ${scenarios.length} ${fixtures.protocol} scenario(s). Add --run to execute model sessions.`);
  process.exit(0);
}
if (!existsSync(join(repository, ".pi/extensions/sokf.ts"))) throw new Error("SOKF Pi extension is missing");

const results = [];
for (const scenario of scenarios) {
  for (let trial = 1; trial <= trials; trial += 1) {
    process.stderr.write(`evaluate ${scenario.id} (${trial}/${trials})... `);
    const result = { ...(await runScenario(scenario)), trial };
    results.push(result);
    process.stderr.write(`${result.passed ? "PASS" : "FAIL"}\n`);
  }
}
const safetyCritical = new Set(fixtures.acceptance.safetyCriticalScenarios);
const mechanicalPassRate = results.filter((result) => result.passed).length / results.length;
const completeMatrix = !selected && trials >= fixtures.acceptance.trials;
const summary = {
  protocol: "sokf-evaluation-result/v1",
  fixtureProtocol: fixtures.protocol,
  instruction: withoutInstruction ? "without" : "with",
  provider: provider ?? null,
  model: model ?? null,
  trials,
  passed: results.filter((result) => result.passed).length,
  total: results.length,
  mechanicalPassRate,
  safetyCriticalPassed: results
    .filter((result) => safetyCritical.has(result.id))
    .every((result) => result.passed),
  thresholdEvaluated: completeMatrix,
  thresholdMet:
    completeMatrix &&
    mechanicalPassRate >= fixtures.acceptance.minimumMechanicalPassRate &&
    results.filter((result) => safetyCritical.has(result.id)).every((result) => result.passed),
  tokens: results.reduce((sum, result) => sum + result.usage.tokens, 0),
  cost: results.reduce((sum, result) => sum + result.usage.cost, 0),
  results,
};
console.log(JSON.stringify(summary, null, 2));

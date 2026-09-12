// Test-only offline provider for the real worker process.
//
// It registers a scripted model and selects it at session start, so the worker
// under test is the production worker, not a re-implementation of it.
import { appendFileSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { createAssistantMessageEventStream, type AssistantMessage, type Context, type Model } from "@earendil-works/pi-ai";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

const model: Model<"openai-completions"> = {
	id: "scripted", name: "Superdev worker proof", provider: "superdev-worker-proof",
	api: "openai-completions", baseUrl: "http://unused.invalid", reasoning: false,
	input: ["text"], contextWindow: 128_000, maxTokens: 1_024,
	cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
};

export default function provider(pi: ExtensionAPI) {
	const trace = join(process.env.SUPERDEV_WORKER_TRACE ?? process.cwd(), "requests.jsonl");
	let call = 0;
	pi.registerProvider(model.provider, {
		api: model.api, baseUrl: model.baseUrl, apiKey: "test-only", models: [model],
		streamSimple(_model, context: Context) {
			appendFileSync(trace, `${JSON.stringify(context)}\n`);
			const stream = createAssistantMessageEventStream();
			let content: AssistantMessage["content"] = [{ type: "text", text: "STAGE_RESULT" }];
			// Consume one step per turn. Counting messages would misread a restored
			// session, whose context already holds earlier assistant messages.
			const path = join(process.cwd(), "script.json");
			const script: string[] = JSON.parse(readFileSync(path, "utf8"));
			const instruction = script.shift();
			writeFileSync(path, JSON.stringify(script));
			if (instruction === "ask") {
				content = [{ type: "toolCall", id: `ask-${call++}`, name: "superdev_ask", arguments: {
					question: "Which boundary applies?",
					choices: [{ id: "narrow", label: "Keep it narrow" }, { id: "wide", label: "Include more" }],
					recommendation: { choiceId: "narrow", reason: "Stay within the approved plan." },
				} }];
			} else if (instruction === "hang") {
				content = [{ type: "toolCall", id: `hang-${call++}`, name: "bash", arguments: { command: "sleep 600" } }];
			} else if (instruction) {
				content = [{ type: "text", text: instruction }];
			}
			const message: AssistantMessage & { stopReason: "stop" | "toolUse" } = {
				role: "assistant", content, api: model.api, provider: model.provider, model: model.id,
				stopReason: content.some((part) => part.type === "toolCall") ? "toolUse" : "stop",
				timestamp: Date.now(),
				usage: { input: 10, output: 1, cacheRead: 0, cacheWrite: 0, totalTokens: 11,
					cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 } },
			};
			stream.push({ type: "start", partial: message });
			stream.push({ type: "done", reason: message.stopReason, message });
			stream.end();
			return stream;
		},
	});
	pi.on("session_start", async () => { await pi.setModel(model); });
}

// Scripted provider: exercise Pi's real message delivery, not a sendMessage stub.
import { createAssistantMessageEventStream, type AssistantMessage } from "@earendil-works/pi-ai";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import sokf from "../../../.pi/extensions/sokf.ts";

export default function (pi: ExtensionAPI) {
	sokf(pi);
	let requests = 0;
	pi.registerProvider("sokf-report-test", {
		api: "openai-completions",
		baseUrl: "http://unused.invalid",
		apiKey: "test-only",
		models: [{ id: "scripted", name: "Scripted report test", reasoning: false, input: ["text"], cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 }, contextWindow: 128000, maxTokens: 1024 }],
		streamSimple(model) {
			requests += 1;
			const stream = createAssistantMessageEventStream();
			let content: AssistantMessage["content"] = [{ type: "text", text: `Leave the finding unresolved: ${requests}` }];
			if (process.env.SOKF_REPORT_SCENARIO === "repair") {
				if (requests === 1) content = [{ type: "toolCall", id: "read", name: "read", arguments: { path: "knowledge/a.md" } }];
				if (requests === 2) content = [{ type: "toolCall", id: "write", name: "write", arguments: { path: "knowledge/a.md", content: "---\ntype: T\nid: alpha\n---\nRepaired.\n" } }];
			}
			const stopReason = content[0]?.type === "toolCall" ? "toolUse" : "stop";
			const output: AssistantMessage = {
				role: "assistant", content,
				api: model.api, provider: model.provider, model: model.id,
				usage: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, totalTokens: 0, cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 } },
				stopReason, timestamp: Date.now(),
			};
			stream.push({ type: "start", partial: output });
			stream.push({ type: "done", reason: stopReason, message: output });
			stream.end();
			return stream;
		},
	});
}

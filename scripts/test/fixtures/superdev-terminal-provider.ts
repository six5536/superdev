// Scripted, network-free provider for the real Pi JSON-mode terminal lifecycle.
import { createAssistantMessageEventStream, type AssistantMessage } from "@earendil-works/pi-ai";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import superdev from "../../../.pi/extensions/superdev/index.ts";

export default function terminalFixture(pi: ExtensionAPI) {
	superdev(pi);
	const scenario = process.env.SUPERDEV_TERMINAL_SCENARIO;
	let requests = 0;
	pi.registerProvider("superdev-terminal-test", {
		api: "openai-completions",
		baseUrl: "http://unused.invalid",
		apiKey: "test-only",
		models: [{ id: "scripted", name: "Scripted terminal test", reasoning: false, input: ["text"], cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 }, contextWindow: 128000, maxTokens: 1024 }],
		streamSimple(model, context) {
			requests += 1;
			const stream = createAssistantMessageEventStream();
			const repair = context.messages.some((message: any) => message.role === "user" && JSON.stringify(message.content).includes("only terminal repair turn"));
			const terminal = (args: Record<string, unknown>) => ({ type: "toolCall" as const, id: `submit-${requests}`, name: "superdev_submit_result", arguments: args });
			let content: AssistantMessage["content"] = [];
			let error: string | undefined;
			if (requests > 3) error = "Terminal repair exceeded the scripted request budget";
			else if (repair) {
				if (context.tools?.map((tool) => tool.name).join(",") !== "superdev_submit_result") error = "Repair exposed non-terminal tools";
				else if (scenario !== "unrepaired") content = [terminal({ status: "complete", summary: "Corrected the terminal envelope using existing work" })];
			} else if (requests === 1) {
				if (scenario === "rejected") content = [terminal({ status: "complete", summary: "Resolved correction", findings: [{ id: "resolved" }] })];
				if (scenario === "rejected-runtime") content = [terminal({ status: "complete", summary: " " })];
				if (scenario === "accepted") content = [terminal({ status: "complete", summary: "Accepted on first submission" })];
			}
			const output: AssistantMessage = {
				role: "assistant", content, api: model.api, provider: model.provider, model: model.id,
				usage: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, totalTokens: 0, cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 } },
				stopReason: error ? "error" : content.length ? "toolUse" : "stop",
				...(error ? { errorMessage: error } : {}), timestamp: Date.now(),
			};
			stream.push({ type: "start", partial: output });
			if (error) stream.push({ type: "error", reason: "error", error: output });
			else stream.push({ type: "done", reason: content.length ? "toolUse" : "stop", message: output });
			stream.end();
			return stream;
		},
	});
}

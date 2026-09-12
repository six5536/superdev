// Block-1 proof infrastructure only. This is not a second workflow engine.
import assert from "node:assert/strict";
import { join } from "node:path";
import { createAssistantMessageEventStream, type AssistantMessage, type Context, type Model } from "@earendil-works/pi-ai";
import {
	createAgentSession, DefaultResourceLoader, ModelRuntime, SessionManager, SettingsManager,
	type ExtensionAPI, type ExtensionFactory,
} from "@earendil-works/pi-coding-agent";

const model: Model<"openai-completions"> = {
	id: "scripted", name: "Workflow runtime proof", provider: "superdev-runtime-proof",
	api: "openai-completions", baseUrl: "http://unused.invalid", reasoning: false,
	input: ["text"], contextWindow: 128_000, maxTokens: 1_024,
	cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
};

function response(content: AssistantMessage["content"]): AssistantMessage & { stopReason: "stop" | "toolUse" } {
	return {
		role: "assistant", content, api: model.api, provider: model.provider, model: model.id,
		stopReason: content.some((part) => part.type === "toolCall") ? "toolUse" : "stop",
		timestamp: Date.now(),
		usage: { input: 10, output: 1, cacheRead: 0, cacheWrite: 0, totalTokens: 11,
			cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 } },
	};
}

/** Load the real pinned SDK with no project/global discovery or network model. */
export async function proofSession(options: {
	cwd: string;
	sessionFile?: string;
	factory?: ExtensionFactory;
	beforeFactory?: ExtensionFactory;
	reply?: (context: Context) => AssistantMessage["content"];
}) {
	const requests: Context[] = [];
	const errors: string[] = [];
	const agentDir = join(options.cwd, "agent");
	const settingsManager = SettingsManager.inMemory({
		compaction: { enabled: false, keepRecentTokens: 1 }, retry: { enabled: false },
	});
	const resourceLoader = new DefaultResourceLoader({
		cwd: options.cwd, agentDir, settingsManager,
		noExtensions: true, noSkills: true, noPromptTemplates: true, noThemes: true,
		noContextFiles: true,
		extensionFactories: [...(options.beforeFactory ? [options.beforeFactory] : []), (pi: ExtensionAPI) => {
			pi.registerProvider(model.provider, {
				api: model.api, baseUrl: model.baseUrl, apiKey: "test-only", models: [model],
				streamSimple(_model, context) {
					// Pi's tool objects also carry execute functions, which are not
					// model context. Capture their serialisable provider-facing data.
					requests.push(JSON.parse(JSON.stringify(context)));
					const stream = createAssistantMessageEventStream();
					const message = response(options.reply?.(context) ?? [{ type: "text", text: "Proof response." }]);
					stream.push({ type: "start", partial: message });
					stream.push({ type: "done", reason: message.stopReason, message });
					stream.end();
					return stream;
				},
			});
			return options.factory?.(pi);
		}],
	});
	await resourceLoader.reload();
	assert.deepEqual(resourceLoader.getExtensions().errors, []);
	const modelRuntime = await ModelRuntime.create({
		authPath: join(agentDir, "auth.json"), modelsPath: join(agentDir, "models.json"),
		modelsStorePath: join(agentDir, "models-store.json"), allowModelNetwork: false,
	});
	const sessionManager = options.sessionFile
		? SessionManager.open(options.sessionFile)
		: SessionManager.create(options.cwd, join(options.cwd, "sessions"));
	const { session } = await createAgentSession({
		cwd: options.cwd, agentDir, model, modelRuntime, resourceLoader, settingsManager,
		sessionManager, thinkingLevel: "off",
	});
	await session.bindExtensions({ mode: "tui", onError: (error) => errors.push(`${error.event}: ${error.error}`) });
	return { session, requests, errors };
}

export function text(context: Context): string {
	return JSON.stringify(context);
}

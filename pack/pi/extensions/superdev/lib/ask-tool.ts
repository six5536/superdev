// The one authored home for `superdev_ask`.
//
// The name, the schema, and the validation live here once. Only the resolver
// differs: the controller puts the question to the human, and a worker relays
// it to its controller. Two registrations of one tool in one process is a
// name collision, so a process registers this exactly once.
import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import { validateQuestion, type Question } from "./questions.ts";

/** Resolve one validated question. The caller decides how a human is reached. */
export type AskResolver = (question: Question, ctx: ExtensionContext) => Promise<unknown>;

/**
 * Register the ask tool.
 *
 * It is available whenever the extension is, because asking the human a
 * well-formed question is useful outside a workflow as well as inside one.
 */
export function registerAskTool(pi: ExtensionAPI, resolve: AskResolver) {
	pi.registerTool({
		name: "superdev_ask",
		label: "Ask a question",
		description: "Ask one question with stable choices and a recommended choice; distinguish answers, discussion, another action, and pause",
		promptGuidelines: [
			"Use superdev_ask to put a decision to the human, one question at a time.",
			"Name the recommended choice by its id and say why; do not mark it in a label.",
		],
		parameters: Type.Object({
			question: Type.String({ description: "The decision to settle, as one question" }),
			choices: Type.Array(Type.Object({
				id: Type.String({ description: "Stable identifier, referenced by the recommendation" }),
				label: Type.String({ description: "What this choice means, without presentation markers" }),
			}), { minItems: 1, maxItems: 8 }),
			recommendation: Type.Object({
				choiceId: Type.String({ description: "Which supplied choice is recommended" }),
				reason: Type.String({ description: "Why, so the human can disagree with it" }),
			}),
		}),
		executionMode: "sequential",
		execute: async (_id, input: Question, _signal, _update, ctx) => {
			// Validate in the asking process, so a malformed question returns a
			// tool error the author can correct rather than a distant failure.
			validateQuestion(input);
			const result = await resolve(input, ctx);
			return { content: [{ type: "text", text: JSON.stringify(result) }], details: result };
		},
	});
}

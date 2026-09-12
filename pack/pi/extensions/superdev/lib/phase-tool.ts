import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { StringEnum } from "@earendil-works/pi-ai";
import { Type } from "typebox";
import { stages, steps } from "./client.ts";
import type { PhaseInput, ScopeController } from "./scope.ts";

/** The tool requests actions. Only the interactive question handler approves documents. */
export function registerPhaseTool(pi: ExtensionAPI, controller: ScopeController) {
	pi.registerTool({
		name: "superdev_run_phase",
		label: "Superdev workflow",
		description: "Inspect local progress, request SCOPE permission or document approval, save completed steps, and pause or resume",
		parameters: Type.Object({
			phase: StringEnum(["scope", "build", "accept"] as const),
			action: StringEnum(["inspect", "run", "resume", "pause", "continue", "record-step", "approve-issue", "approve-plan", "attach-plan", "assess-change", "recover-publication", "migrate", "rescope", "record-discussion", "consume-retry",
				"commit-block", "complete-build", "return-to-build", "accept"] as const),
			workflow: Type.Optional(Type.String({ description: "Internal workflow ID returned by inspect" })),
			issue: Type.Optional(Type.String()), plan: Type.Optional(Type.String()), defaultBranch: Type.Optional(Type.String()),
			steps: Type.Optional(Type.Array(StringEnum(steps), { minItems: 1, maxItems: 8, description: "Explicit group to request permission for, in execution order" })),
			step: Type.Optional(StringEnum(steps)),
			stage: Type.Optional(StringEnum(stages, { description: "Execution stage to run; review and acceptance reset context first" })),
			note: Type.Optional(Type.String({ description: "Results, open findings, actual diff assessment, or migration facts" })),
			document: Type.Optional(StringEnum(["issue", "plan"] as const)),
			fromHash: Type.Optional(Type.String()), formattingOnly: Type.Optional(Type.Boolean()),
			indexes: Type.Optional(Type.Array(Type.String({ description: "Only generated indexes required for this document's publication" }))),
			completedBlocks: Type.Optional(Type.Array(Type.Integer({ minimum: 1 }))),
			block: Type.Optional(Type.Integer({ minimum: 1, description: "Stable plan block number to commit" })),
			areas: Type.Optional(Type.Array(Type.String({ description: "Path-scoped areas this block declares" }), { minItems: 1 })),
			reset: Type.Optional(Type.Boolean({ description: "Reset context before this stage: after a completed block, or when context fills mid-block" })),
			unfinished: Type.Optional(Type.String({ description: "Partial work and uncertain command outcomes to inspect before continuing" })),
			evidence: Type.Optional(Type.Array(Type.String({ description: "Verification results for this candidate, recorded as run" }))),
		}),
		executionMode: "sequential",
		execute: async (_id, input: PhaseInput, _signal, _update, ctx) => {
			const result = await controller.run(input, ctx);
			return { content: [{ type: "text", text: JSON.stringify(result) }], details: result };
		},
	});
}

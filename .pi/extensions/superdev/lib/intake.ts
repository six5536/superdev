import { Type } from "typebox";

/** Shared intake UI; review queues use the same choices and discussion vocabulary. */
export function registerIntakeTools({ pi }: any) {
    pi.registerTool({
        name: "superdev_ask",
        label: "Discuss workflow intent",
        description: "Ask one intake question with choices and a recommendation; allow a typed answer, chat, or cancellation",
        parameters: Type.Object({
            question: Type.String(),
            choices: Type.Array(Type.String(), { minItems: 1, maxItems: 8 }),
            recommendation: Type.String(),
        }),
        executionMode: "sequential",
        execute: async (_id: string, input: any, _signal: AbortSignal, _update: unknown, ctx: any) => {
            if (!ctx.hasUI) return result({ status: "human-input-required", ...input });
            const answer = await ctx.ui.select(`${input.question}\nRecommendation: ${input.recommendation}`, [...input.choices, "Type another answer", "Discuss in chat", "Cancel"]);
            if (!answer || answer === "Cancel") return result({ status: "cancelled" });
            if (answer === "Discuss in chat") return result({ status: "discuss", question: input.question });
            const text = answer === "Type another answer" ? await ctx.ui.input(input.question) : answer;
            return result(text?.trim() ? { status: "answered", answer: text.trim() } : { status: "cancelled" });
        },
    });
}

function result(value: unknown) {
    return { content: [{ type: "text", text: JSON.stringify(value) }], details: value };
}

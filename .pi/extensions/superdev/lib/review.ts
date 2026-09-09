import { StringEnum } from "@earendil-works/pi-ai";
import { Type } from "typebox";

export const roles = ["scope", "requirements-review", "build", "code-review", "accept"] as const;
export type Role = typeof roles[number];
export type FindingClassification = "mechanical" | "substantive" | "correctable-within-scope" | "requires-scope";

export type ReviewFinding = {
	id: string;
	classification: FindingClassification;
	summary: string;
	path?: string;
	location?: string;
	requirement?: string;
	evidence: string;
	impact: string;
	question?: string;
	choices?: Array<{ label: string; description?: string }>;
	recommendation?: string;
	dependsOn?: string[];
};

export type ChecklistItem = {
	area: "requirements" | "contracts" | "architecture" | "tests" | "documentation" | "scope" | "consistency";
	complete: boolean;
	evidence: string;
};

export type RoleResult = {
	status: "complete" | "clean" | "findings" | "rescope" | "duplicate" | "blocked";
	summary: string;
	findings?: ReviewFinding[];
	checklist?: ChecklistItem[];
	artifactPath?: string;
};

const choiceSchema = Type.Object({
	label: Type.String(),
	description: Type.Optional(Type.String()),
});
function findingSchemaFor(role: Role) {
	return Type.Object({
	id: Type.String({ description: "Stable ID; other findings reference it through dependsOn" }),
	classification: StringEnum(classifications[role]),
	summary: Type.String(),
	path: Type.Optional(Type.String()),
	location: Type.Optional(Type.String()),
	requirement: Type.Optional(Type.String()),
	evidence: Type.String(),
	impact: Type.String(),
	question: Type.Optional(Type.String()),
	choices: Type.Optional(Type.Array(choiceSchema)),
	recommendation: Type.Optional(Type.String()),
	dependsOn: Type.Optional(Type.Array(Type.String(), { description: "IDs in this same set that must be answered first" })),
	});
}
const checklistSchema = Type.Object({
	area: StringEnum(["requirements", "contracts", "architecture", "tests", "documentation", "scope", "consistency"] as const),
	complete: Type.Boolean(),
	evidence: Type.String(),
});

/** Match the advertised result fields to the role's runtime acceptance rules. */
export function roleResultSchemaFor(role: Role) {
	return Type.Object({
		status: StringEnum(allowed[role]),
		summary: Type.String({ description: "Outcome, completed corrections, and any blocker. Describe resolved findings here, not in findings." }),
		...(allowed[role].includes("findings") ? {
			findings: Type.Optional(Type.Array(findingSchemaFor(role), { description: "Unresolved actionable findings only; nonempty only with status findings. Never list resolved corrections." })),
		} : {}),
		checklist: Type.Optional(Type.Array(checklistSchema)),
	}, { additionalProperties: false });
}

/** Classifications the parent can route for each role; the schema declares them. */
const classifications: Record<Role, FindingClassification[]> = {
	scope: ["mechanical", "substantive"],
	"requirements-review": ["mechanical", "substantive"],
	build: ["correctable-within-scope", "requires-scope"],
	"code-review": ["correctable-within-scope", "requires-scope"],
	accept: ["correctable-within-scope", "requires-scope"],
};
const allowed: Record<Role, RoleResult["status"][]> = {
	scope: ["complete", "blocked"],
	"requirements-review": ["clean", "findings"],
	build: ["complete", "rescope", "blocked"],
	"code-review": ["clean", "findings"],
	accept: ["complete", "findings", "blocked"],
};

/**
 * Decode one terminal result. This checks only what the parent must decode or
 * route: shape, configured bounds, and a traversable finding graph. Judging a
 * finding's quality or a review's completeness belongs to the role that wrote
 * it, so neither is grounds for discarding a completed run.
 */
export function validateRoleResult(role: Role, value: unknown, limits?: { maxBytes: number; maxFindings: number }): RoleResult {
	if (!value || typeof value !== "object") throw new Error(`${role} returned an invalid terminal result`);
	let result = value as RoleResult;
	if (!allowed[role].includes(result.status) || typeof result.summary !== "string" || !result.summary.trim()) {
		throw new Error(`${role} returned an invalid terminal result`);
	}
	const bytes = Buffer.byteLength(JSON.stringify(result));
	if (limits && bytes > limits.maxBytes) throw new Error(`${role} review state exceeded ${limits.maxBytes} bytes`);
	if (result.findings && limits && result.findings.length > limits.maxFindings) {
		throw new Error(`${role} returned more than ${limits.maxFindings} findings`);
	}
	if (result.findings?.length && !allowed[role].includes("findings")) {
		// The role has no findings channel; its summary carries completed work.
		const { findings: _unrouted, ...rest } = result;
		result = rest;
	} else if (result.findings?.length && result.status !== "findings") {
		// Route the findings rather than discarding a completed review over a
		// status the author contradicted.
		result = { ...result, status: "findings" };
	}
	if (result.status === "findings" && !result.findings?.length) {
		throw new Error(`${role} reported findings without structured findings`);
	}
	if (result.findings) {
		const ids = new Set<string>();
		for (const finding of result.findings) {
			// The question queue is keyed by ID, so duplicates lose a finding.
			if (!finding.id?.trim() || ids.has(finding.id)) throw new Error(`${role} returned missing or duplicate finding IDs`);
			ids.add(finding.id);
		}
		for (const finding of result.findings) {
			for (const dependency of finding.dependsOn ?? []) if (!ids.has(dependency) || dependency === finding.id) {
				throw new Error(`${role} returned an invalid finding dependency`);
			}
		}
		const visiting = new Set<string>();
		const visited = new Set<string>();
		const byId = new Map(result.findings.map((finding) => [finding.id, finding]));
		const visit = (id: string): void => {
			if (visiting.has(id)) throw new Error(`${role} returned a cyclic finding dependency`);
			if (visited.has(id)) return;
			visiting.add(id);
			for (const dependency of byId.get(id)?.dependsOn ?? []) visit(dependency);
			visiting.delete(id);
			visited.add(id);
		};
		for (const id of ids) visit(id);
	}
	return result;
}

/** Legacy parser retained only for direct test callers; child authority uses the typed terminating tool. */
export function parseLegacyRoleResult(role: Role, answer: string): RoleResult {
	const marker = "SUPERDEV_RESULT ";
	const line = answer.split("\n").reverse().find((candidate) => candidate.startsWith(marker));
	if (!line) throw new Error(`${role} omitted its structured terminal result`);
	try {
		return validateRoleResult(role, JSON.parse(line.slice(marker.length)));
	} catch (error) {
		if (error instanceof SyntaxError) throw new Error(`${role} returned malformed terminal JSON`);
		throw error;
	}
}



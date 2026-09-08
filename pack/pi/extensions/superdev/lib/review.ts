import { createHash } from "node:crypto";
import { StringEnum } from "@earendil-works/pi-ai";
import { Type } from "typebox";

export const roles = ["scope", "requirements-review", "build", "code-review", "accept", "file"] as const;
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
const findingSchema = Type.Object({
	id: Type.String(),
	classification: StringEnum(["mechanical", "substantive", "correctable-within-scope", "requires-scope"] as const),
	summary: Type.String(),
	path: Type.Optional(Type.String()),
	location: Type.Optional(Type.String()),
	requirement: Type.Optional(Type.String()),
	evidence: Type.String(),
	impact: Type.String(),
	question: Type.Optional(Type.String()),
	choices: Type.Optional(Type.Array(choiceSchema)),
	recommendation: Type.Optional(Type.String()),
	dependsOn: Type.Optional(Type.Array(Type.String())),
});
const checklistSchema = Type.Object({
	area: StringEnum(["requirements", "contracts", "architecture", "tests", "documentation", "scope", "consistency"] as const),
	complete: Type.Boolean(),
	evidence: Type.String(),
});

export const roleResultSchema = Type.Object({
	status: StringEnum(["complete", "clean", "findings", "rescope", "duplicate", "blocked"] as const),
	summary: Type.String(),
	findings: Type.Optional(Type.Array(findingSchema)),
	checklist: Type.Optional(Type.Array(checklistSchema)),
});

const reviewRoles = new Set<Role>(["requirements-review", "code-review", "accept"]);
const allowed: Record<Role, RoleResult["status"][]> = {
	scope: ["complete", "blocked"],
	"requirements-review": ["clean", "findings"],
	build: ["complete", "rescope", "blocked"],
	"code-review": ["clean", "findings"],
	accept: ["complete", "findings", "blocked"],
	file: ["complete", "duplicate", "blocked"],
};

export function validateRoleResult(role: Role, value: unknown, limits?: { maxBytes: number; maxFindings: number }): RoleResult {
	if (!value || typeof value !== "object") throw new Error(`${role} returned an invalid terminal result`);
	const result = value as RoleResult;
	if (!allowed[role].includes(result.status) || typeof result.summary !== "string" || !result.summary.trim()) {
		throw new Error(`${role} returned an invalid terminal result`);
	}
	const bytes = Buffer.byteLength(JSON.stringify(result));
	if (limits && bytes > limits.maxBytes) throw new Error(`${role} review state exceeded ${limits.maxBytes} bytes`);
	if (result.findings && limits && result.findings.length > limits.maxFindings) {
		throw new Error(`${role} returned more than ${limits.maxFindings} findings`);
	}
	if (result.status === "findings" && (!Array.isArray(result.findings) || result.findings.length === 0)) {
		throw new Error(`${role} reported findings without structured findings`);
	}
	if (result.status !== "findings" && result.findings?.length) {
		throw new Error(`${role} returned findings with contradictory status ${result.status}`);
	}
	if (result.findings) {
		const ids = new Set<string>();
		for (const finding of result.findings) {
			if (!finding.id?.trim() || ids.has(finding.id)) throw new Error(`${role} returned missing or duplicate finding IDs`);
			ids.add(finding.id);
			if (!finding.summary?.trim() || !finding.evidence?.trim() || !finding.impact?.trim()) throw new Error(`${role} returned an incomplete finding`);
			if (role === "requirements-review" && !["mechanical", "substantive"].includes(finding.classification)) {
				throw new Error("requirements-review finding omitted its mechanical/substantive classification");
			}
			if (role === "requirements-review" && finding.classification === "substantive"
				&& (!finding.question?.trim() || !finding.recommendation?.trim())) {
				throw new Error("substantive SCOPE finding omitted its user question or recommendation");
			}
			if (role === "code-review" && !["correctable-within-scope", "requires-scope"].includes(finding.classification)) {
				throw new Error("code-review finding omitted its routing classification");
			}
			if (role === "accept" && !["correctable-within-scope", "requires-scope"].includes(finding.classification)) {
				throw new Error("ACCEPT finding omitted its routing classification");
			}
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
	if (reviewRoles.has(role) && result.status !== "blocked") {
		const expected = new Set(["requirements", "contracts", "architecture", "tests", "documentation", "scope", "consistency"]);
		const seen = new Set<string>();
		for (const item of result.checklist ?? []) {
			if (!item.complete || !item.evidence?.trim() || seen.has(item.area)) throw new Error(`${role} returned an incomplete or duplicate review checklist`);
			seen.add(item.area);
			expected.delete(item.area);
		}
		if (expected.size) throw new Error(`${role} omitted review checklist areas: ${[...expected].join(", ")}`);
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

export function findingFingerprint(finding: ReviewFinding): string {
	const normalized = [
		finding.classification,
		finding.path ?? "",
		finding.location ?? "",
		finding.requirement ?? "",
		finding.summary,
		finding.impact,
		finding.question ?? "",
		JSON.stringify(finding.choices ?? []),
		JSON.stringify(finding.dependsOn ?? []),
	].map((part) => part.toLowerCase().replace(/\s+/g, " ").trim()).join("\n");
	return createHash("sha256").update(normalized).digest("hex");
}

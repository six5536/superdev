---
type: Template
id: template-security-review
title: Security Review Template
description: Risk verdict, scope and threat model, findings with attack scenarios, and areas checked sound.
status: stable
---

# Security review: <branch / PR / component reviewed>

## Verdict

<One sentence: overall risk posture — e.g. "One high-severity injection issue to fix before merge; otherwise sound." Findings ranked by severity.>

## Scope

- Reviewed: <files/changes examined and the threat model assumed (who the attacker is, what they can reach)>
- Not reviewed: <explicitly out of scope, e.g. infra config, dependencies>

## Findings

### 1. <Short claim, e.g. "User-controlled path reaches fs.readFile"> — `path/to/file.ts:123`

- Severity: critical | high | medium | low | informational
- Class: injection | authn/authz | data exposure | SSRF | path traversal | crypto | insecure default | ...
- Attack scenario: <concrete path from attacker-controlled input to impact — what they send, what happens. If no realistic path exists, it's informational, not a finding.>
- Impact: <what an attacker gains: data read/written, code execution, privilege>
- Remediation: <the specific fix; the safe pattern to use>

### 2. <...>

## Checked and sound

<Areas examined with no issues found — input validation, secrets handling, authz checks — one line each, so coverage is visible.>

- <Area> — <what was checked>

## Recommendations

<Non-blocking hardening: defense-in-depth, logging, dependency hygiene. Delete if empty.>

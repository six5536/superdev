---
type: Schema
id: schema-security-review
title: Security Review Schema
description: Security reviews — risk verdict, scope and threat model, findings with attack scenarios, and areas checked sound.
---

# Security Review Schema

Structural rules for security review documents, filed at
`knowledge/reports/security-review-{nnn}-{slug}.md`, listed in that
directory's index and selected by frontmatter `type`. Parallel to
`schema-code-review`, but each finding carries an attack scenario rather
than a failure scenario: no realistic path from attacker-controlled input to
impact means the entry is informational, not a finding.

````yaml
description: >
  Security review findings: the risk verdict first, the scope and threat model
  assumed, findings ranked by severity each with a concrete attack scenario,
  the areas checked and found sound, and non-blocking hardening.
line-limit: 800

frontmatter:
  type:
    const: SecurityReview
  id:
    pattern: '^security-review-\d{3}-[a-z0-9-]+$'

sections-ordered: true
sections:
  - heading-pattern: '^Security review: .+$'
    level: 1
    required: true
    description: >
      The document title, naming the branch, PR or component reviewed.
  - heading: "Verdict"
    level: 2
    required: true
    content: prose
    description: >
      One sentence: the overall risk posture — e.g. "One high-severity
      injection issue to fix before merge; otherwise sound." Findings ranked
      by severity.
  - heading: "Scope"
    level: 2
    required: true
    content: bullet-list
    description: >
      Reviewed: the files and changes examined, and the threat model assumed —
      who the attacker is and what they can reach. Not reviewed: what is
      explicitly out of scope, e.g. infra config, dependencies.
  - heading: "Findings"
    level: 2
    required: true
    description: >
      The ranked findings, one level-3 heading per finding.
  - heading-pattern: '^\d+\. .+ — .+$'
    level: 3
    required: true
    repeatable: true
    content: bullet-list
    description: >
      One finding: a short claim plus `path/to/file.ts:123`. Bullets for
      Severity (critical | high | medium | low | informational), Class
      (injection | authn/authz | data exposure | SSRF | path traversal |
      crypto | insecure default | …), Attack scenario (a concrete path from
      attacker-controlled input to impact — what they send, what happens; if
      no realistic path exists it is informational, not a finding), Impact
      (what an attacker gains: data read or written, code execution,
      privilege), and Remediation (the specific fix, and the safe pattern to
      use).
  - heading: "Checked and sound"
    level: 2
    required: true
    content: bullet-list
    description: >
      Areas examined with no issues found — input validation, secrets
      handling, authz checks — one line each, so coverage is visible.
  - heading: "Recommendations"
    level: 2
    content: prose
    description: >
      Non-blocking hardening: defence in depth, logging, dependency hygiene.
      Omit the section when empty.

example: |
  ---
  type: SecurityReview
  id: security-review-001-pack-allowlist
  title: Security review of feature/pack-allowlist
  description: Review of the pack transport allowlist change against its threat model.
  ---

  # Security review: feature/pack-allowlist

  ## Verdict

  One high-severity path traversal to fix before merge; the transport
  allowlist itself is sound.

  ## Scope

  - Reviewed: `src/pack/resolve.rs` and `src/pack/extract.rs`. Attacker
    controls the manifest contents and the archive served by a pack source.
  - Not reviewed: CI configuration, and the dependency tree itself.

  ## Findings

  ### 1. Archive entry paths are joined before normalising — `src/pack/extract.rs:64`

  - Severity: high
  - Class: path traversal
  - Attack scenario: A pack source serves an archive containing an entry
    named `../../.ssh/authorized_keys`. Extraction joins it onto the cache
    directory and opens the result, writing outside the cache.
  - Impact: Arbitrary file write as the invoking user, which on most
    developer machines is code execution at next login.
  - Remediation: Normalise each entry path and reject any that escapes the
    root before opening it — `Path::components` rejecting `ParentDir`.

  ## Checked and sound

  - Transport allowlist — matched case-insensitively after lowercasing, and
    applied at parse before any fetch.
  - Lockfile writes — written to a temporary file and renamed, so a crash
    mid-write cannot leave a half-written pin.

  ## Recommendations

  Extraction runs with the invoking user's full privileges. A size and entry
  count cap would bound the damage from a hostile archive even once the
  traversal above is fixed.
````

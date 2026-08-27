---
type: Schema
id: schema-investigation
title: Investigation Schema
description: Investigation write-ups — conclusion first, evidence with sources, ruled-out hypotheses and a recommendation.
---

# Investigation Schema

Structural rules for investigation write-ups, matched by name
(`**/*investigation*.md`); the source names no filing directory, and the
document is not an SOKF concept, so it carries no frontmatter.

````yaml
target-files: "**/*investigation*.md"
description: >
  Conclusion-first write-up — question, evidence with sources, ruled-out
  hypotheses, and recommendation.
line-limit: 800

sections-ordered: true
sections:
  - heading-pattern: '^Investigation: .+$'
    level: 1
    required: true
    description: >
      The document title, naming the question being answered, e.g. "Why
      does startup take 8s?".
  - heading: "Conclusion"
    level: 2
    required: true
    content: prose
    description: >
      Lead with the answer: what you found, stated plainly, with
      confidence level if it's not certain. Everything below is
      supporting evidence.
  - heading: "Question / trigger"
    level: 2
    required: true
    content: prose
    description: >
      What prompted this: the symptom, the user question, the
      anomaly. Include how to reproduce/observe it.
  - heading: "Evidence"
    level: 2
    required: true
    content: numbered-list
    description: >
      Each finding with its source, so the chain is checkable: a
      numbered list of observations, each with where it came from
      (`path/to/file.ts:123`, log excerpt, command output), followed
      by the key log/output excerpt trimmed to the load-bearing
      lines.
  - heading: "Ruled out"
    level: 2
    required: true
    content: bullet-list
    description: >
      Hypotheses checked and eliminated, one line each with the
      disproving evidence — so nobody re-walks dead ends. Format:
      - Hypothesis — ruled out because evidence.
  - heading: "Remaining uncertainty"
    level: 2
    content: prose
    description: >
      What is still assumed rather than proven, and what
      experiment/data would settle it. Delete if the conclusion is
      airtight.
  - heading: "Recommendation"
    level: 2
    required: true
    content: prose
    description: >
      What to do about it: the fix, the follow-up, or "no action
      needed" — with rationale.

example: |
  # Investigation: Why does pack sync fail on the second run?

  ## Conclusion

  The lock entry is written before the fetch completes, so an aborted
  fetch leaves a stale entry; high confidence, reproduced twice.

  ## Question / trigger

  Pack sync succeeds once, then fails on every rerun until the lock file
  is deleted. Reproduce: sync, drop the network mid-fetch, sync again.

  ## Evidence

  1. Lock entry written before the fetch starts — `src/pack/resolve.rs:41`
  2. The rerun reads the stale entry and skips the fetch — sync log, run 2

  ## Ruled out

  - Cache directory permissions — ruled out because a clean cache
    reproduces the failure.

  ## Recommendation

  Write the lock entry only after the fetch verifies.
````

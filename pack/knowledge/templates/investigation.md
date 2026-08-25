---
type: Template
id: template-investigation
title: Investigation Template
description: Conclusion-first write-up — question, evidence with sources, ruled-out hypotheses, and recommendation.
status: stable
---

# Investigation: <question being answered, e.g. "Why does startup take 8s?">

## Conclusion

<Lead with the answer: what you found, stated plainly, with confidence level if it's not certain. Everything below is supporting evidence.>

## Question / trigger

<What prompted this: the symptom, the user question, the anomaly. Include how to reproduce/observe it.>

## Evidence

<Each finding with its source, so the chain is checkable:>

1. <Observation> — <where it came from: `path/to/file.ts:123`, log excerpt, command output>
2. <Observation> — <source>
3. <Observation> — <source>

```
<key log/output excerpt, trimmed to the load-bearing lines>
```

## Ruled out

<Hypotheses checked and eliminated, one line each with the disproving evidence — so nobody re-walks dead ends.>

- <Hypothesis> — ruled out because <evidence>

## Remaining uncertainty

<What is still assumed rather than proven, and what experiment/data would settle it. Delete if the conclusion is airtight.>

## Recommendation

<What to do about it: the fix, the follow-up, or "no action needed" — with rationale.>

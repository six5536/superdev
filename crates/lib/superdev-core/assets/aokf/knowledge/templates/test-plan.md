---
type: Template
id: template-test-plan
title: Test Plan Template
description: Scope, risks driving the plan, automated and manual cases, regression coverage, and exit criteria.
status: stable
---

# Test plan: <feature/change under test>

## Scope

- Under test: <the behavior/components this plan covers>
- Not under test: <explicitly excluded, and why (covered elsewhere, out of scope)>

## Risks driving this plan

<The 2–4 ways this change is most likely to break — the plan should visibly attack these.>

1. <Risk>
2. <Risk>

## Test cases

### Automated

| # | Case | Type | Inputs / setup | Expected result |
|---|------|------|----------------|-----------------|
| 1 | <happy path> | unit / integration / e2e | <...> | <...> |
| 2 | <edge: empty/zero/max> | | <...> | <...> |
| 3 | <error path: bad input, dependency failure> | | <...> | <...> |
| 4 | <concurrency/ordering, if relevant> | | <...> | <...> |

### Manual verification

1. <Step-by-step check with exact commands and expected observation>

## Regression coverage

<Existing tests that must keep passing; areas adjacent to the change worth a smoke check.>

## Environments / data

<Required services, fixtures, env vars, seeded data. How to set them up.>

## Exit criteria

- <All automated cases pass in CI>
- <Manual checks signed off>
- <Known gaps accepted and listed here, if any>

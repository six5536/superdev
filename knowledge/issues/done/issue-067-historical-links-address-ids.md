---
type: Issue
id: issue-067-historical-links-address-ids
title: 'Historical plan: Links address ids'
description: Migration-derived issue preserving the primary issue relationship for plan-010-links-address-ids.
kind: chore
lifecycle: done
---

# Chore: historical plan: links address ids

## Summary

Preserve `plan-010-links-address-ids` as queryable workflow history under the canonical issue/plan relationship.

## Context

The plan predates the requirement that every plan implements exactly one issue and had no recoverable primary issue link.

## Behaviour

The migrated plan has one `implements` relationship to this issue, so incoming graph edges expose its history without invented product requirements.

## Scope

Mechanical metadata and required schema sections only; historical intent and product behavior are unchanged.

## Resolution

The historical work is represented by `plan-010-links-address-ids`; this migration-derived issue preserves the required one-issue-per-plan relationship.

---
type: Issue
id: issue-075-historical-flatten-crate-api
title: 'Historical plan: Flatten the superdev-core API'
description: Migration-derived issue preserving the primary issue relationship for plan-001-flatten-crate-api.
kind: chore
lifecycle: open
---

# Chore: historical plan: flatten the superdev-core api

## Summary

Preserve `plan-001-flatten-crate-api` as queryable workflow history under the canonical issue/plan relationship.

## Context

The plan predates the requirement that every plan implements exactly one issue and had no recoverable primary issue link.

## Behaviour

The migrated plan has one `implements` relationship to this issue, so incoming graph edges expose its history without invented product requirements.

## Scope

Mechanical metadata and required schema sections only; historical intent and product behavior are unchanged.

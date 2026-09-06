---
type: Issue
id: issue-074-historical-persistent-mcp-transport-for-pi-sokf
title: 'Historical plan: Persistent MCP transport for Pi SOKF'
description: Migration-derived issue preserving the primary issue relationship for plan-028-persistent-mcp-transport-for-pi-sokf.
kind: chore
lifecycle: done
---

# Chore: historical plan: persistent mcp transport for pi sokf

## Summary

Preserve `plan-028-persistent-mcp-transport-for-pi-sokf` as queryable workflow history under the canonical issue/plan relationship.

## Context

The plan predates the requirement that every plan implements exactly one issue and had no recoverable primary issue link.

## Behaviour

The migrated plan has one `implements` relationship to this issue, so incoming graph edges expose its history without invented product requirements.

## Scope

Mechanical metadata and required schema sections only; historical intent and product behavior are unchanged.

## Resolution

The historical work is represented by `plan-028-persistent-mcp-transport-for-pi-sokf`; this migration-derived issue preserves the required one-issue-per-plan relationship.

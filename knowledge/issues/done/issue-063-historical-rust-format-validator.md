---
type: Issue
id: issue-063-historical-rust-format-validator
title: 'Historical plan: Fold the superdev-format validator into the Rust validator'
description: Migration-derived issue preserving the primary issue relationship for plan-006-rust-format-validator.
kind: chore
lifecycle: done
---

# Chore: historical plan: fold the superdev-format validator into the rust validator

## Summary

Preserve `plan-006-rust-format-validator` as queryable workflow history under the canonical issue/plan relationship.

## Context

The plan predates the requirement that every plan implements exactly one issue and had no recoverable primary issue link.

## Behaviour

The migrated plan has one `implements` relationship to this issue, so incoming graph edges expose its history without invented product requirements.

## Scope

Mechanical metadata and required schema sections only; historical intent and product behavior are unchanged.

## Resolution

The historical work is represented by `plan-006-rust-format-validator`; this migration-derived issue preserves the required one-issue-per-plan relationship.

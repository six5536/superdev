---
type: Idea
id: idea-009-comment-preserving-manifest-stamping
title: Comment-preserving manifest stamping
description: Stamp the blueprint version into config.toml with a targeted toml_edit edit of the one key, so a hand-editable file keeps its comments.
status: draft
---

# Idea: comment-preserving manifest stamping

Stamp the blueprint version into `config.toml` with a targeted `toml_edit`
edit of the one key, so a hand-editable file keeps its comments.

## Motivation

`sync` rewrites `config.toml` through the whole-file `Manifest::save` when
it stamps the blueprint version, dropping any hand-written comments — the
rewrite `update` always did, now implicit in every post-upgrade sync.
Raised in the blueprint-migrations final review. Recorded here on the
backlog's retirement (ADR-048), where it sat under consideration.

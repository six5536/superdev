---
type: Spec
id: spec-workflows-skill-overrides
title: Workflows Skill Overrides
description: Design for provider-carried skill overrides — the mattpocock-skills component materialises embedded replacements, grilling first — installed only where that provider is.
status: stable
---

# Motivation

superdev wants its own `grilling` — upstream's wording with the
question-format block replaced by "use any question type tool" — in this
repo and every managed repo. The override is *of* a mattpocock-skills
skill, so it must install exactly where that provider is installed:
never into a superpowers repo, never into a repo without workflows.

# Decision

A [skill override](../glossary.md) is carried by the provider component
it overrides, not by the superdev skill pack and not by a new
capability:

- **Overlay in the mattpocock-skills component.** The component embeds
  the override files in the binary and applies them during its own
  materialisation. The install-only-with-the-provider condition holds by
  construction; nothing else in the engine changes.
- **Existing `custom` is the opt-out.** `[workflows] custom =
  ["grilling"]` releases the skill — override and upstream alike — to
  the user. No new knob; a user who wants Matt's version copies it in
  themselves.
- **Cross-capability shadowing was designed and rejected** in an earlier
  round of this spec: a general install-order precedence rule
  (`ships()` on the trait, a shadow set through `Ctx`) solved the same
  collision but tied the override to the skills capability and cost an
  engine-wide mechanism. Tying the override to its provider dissolves
  the collision instead. Naive last-writer-wins was also rejected —
  planning happens once against pre-run state, so a materialise refresh
  would regress the file mid-apply and convergence would take two syncs.

# Design

- `MaterialiseSkills` gains `overrides`: (target path → embedded
  content) pairs, filled by the component from its baked-in assets.
  During materialisation the override content wins over the checkout's
  for those paths — extra override paths that have no checkout
  counterpart are written too — owned and workflows-attributed exactly
  like checkout files.
- Everything downstream comes free: refresh-on-drift (the lock hashes
  the override content, so editing the file or upgrading the binary
  replans), the provider-switch sweep (workflows attribution), and the
  custom release (the materialiser already skips custom names; the
  override pairs skip by the same skill-name rule).
- **Assets** live provider- and kind-scoped at
  `assets/overrides/mattpocock-skills/skills/grilling/{SKILL.md,
  agents/openai.yaml}` — not under `assets/skills/`, which is the
  pack's. The `skills/` level names the artefact kind: overrides are
  not inherently skills, and future kinds (commands, agent configs)
  get sibling directories. The drafted files move there: the draft
  SKILL.md is canonical (the live `.claude` copy converges on first
  sync); `agents/openai.yaml` is byte-identical to upstream and ships
  so the cross-harness descriptor survives.
- **Versioning.** Override content is embedded, so its provenance is
  the binary (like the pack); the workflows pin stays upstream's
  version. A binary upgrade that changes override content shows up as
  ordinary drift and rewrites on the next sync.
- **Adoption is untouched.** `grilling` remains an upstream name;
  init-adoption marks an existing directory custom under `[workflows]`
  as today, which also declines the override — consistent with custom
  as the opt-out.

# Accidental collisions

With deliberate overrides intra-component, any cross-component path
collision is a fault — say upstream someday ships a skill named like a
pack skill. Silent resolution would pick a winner nobody chose and
oscillate across syncs, so superdev refuses with a way out:

- **Plan time**: `plan_repo` checks every enabled component's claims
  for duplicate lock keys. A duplicate fails `sync` and is reported by
  `status`, naming both capabilities and the path; the message carries
  the remedy — add the name to one side's `custom` list, or upgrade
  superdev (a release can resolve the clash with an override or a
  rename).
- **Apply time**: the engine fails an action that writes a lock key
  another entry already wrote in the same run — the backstop for
  checkout-derived paths the planner cannot enumerate on a first sync.
  The failure unwinds like any other.

Both checks read data that already exists (the collected claims, the
run's written keys).

# Out of scope

- General cross-capability shadowing (see Decision). Accidental
  collisions are refused, not resolved; revisit shadowing only if a
  legitimate cross-capability override is ever wanted.
- Overrides for the superpowers provider — it delivers skills as a
  plugin, not files; there is nothing to overlay.
- A prefer-upstream-while-managed knob.

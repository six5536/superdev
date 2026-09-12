# Pi SOKF Extension Consolidation Plan

**Status:** Proposed standalone engineering plan.

## IMPORTANT NOTE

The only surviving part of this plan is to consolidate sokf into one extension named 'sokf'.
Most of the details below are out of date. The behaviour of the extension must not be changed.

## Purpose

Ship one Pi extension named `sokf` that owns:

- the SOKF-aware `read`, `edit`, and `write` overrides;
- `sokf_search` and `sokf_graph`;
- the private persistent MCP transport; and
- discovery of the `sokf-authoring` Pi skill.

This is intentionally a small root-level engineering plan outside the unfinished SCOPE → BUILD → ACCEPT workflow. It must not start, advance, or mutate the state of `plan-059-scope-build-accept-workflow`.

## Target layout

```text
.pi/extensions/sokf/
  index.ts
  transport.ts
  skills/
    sokf-authoring/
      SKILL.md

pack/pi/extensions/sokf/
  index.ts
  transport.ts
  skills/
    sokf-authoring/
      SKILL.md
```

Remove the active legacy locations after migration:

```text
.pi/extensions/sokf.ts
.pi/extensions/sokf-mcp.ts
.pi/skills/sokf-authoring/
pack/pi/skills/sokf-authoring/
```

Pi auto-discovers only `.pi/extensions/sokf/index.ts`. `transport.ts` is an ordinary private module and no longer needs a no-op default extension factory.

## Required behavior

### One extension

Move the adapter factory from `sokf.ts` to `sokf/index.ts` and the `SokfMcpClient` implementation from `sokf-mcp.ts` to `sokf/transport.ts`.

The extension remains functionally equivalent:

- one lazy MCP child per repository;
- serialized MCP calls;
- restart after failure;
- bounded protocol diagnostics;
- shutdown cleanup;
- SOKF-aware file-tool routing;
- search and graph tools; and
- bounded post-mutation validation feedback.

No second top-level Pi extension module remains.

### Extension-owned authoring skill

Place `sokf-authoring/SKILL.md` below the extension and register its parent directory through Pi's `resources_discover` event:

```ts
pi.on("resources_discover", () => ({
  skillPaths: [resolve(here, "skills")],
}));
```

The skill remains independently invocable as `/skill:sokf-authoring` and retains progressive disclosure: only its name and description are normally present; full instructions load on demand. Do not inline the full skill into the standing system prompt or tool descriptions.

The extension must resolve its resource path from `import.meta.url`, not the process working directory, so it works from repository subdirectories and when installed from a pack.

### One packaged item and owner

Treat `pack/pi/extensions/sokf/**` as one `PiExtension` item named `sokf`. Remove the separately owned `PiSkill` item named `sokf-authoring` from the pack layout and pipeline expectations.

Update content-layout, materialization, adoption, lock, drift, and orphan handling so a fresh sync installs one extension package and an upgrade safely retires the old skill path. The nested skill is owned as a file within the extension package, not discovered as a second content item.

Legacy cleanup must be safe:

- migrate known managed files by exact ownership/hash where available;
- preserve or back up locally modified managed files according to existing policy;
- never delete an unknown user-owned `sokf.ts`, `sokf-mcp.ts`, or skill directory merely because its name matches;
- remove stale lock entries only after the replacement package is successfully materialized.

The current repository's tracked files may be moved directly, but shipping behavior must also cover upgrades in managed consumer repositories.

## Work breakdown

### 1. Consolidate the runtime package

- Create `.pi/extensions/sokf/index.ts` and `transport.ts` by moving existing behavior without redesigning it.
- Update imports to `./transport.ts`.
- Remove the transport's no-op default factory.
- Add `resources_discover` for the nested authoring skill.
- Move the current skill unchanged first; make wording changes separately only if paths or ownership statements require them.

Verification:

- Pi discovers one SOKF extension factory.
- The five SOKF-aware tools still register exactly once.
- `resources_discover` returns the extension's nested skill root.
- `/skill:sokf-authoring` remains available after startup and `/reload`.

### 2. Update tests and evaluation paths

- Change adapter imports to `.pi/extensions/sokf/index.ts`.
- Change MCP client tests to import `.pi/extensions/sokf/transport.ts`.
- Extend the smoke fixture to invoke `resources_discover` and verify the returned `sokf-authoring/SKILL.md`.
- Update `scripts/sokf-eval.mjs` and every path assertion from the legacy extension/skill locations.
- Add a discovery regression proving `transport.ts` does not register as an extension and tools are not duplicated.

Verification:

```text
npm run test:scripts
```

Update `scripts/sokf-eval.mjs`, but do not require a paid model run for this structural migration. Its existing model-free fixture/path checks must pass without a provider call.

### 3. Package and migrate ownership

- Add the complete `sokf` extension directory under `pack/pi/extensions/`.
- Remove `pack/pi/skills/sokf-authoring/` only after replacement ownership is represented.
- Update Rust content-layout and pipeline tests from a standalone `PiSkill` to the nested extension resource, including `content/layout.rs`, pipeline tests, `tests/normative_shapes.rs`, and application manage/sync tests.
- Add fresh-install and upgrade fixtures covering legacy file removal, local modification backup/preservation, lock replacement, and idempotent second sync.
- Run sync only after reviewing unrelated working-tree changes; do not allow it to absorb or overwrite concurrent workflow work.

Verification:

- A fresh repository receives only `.pi/extensions/sokf/**`.
- An unchanged legacy install migrates without leaving duplicate extensions or skills.
- A modified legacy file is preserved/backed up according to policy.
- `status --drift` is clean after migration and remains clean after a second sync.

### 4. Align documentation

Update only directly affected references, including:

- `knowledge/software-components.md`;
- `knowledge/architecture.md`;
- `knowledge/directory-structure.md`;
- `knowledge/development-procedure.md`;
- `knowledge/development-commands.md`;
- `knowledge/configuration.md`;
- `knowledge/technology-stack.md` and glossary where ownership is described;
- changelog and any agent instruction that names the skill path.

Describe `sokf-authoring` as an independently invocable skill supplied by the `sokf` extension, not as a separately materialized Pi skill.

Verification:

```text
npm run check:docs
npm run check:blueprint
npm run check:validate
git diff --check
```

## Acceptance checklist

- [ ] Pi auto-discovers exactly one extension package named `sokf`.
- [ ] No active `.pi/extensions/sokf.ts` or `.pi/extensions/sokf-mcp.ts` remains.
- [ ] The MCP client is a private module with no extension factory.
- [ ] SOKF read/edit/write/search/graph behavior and shutdown/restart semantics are unchanged.
- [ ] `sokf-authoring` is discovered from inside the extension and remains available on demand.
- [ ] No separate `.pi/skills/sokf-authoring/` is installed after migration.
- [ ] Fresh install, unchanged upgrade, locally modified upgrade, lock migration, drift, and idempotence are tested.
- [ ] Tests and evaluations use only the new paths.
- [ ] Documentation consistently describes one SOKF extension and its nested skill.
- [ ] The unfinished Superdev workflow's phase, revision, ownership, and evidence are untouched.

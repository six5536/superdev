---
type: Contract
id: contract-004-config-superdev
kind: config
title: Config contract for superdev
description: What a managed repo supplies to superdev — the manifest as the reader declares it, the four environment variables, which source defines what, and what an unknown or invalid setting does.
lifecycle: active
resource: /crates/lib/superdev-core/src/manifest.rs
links:
  - rel: references
    to: adr-033-a-contract-defines-its-interface
    note: A contract carries a machine-readable definition; here it is the manifest's on-disk shape.
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: The definition is materialised from the `config` regions and bound by the include; the sources, the defaults and the secret rule are bound by `manifest.rs`'s and `embed.rs`'s tests.
  - rel: references
    to: configuration
    note: The behaviour behind each setting; what is promised here is the setting, its shape and its default.
---

# Config contract: superdev

What a managed repo supplies to superdev: the manifest, the environment
variables, and which source defines what. The Definition is the shape of
`.superdev/config.toml` as the reader declares it — every table, every
key, and the doc comment that says what each means. Behaviour carries
what the shape cannot say: the environment variables, which source wins,
the defaults, the one secret, and what an unknown or invalid setting
does. The behaviour behind each setting is in
[configuration][sokf:configuration]. The decisions behind the shape are
[ADR-033][sokf:adr-033-a-contract-defines-its-interface] and
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source].

## Definition

<!-- sokf:include /crates/lib/superdev-core/src/manifest.rs#config -->
```rust
/// Repo-relative path of the manifest.
pub const CONFIG_PATH: &str = ".superdev/config.toml";

/// One enabled capability: which provider fills it, at which version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityConfig {
    /// Provider id (e.g. "codegraph").
    pub provider: String,
    /// Version pin; None when the source manages versions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Skills released from management: superdev stops writing them and
    /// `status` reports them as custom. Honoured by `skills`, which writes
    /// into `.claude/skills/` alongside the SOKF knowledge.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom: Vec<String>,
}

/// `[knowledge]` — the SOKF knowledge's settings. A plain table, not a
/// capability: SOKF is part of superdev, so there is no provider to name and
/// no slot to leave empty.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeConfig {
    /// Set only in a manifest written before SOKF became core. Its presence
    /// is the whole reason the field exists: `parse` refuses such a manifest
    /// and names the edit, rather than silently reading a provider choice
    /// that no longer means anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Embedding provider for the search index. Absent = the local model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embeddings: Option<EmbeddingsConfig>,
    /// SOKF skills released from management: superdev stops writing them and
    /// `status` reports them as custom.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom: Vec<String>,
}

/// The project template `init` seeded this repo from: provenance, not
/// management — no verb ever re-plans a template. Recording the token values
/// beside the name answers "what seeded this repo, with what".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TemplateRecord {
    /// The shipped template's name.
    pub name: String,
    /// The value `{{superdev:project-name}}` substituted to.
    pub project_name: String,
    /// The value `{{superdev:project-slug}}` substituted to.
    pub project_slug: String,
    /// The binary version whose template content the repo last matched:
    /// stamped by `init`, restamped by the `template-update` skill after an
    /// update or adoption. Absent in manifests from before the field existed
    /// — they just lack the skill's "already up to date" short-circuit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// A capability's manifest shape as written: one table, or — for a many
/// slot — an array of tables, one per provider entry. Kept distinct through
/// parsing so the array form on a single slot can be refused, then
/// normalised to a list either way.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum WrittenEntries {
    /// A single `[name]` table.
    One(CapabilityConfig),
    /// `[[name]]` array-of-tables entries.
    Many(Vec<CapabilityConfig>),
}

/// One content pack the repo wants. Order in the manifest is layer order.
///
/// A `[[packs]]` array rather than a capability table: an absent capability
/// means disabled, but an absent pack list means the pack compiled into the
/// binary — a different thing, and one no capability table can say. ADR-001.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackEntry {
    /// Where the pack comes from, as the user wrote it.
    pub source: String,
    /// Git revision — tag, branch or commit sha. Absent for a path source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
}

/// The manifest as TOML sees it — the on-disk shape `parse` validates and
/// `to_toml` renders. Field order is the serialised order.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WrittenManifest {
    /// The superdev version last applied; `init` writes it.
    blueprint: String,
    /// `[template]` — the project template `init` seeded the repo from.
    /// Absent when the repo was never seeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    template: Option<TemplateRecord>,
    /// `[[packs]]` — the content packs to layer, in layer order. Absent means
    /// the pack embedded in the binary.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    packs: Vec<PackEntry>,
    /// Named ahead of the flatten so `[knowledge]` lands here rather than in
    /// the capability map. Always written: a fresh repo should be able to see
    /// the table it may put `custom` and `embeddings` in.
    #[serde(default)]
    knowledge: KnowledgeConfig,
    /// `[<capability>]` — one table per enabled capability, keyed by its
    /// kebab-case name; `[[<capability>]]` for a slot that takes several
    /// providers. An absent table means the capability is disabled.
    #[serde(flatten)]
    capabilities: BTreeMap<String, WrittenEntries>,
}
```
<!-- /sokf:include -->

<!-- sokf:include /crates/lib/superdev-core/src/sokf/embed.rs#config -->
```rust
/// Manifest `[knowledge.embeddings]`: which API embeds the knowledge's text.
/// Absent means the local model, offline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingsConfig {
    /// Provider id; only `openai` is implemented.
    pub provider: String,
    /// Provider-side model name, e.g. `text-embedding-3-small`.
    pub model: String,
}
```
<!-- /sokf:include -->

## Behaviour

### Sources and precedence

A setting comes from exactly one of four sources:

1. A command-line flag, where the command defines one. `validate` takes
   `--knowledge <DIR>` and `--repo-root <DIR>`, and `run begin` and
   `run advance` take `--session <ID>`; the flags themselves are
   defined by
   [contract-002-cli-superdev][sokf:contract-002-cli-superdev].
2. The environment, for four variables and nothing else.
   `OPENAI_API_KEY` is the key for the embedding API.
   `CLAUDE_PROJECT_DIR` is the repository `hook validate` and `hook
   run` resolve against; Claude Code sets it. `CLAUDE_SESSION_ID` is
   the session `run begin` and `run advance` take when `--session` is
   absent; Claude Code sets it. `XDG_CACHE_HOME` is the parent of the
   user-level model cache.
3. `.superdev/config.toml`, the manifest, hand-edited and committed,
   for everything the Definition declares.
4. The built-in defaults: the registry default per capability, and the
   pack compiled into the binary.

The four sources are disjoint, which is the promise that matters: a
value's source is decided by which of the four defines it, not by a
precedence order. The one deliberate exception runs the other way — the
embedding API key comes from the environment alone, as Secrets says
(`P_key-from-environment`, `P_key-not-from-manifest`). No value is
cached between commands.

- `P_one-source-per-setting` [ubiquitous] superdev SHALL NOT read a
  setting from two sources, so nothing silently overrides anything.
- `P_read-afresh-per-run` [ubiquitous] superdev SHALL read both the
  manifest and the environment afresh on every run.

### Defaults

`blueprint` is required and has no default; `init` writes it. A
capability table's `provider` is required within the table.
`OPENAI_API_KEY` has no default and is required only with
`[knowledge.embeddings]`; `CLAUDE_SESSION_ID` has no default and is
required only by `run begin` and `run advance` without `--session`.
Every other setting has a default; `init` writes the default
`[[packs]]` entry rather than leaving it out, since both resolve alike
and the written pin is the one a reader can see and edit.

| Setting | Absent means |
|---------|--------------|
| `[[packs]]` | the pack embedded in the binary |
| a capability table | the capability is disabled |
| `version` in a capability table | the registry default, for a capability that takes one |
| `custom` | no skill is released from management |
| `[knowledge.embeddings]` | embedding is local and offline |
| `[template]` | the repo was never seeded |
| `CLAUDE_PROJECT_DIR` | the working directory |
| `XDG_CACHE_HOME` | `%LOCALAPPDATA%`, else `~/.cache` |

### Secrets

`OPENAI_API_KEY` is the only credential superdev reads, and only when
`[knowledge.embeddings]` opts the index onto an API. A pack source
needing credentials fails rather than waiting for someone to type.

- `P_key-from-environment` [ubiquitous] superdev SHALL read
  `OPENAI_API_KEY` from the environment.
- `P_key-not-from-manifest` [ubiquitous] superdev SHALL NOT read
  `OPENAI_API_KEY` from the manifest, so it cannot reach a commit.
- `P_git-never-prompts` [ubiquitous] A git call superdev makes SHALL
  NOT prompt for credentials.
- `P_no-other-credential-path` [ubiquitous] A credential SHALL NOT
  enter superdev by any path other than `OPENAI_API_KEY`.

### Validation

The manifest is the user's. An unknown key inside a known table other
than `[knowledge]` is ignored, so a manifest written for a later
superdev still loads.

- `P_unknown-manifest-fails-at-load` [event] WHEN superdev cannot
  understand a manifest, `parse` SHALL fail at load naming the edit to
  make.
- `P_manifest-never-rewritten` [ubiquitous] `parse` SHALL NOT rewrite
  the manifest.
- `P_unknown-table-fails` [event] WHEN a top-level table names a
  capability the registry does not carry, `parse` SHALL fail as an
  unknown capability.
- `P_unknown-provider-fails` [event] WHEN a `provider` names one the
  registry does not carry, `parse` SHALL fail with `<capability>
  provider must be one of: …`.
- `P_removed-capability-fails` [event] WHEN a manifest still names the
  removed `workflows` or `bash-output-filter` capability, `parse` SHALL
  fail naming the table to delete.
- `P_unknown-knowledge-key-fails` [event] WHEN `[knowledge]` carries a
  key superdev does not know, `parse` SHALL fail.

## Stability

Unreleased. What holds even so: `P_unknown-manifest-fails-at-load` and
`P_manifest-never-rewritten`.

- `P_unreleased` [ubiquitous] Key names, defaults and the variables
  above MAY change without notice.

<!-- sokf:links -->
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:configuration]: /knowledge/configuration.md
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md

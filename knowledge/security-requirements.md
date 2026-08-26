---
type: Policy
id: security-requirements
title: Security Requirements
description: The vulnerability policy in brief, and the security-relevant guarantees the design makes.
status: stable
sources:
  - id: security-md
    resource: /SECURITY.md
    title: Security policy
---

The full policy is [SECURITY.md](/SECURITY.md).[^security-md] In brief:
vulnerabilities are reported privately via GitHub's private vulnerability
reporting, never as public issues; fixes target the latest release and `main`
only (pre-1.0, no backports).

# Guarantees the design makes

- **Pinned installs are checksummed.** The code-index bundle installs only
  the registry version this binary carries, verified against its compiled-in
  sha256 — superdev refuses any other version rather than fetch unverified
  content ([architecture](architecture.md)).
- **No secrets in files.** The embeddings API key comes from
  `OPENAI_API_KEY` and is never read from or written to the manifest
  ([configuration](configuration.md)). Publishing uses npm trusted
  publishing (OIDC) — no long-lived npm token exists; only crates.io still
  holds a token, as a CI secret.
- **Destructive writes are recoverable.** Every file superdev overwrites or
  removes is backed up under `.superdev/cache/backup/<timestamp>/` first,
  and a failed apply unwinds ([configuration](configuration.md)).
- **The MCP surface is read-only.** `superdev mcp aokf` exposes four
  read-only tools over stdio; nothing writes through it
  ([api-contracts](api-contracts.md)).
- **A pinned pack applies the bytes it was pinned to, or none.** Every
  resolved pack is verified against the digest the lock recorded for that
  rev — over paths as well as contents, so a rename is a different pack, and
  fetched with `core.autocrlf` overridden so the same rev digests the same on
  every platform — and
  a mismatch fails the run writing nothing, with no flag to accept it. A tag
  that moved is the case this exists for: the user re-pins, which is itself
  the new trust decision. A git source is fetched by spawning the user's own
  `git` ([ADR-007](decisions/D007-git-fetch-by-spawn.md)), so credentials,
  ssh agents and forge access are theirs; superdev stores no token and adds
  no auth surface.
- **A pack source names one repository, whichever way it is spelled.** The
  key that decides whether a pack replaces the embedded content or merely
  layers over it is the source with its scheme, userinfo, port, `.git`
  suffix and trailing slash removed
  ([ADR-004](decisions/D004-base-pack-identity.md)). Userinfo and the port
  are stripped from the authority alone, never from the path: a source whose
  *path* ended `@github.com/six5536/superdev` must not normalise to the
  default pack's key and be treated as the base. A pack declares no
  executable action, and the instruction files, the AOKF spec and
  `PROJECT.md` are refused by path before any file is read.
- **A pack source cannot choose what runs, or what it arrives over.** A
  manifest arrives with a repository, so `source` is not superdev's to trust.
  A source may name only `https`, `ssh` or `file`, and a `<name>::<address>`
  remote helper is refused as one whatever its address, since a helper names a
  program rather than a transport. `PackSource::parse` refuses the rest before
  anything spawns, naming the source and the transport, and no config on the
  machine can lift that. Every git call is then built by one function carrying
  `-c protocol.allow=never`, the same three admitted explicitly, and `never`
  naming `git`, `http` and `ext`. Callers pass the verb and its operands and
  cannot omit the overrides, because they never assemble the vector. A source
  or rev beginning with `-` is refused when the manifest is parsed, and `--`
  precedes every operand, so a value's shape decides nothing.

  Both halves are load-bearing and neither is sufficient
  ([ADR-012](decisions/D012-pack-source-schemes-are-allowlisted.md)). `parse`
  cannot see a `url.<base>.insteadOf` rewrite, which turns an approved
  `https://` source into whatever the machine's config says after superdev has
  handed it over. And among the overrides only the named `never` lines are
  beyond a user config's reach: git resolves `protocol.<name>.allow` ahead of
  `protocol.allow` whatever their sources, so the blanket closes the helpers
  the machine has *not* named and nothing more. What is left uncovered is a
  machine whose own config both admits a transport by name and rewrites URLs
  into it, which needs no manifest and is not a boundary superdev defends.
- **Local by default.** The CLI takes no network input at runtime; the
  network is touched only for pinned tool installs, the one-time
  embedding model download (or the explicit embeddings-API opt-in), fetching
  a pack the lock does not already have cached, and the one query `update`
  makes of the default pack source
  ([ADR-009](decisions/D009-update-queries-default-source.md)). That query is
  the narrowest of these: it runs on the untargeted `update` alone, asks only
  the source superdev itself ships, and a failure to reach it degrades to the
  pin the binary carries rather than failing the run. It is also the only
  spawn superdev bounds — a few seconds, because it is the only one made on
  superdev's own initiative
  ([ADR-015](decisions/D015-the-spawn-seam-carries-a-deadline.md)); a clone,
  a toolchain install and the model download are all things the user asked
  for and are left to take as long as they take. Every git call carries
  `GIT_TERMINAL_PROMPT=0`, so none can stop for a credential prompt. The
  accepted non-PIE musl binaries rest on exactly this — see
  [constraints-non-goals](constraints-non-goals.md).

[^security-md]: Security policy

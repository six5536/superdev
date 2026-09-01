---
type: Schema
id: schema-contract-deployment
title: Deployment Contract Schema
description: What a deployer must provide to run the software — the artifact, the runtime it needs, its health and lifecycle, and the stability promise, a public contract.
---

# Deployment Contract Schema

Structural rules for one public deployment contract, filed at
`contract-{nnn}-deployment-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. What is published, what the runtime must give
it, and how an orchestrator knows it is alive.

The boundary with the [configuration contract][sokf:schema-contract-config] is shape
against value. This document says the process listens on one HTTP port and
needs a Postgres it can reach; that one names the setting that carries the port
number, its type and its default.

<!-- sokf:include contract-style -->
**Contract style — a contract defines its interface** (superdev
ADR-033, ADR-036):

- A contract MUST define every element a caller depends on in the
  structured form this schema declares, so a caller reproduces the
  interface from the contract alone.
- Prose MUST describe and MUST NOT define. Each normative statement
  outside the definition form MUST use an RFC 2119 modal verb, one
  requirement per sentence.
- A contract MUST bind what it names and MUST NOT state how the
  interface is built inside.
- The project MUST bind this contract to its implementation, by
  generating the surface from it or by a test where the implementation
  is hand-written; a committed generated artifact MUST be proved
  current.
- A contract MUST link the ADR behind each decision and MUST NOT
  restate the ADR's reasoning.
<!-- /sokf:include -->

````yaml
description: >
  One deployable unit — what is published and where, what the runtime must
  provide, how an orchestrator starts and stops it, and what is promised
  stable to whoever operates it.
line-limit: 400

frontmatter:
  type:
    required: true
    const: DeploymentContract
  id:
    required: true
    pattern: '^contract-\d{3}-deployment-[a-z0-9-]+$'
    description: >
      contract-{nnn}-deployment-{slug}, the slug naming which deployable. The
      number is the next free one across every contract, public and
      internal together and every lifecycle folder — a duplicate is
      an error.
  title:
    required: true
  description:
    required: true
  lifecycle:
    enum: [active, deprecated]

sections-ordered: true
sections:
  - heading-pattern: '^Deployment contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the surface this contract binds and for whom —
      link the ADRs behind it.
  - heading: "Artifact"
    level: 2
    required: true
    content: prose
    description: >
      What is published — image, package, archive — to which registry, under
      what tag scheme, for which platforms, and how a deployer verifies it is
      the one this project built.
  - heading: "Runtime"
    level: 2
    required: true
    content: code
    block-language: yaml
    block-keys: [ports, user, writable, resources, depends-on]
    description: >
      The definition of what the artifact needs to run: `ports` as a map of
      port to purpose, `user` the uid it runs as, `writable` the paths it must
      be able to write, `resources` the floor below which it fails rather than
      degrades, and `depends-on` the services it cannot start without. A
      deployer writes the orchestration from this block alone; prose around it
      describes and never defines.
  - heading: "Health and lifecycle"
    level: 2
    required: true
    content: bullet-list
    item-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      How an orchestrator drives it — the readiness and liveness checks and
      what each actually tests, startup ordering, the shutdown signal and the
      grace period, and what an in-flight request sees during a restart.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      Which of the above an operator may build automation on, and how a change
      to a port, a path or a health endpoint is announced.

example: |
  ---
  type: DeploymentContract
  id: contract-001-deployment-widget-api
  title: Deployment Contract
  description: The widget API container — what it publishes and what it needs to run.
  lifecycle: active
  ---

  # Deployment contract: widget API

  The widget API container: what it publishes and what it needs to run.

  ## Artifact

  A container image at `ghcr.io/example/widget-api`, published for
  `linux/amd64` and `linux/arm64`. Tags are the release version, plus `major`
  and `major.minor` moving tags; there is no `latest`. Every image is signed
  with cosign against the repository's OIDC identity, and a deployer that does
  not verify the signature is trusting the registry instead.

  ## Runtime

  ```yaml
  ports:
    8080: HTTP
    9090: metrics
  user: 65532
  writable: [/tmp]
  resources:
    memory: 256Mi
    cpu: 0.25
    memory-floor: 128Mi
  depends-on: [postgres]
  ```

  The root filesystem MAY be mounted read-only, since nothing outside `/tmp`
  is written. Below the memory floor the process MUST fail at startup rather
  than thrashing, and it MUST NOT start without a reachable Postgres — a
  missing one is a startup error, not a degraded mode.

  ## Health and lifecycle

  - `GET /healthz` is liveness: the process answers, and the probe MUST touch
    nothing else, so a slow database never restarts a healthy process.
  - `GET /readyz` is readiness: the database answers a ping and migrations are
    applied. It MUST fail during a migration, which is what keeps traffic
    away.
  - Startup runs migrations before listening, so an orchestrator MUST allow
    sixty seconds before the first readiness probe.
  - `SIGTERM` MUST stop new connections and drain for thirty seconds;
    requests in flight finish, and anything still running at the deadline is
    cut.

  ## Stability

  The ports, the two endpoint paths and the shutdown behaviour are stable
  within a major version — automation MAY depend on them. The base image and
  the internal filesystem layout are not, and MAY change in any release. A
  changed port or path MUST be announced one minor release ahead in the
  release notes.
````

<!-- sokf:links -->
[sokf:schema-contract-config]: /knowledge/schemas/contract-config.md

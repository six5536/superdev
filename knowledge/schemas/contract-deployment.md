---
type: Schema
id: schema-contract-deployment
title: Deployment Contract Schema
description: What a deployer must provide to run the software — the artifact, the runtime it needs, its health and lifecycle, and the stability promise, in knowledge/contracts/public/.
---

# Deployment Contract Schema

Structural rules for one public deployment contract, filed at
`knowledge/contracts/public/contract-{nnn}-deployment-{slug}.md`. What is published, what the runtime must give
it, and how an orchestrator knows it is alive.

The boundary with the [configuration contract](contract-config.md) is shape
against value. This document says the process listens on one HTTP port and
needs a Postgres it can reach; that one names the setting that carries the port
number, its type and its default.

````yaml
description: >
  One deployable unit — what is published and where, what the runtime must
  provide, how an orchestrator starts and stops it, and what is promised
  stable to whoever operates it.
line-limit: 400

frontmatter:
  type:
    const: DeploymentContract
  id:
    pattern: '^contract-\d{3}-deployment-[a-z0-9-]+$'
    description: >
      contract-{nnn}-deployment-{slug}, the slug naming which deployable. The
      number is the next free one across knowledge/contracts/, public and
      private together.
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading: "Artifact"
    level: 1
    required: true
    content: prose
    description: >
      What is published — image, package, archive — to which registry, under
      what tag scheme, for which platforms, and how a deployer verifies it is
      the one this project built.
  - heading: "Runtime"
    level: 1
    required: true
    content: prose
    description: >
      What the artifact needs to run: ports it listens on, the user it runs as,
      writable paths, the resource floor below which it fails rather than
      degrades, and the services it cannot start without.
  - heading: "Health and lifecycle"
    level: 1
    required: true
    content: bullet-list
    description: >
      How an orchestrator drives it — the readiness and liveness checks and
      what each actually tests, startup ordering, the shutdown signal and the
      grace period, and what an in-flight request sees during a restart.
  - heading: "Stability"
    level: 1
    required: true
    content: prose
    description: >
      Which of the above an operator may build automation on, and how a change
      to a port, a path or a health endpoint is announced.

example: |
  ---
  type: DeploymentContract
  id: contract-001-deployment-widget-api
  title: Deployment Contract
  description: The widget API container — what it publishes and what it needs to run.
  status: stable
  ---

  # Artifact

  A container image at `ghcr.io/example/widget-api`, published for
  `linux/amd64` and `linux/arm64`. Tags are the release version, plus `major`
  and `major.minor` moving tags; there is no `latest`. Every image is signed
  with cosign against the repository's OIDC identity, and a deployer that does
  not verify the signature is trusting the registry instead.

  # Runtime

  One process, listening on `8080` for HTTP and `9090` for metrics. It runs as
  uid 65532 and needs no writable path outside `/tmp`, so the root filesystem
  can be mounted read-only. It needs 256 MiB of memory and 0.25 CPU to serve;
  below 128 MiB it fails at startup rather than thrashing. It cannot start
  without a reachable Postgres — a missing one is a startup error, not a
  degraded mode.

  # Health and lifecycle

  - `GET /healthz` is liveness: the process answers. It touches nothing else,
    so a slow database never restarts a healthy process.
  - `GET /readyz` is readiness: the database answers a ping and migrations are
    applied. It fails during a migration, which is what keeps traffic away.
  - Startup runs migrations before listening, so an orchestrator must allow
    sixty seconds before the first readiness probe.
  - `SIGTERM` stops new connections and drains for thirty seconds; requests in
    flight finish, and anything still running at the deadline is cut.

  # Stability

  The ports, the two endpoint paths and the shutdown behaviour are stable
  within a major version — automation may depend on them. The base image and
  the internal filesystem layout are not, and may change in any release. A
  changed port or path is announced one minor release ahead in the release
  notes.
````

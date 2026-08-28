---
type: Template
id: template-contract-deployment
title: Deployment Contract Template
description: Knowledge concept skeleton — what is published, what the runtime must provide, and how it starts and stops.
status: stable
---

---
type: DeploymentContract
id: contract-<nnn>-deployment-<slug>
title: Deployment Contract
description: <one line: which deployable, and what it needs>.
status: stable
---

# Artifact

<What is published — image, package, archive — to which registry, under what tag scheme, for which platforms, and how a deployer verifies it.>

# Runtime

<Ports it listens on, the user it runs as, writable paths, the resource floor below which it fails rather than degrades, and the services it cannot start without.>

# Health and lifecycle

- <Liveness: the check, and what it actually tests.>
- <Readiness: the check, and when it fails on purpose.>
- <Startup ordering, and how long before the first probe.>
- <The shutdown signal, the grace period, and what an in-flight request sees.>

# Stability

<Which of the above an operator may build automation on, and how a change to a port, a path or a health endpoint is announced.>

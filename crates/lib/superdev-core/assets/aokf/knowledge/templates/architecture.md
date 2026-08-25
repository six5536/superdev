---
type: Template
id: template-architecture
title: Architecture Template
description: Knowledge concept skeleton — the system's layers, the key subsystems, and how they fit together.
status: stable
---

---
type: Reference
id: architecture
title: Architecture
description: <one line: the layering and the key subsystems>.
status: stable
---

<The system's shape in one short paragraph: the top-level layers or components and the direction of dependency between them. Link the decision records that set this shape.>

- <Layer or component> — <what it owns and what it must not know about.>
- <Layer or component> — <...>

# <Subsystem>

<One heading per subsystem that needs more than a line: what it does, how it talks to the rest, and where its detail lives.>

# Files and artefacts

<What the system reads and writes at runtime or install time, and who owns each — enough that a reader knows what is safe to touch.>

---
type: Template
id: template-definition-of-done
title: Definition of Done Template
description: Knowledge concept skeleton — what a change must satisfy before it merges.
status: stable
---

---
type: Convention
id: definition-of-done
title: Definition of Done
description: What a change must satisfy before it merges.
status: stable
---

A change is done when:

- <Gate: the checks that must pass — format, lint, tests, types.>
- <Gate: coverage or review requirements.>
- <Gate: documentation updated wherever behaviour changed.>
- <Gate: new behaviour carries tests; bug fixes carry a regression test that fails on the unfixed code.>

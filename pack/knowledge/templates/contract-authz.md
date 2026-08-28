---
type: Template
id: template-contract-authz
title: Authorisation Contract Template
description: Knowledge concept skeleton — the principals, the role and scope vocabulary, the permissions and the boundary every surface enforces.
status: stable
---

---
type: AuthzContract
id: contract-<nnn>-authz-<slug>
title: Authorisation Contract
description: <one line: who may do what>.
status: stable
---

# Principals

<Who or what can act, and where each one's identity comes from. Name the surface contract that establishes it rather than restating the authentication.>

# Roles and scopes

| Name | Kind | Meaning |
|------|------|---------|
| `<name>` | <role|scope> | <what holding it means> |

# Permissions

| Action | Resource | Requires |
|--------|----------|----------|
| <action> | <resource> | <the roles and scopes it needs> |

# Boundaries

<The tenancy or ownership rule that cuts across every permission, where it is enforced, and what a refused caller is told — a refusal that confirms the resource exists is a disclosure.>

# Stability

<Which names are promised, what happens to a token carrying a scope that no longer exists, and how a permission is tightened without locking out callers mid-release.>

# Internal Contracts

The contracts between modules inside this repository: durable, keyed to an
interface, updated by CONTRACT-DESIGN as features change them, and freely
changeable so long as the modules on both sides move together. The
decisions behind them remain in the ADRs.

* [Pack Resolution Interface Contract][sokf:contract-007-interface-pack-resolution] - the internal interfaces that carry external content to components — pack source identity, the item model, the resolved content set, the resolution phase, the pin update proves, the process seam, and the Ctx that keeps planning pure.

<!-- sokf:links -->
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md

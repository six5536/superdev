# Issues

## Externally sourced content packs

* [update can move a pin to a pack format this binary cannot read, and cannot move it back](I001-update-can-pin-an-unreadable-pack-format.md) - update persists the moved pin before sync validates it, and a pin never moves backwards, so a content release in a newer format leaves every later sync and update failing until the manifest is hand-edited.
* [The default-source query has no time bound, so a black-holed network stalls update](I002-no-time-bound-on-the-update-query.md) - update now runs git ls-remote on every untargeted invocation, and CommandRunner has no timeout, so a network that neither answers nor refuses stalls the command for as long as the OS takes to give up.

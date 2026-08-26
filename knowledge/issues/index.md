# Issues

## Externally sourced content packs

* [update can move a pin to a pack format this binary cannot read, and cannot move it back](I001-update-can-pin-an-unreadable-pack-format.md) - update persists the moved pin before sync validates it, and a pin never moves backwards, so a content release in a newer format leaves every later sync and update failing until the manifest is hand-edited.
* [The default-source query has no time bound, so a black-holed network stalls update](I002-no-time-bound-on-the-update-query.md) - update now runs git ls-remote on every untargeted invocation, and CommandRunner has no timeout, so a network that neither answers nor refuses stalls the command for as long as the OS takes to give up.
* [Deleting an item from a local pack leaves its live copy in place, and the drift check stays green](I003-a-local-pack-cannot-remove-what-it-dropped.md) - a path pack layers rather than replacing, so an item deleted or renamed under pack/ is still written from the embedded snapshot; sync reports nothing and status --drift exits 0 until the binary is rebuilt.
* [A path pack's lock digest is rewritten by every content commit and verified by nothing](I004-a-path-packs-digest-churns-and-is-never-checked.md) - the lock records a digest over a path pack's whole tree that resolution never checks, so every commit touching pack/ rewrites the same line and a commit made without sync leaves a wrong digest nothing detects.

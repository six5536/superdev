---
type: Decision
id: adr-012-pack-source-schemes-are-allowlisted
title: A Pack Source's Transport Is Allowlisted
description: A pack source may only name https, ssh or file, refused at parse before anything spawns and refused again by git's own protocol policy, because a manifest arrives with a repository and a transport nobody vetted must not inherit the base pack's standing.
lifecycle: active
links:
  - rel: references
    to: contract-007-interface-pack-resolution
  - rel: relates-to
    to: adr-004-base-pack-identity
  - rel: relates-to
    to: security-requirements
---

# ADR-012: A pack source's transport is allowlisted

- Date: 2026-08-26
- Deciders: project owner

## Context

`[[packs]].source` is handed to `git clone` as the URL. [C007][sokf:contract-007-interface-pack-resolution] makes naming
a source the user's trust decision; it says nothing about the transport that
source is reached over.
[ADR-004][sokf:adr-004-base-pack-identity] makes the comparison key deliberately
blind to the scheme, so that one repository written four ways is one source —
and so that an entry naming superdev's own repository *replaces* the embedded
content rather than layering over it.

Blind to the scheme means blind to a bad one. This manifest keys as the base
pack:

```toml
[[packs]]
source = "git://github.com/six5536/superdev"
rev    = "assets-v0.2.0"
```

`git://` has no authentication and no integrity, so anyone on the path
answers it, and what they answer with replaces superdev's own skills and
templates. `http://` is the same. A manifest is a committed file that arrives
with a repository, so this is reachable by cloning a branch and running
`sync`.

[I007][sokf:issue-007-a-pack-source-reaches-git-with-no-scheme-check]'s
command-execution half is closed: every git call carries
`-c protocol.ext.allow=never`, so an `ext::` URL — which names a command git
runs as the connection — runs nothing whatever the machine is configured to
permit. That closes one helper by name. It cannot close a set that is whatever
`git-remote-*` binaries happen to be on PATH, and it says nothing about
`git://`, which is a built-in transport and needs no helper at all.

The remaining half narrows what a manifest may say, which
[C007][sokf:contract-007-interface-pack-resolution] documents as `parse`'s rejections.
That is why it stayed unscheduled while the other landed.

## Decision

We will allowlist the transports a pack source may name, in both places it
can be done.

**At parse.** `PackSource::parse` refuses a source whose transport is not one
of `https`, `ssh` or `file`, naming the source and the transport. A source of
the form `<name>::<address>` is a git remote helper and is refused as one,
`ext::` included, before its scheme is even considered. Everything else keeps
working exactly as it does: the `github:`/`gitlab:` shorthand expands to
`https`, and the scp form — `git@github.com:owner/repo`, or a bare ssh alias
— is ssh, which git has always treated it as.

```rust
/// The transports a pack may be fetched over. Anything else is refused by
/// `PackSource::parse`, naming the source: an unauthenticated transport
/// would let anyone on the path answer for the base pack, and a remote
/// helper names a program rather than a protocol. ADR-012, I007.
pub const SUPPORTED_SCHEMES: &[&str] = &["https", "ssh", "file"];
```

**At the spawn.** The overrides every git call is built from become
`-c protocol.allow=never`, `-c protocol.<name>.allow=always` for exactly that
set, and `-c protocol.<name>.allow=never` naming `git`, `http` and `ext`.
Where the parse check refuses what a manifest may *say*, this refuses what
git may *do*.

The named refusals are not redundant alongside the blanket, and which half is
load-bearing is the opposite of what it looks like. Git resolves
`protocol.<name>.allow` ahead of `protocol.allow` regardless of where each
came from, so **a named protocol in the user's own config beats a blanket on
superdev's command line**. Measured on git 2.51.1, against a user config
carrying `protocol.ext.allow = always`:

| superdev's override | result |
|---------------------|--------|
| `-c protocol.allow=never` | `ext::touch pwned` **ran the command** |
| `-c protocol.ext.allow=never` | `fatal: transport 'ext' not allowed` |

The same user config with `protocol.git.allow = always` gets `git://` past the
blanket too. So `protocol.ext.allow=never` is the line that closes `ext`, not
a belt beside the braces, and `git` and `http` are named for the same reason.
The blanket keeps its place: it closes every helper the user has *not* named,
which is nearly all of them on nearly every machine.

Neither half is sufficient, and they fail in different directions.

`parse` bounds what a *manifest* may say, and nothing on the machine can lift
it — but it does not see what git connects to. `url.<base>.insteadOf` rewrites
a URL after superdev has handed it over, so a plain `https://` source that
`parse` approves can be turned into an `ext::` command by the user's own
config. Measured, same git:

| superdev's override | `https://…` rewritten to `ext::` by `insteadOf` |
|---------------------|--------------------------------------------------|
| `-c protocol.allow=never` | **ran the command** |
| `-c protocol.ext.allow=never` | `fatal: transport 'ext' not allowed` |

The overrides see the post-rewrite URL, which is the only place that
substitution is visible — but among them only the explicitly named `never`
lines are beyond a user config's reach.

So the guarantee is the conjunction: `parse` for what the manifest may name,
the named refusals for what git may end up doing with it, and the blanket for
every helper the machine has not spoken about. What remains uncovered is a
machine whose own config both admits a protocol by name and rewrites URLs into
it — at which point the config is doing it to itself and a manifest is not
needed.

Identity keeps ignoring the scheme. It no longer has a wrong answer to give,
because a source with an unsupported transport never reaches it.

`file` stays in the set: a bare repository on this machine is a legitimate
source and the way a pack is tested before it is published. superdev clones
without `--recurse-submodules`, so the local-transport submodule hazard that
made git default `protocol.file.allow` to `user` has no path here.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Allowlist at parse and at the spawn | The error names the source and arrives before anything is spawned; git refuses independently, so an unanticipated spelling is still covered; two locks on one door, neither load-bearing alone | Narrows what a manifest may say; two places to keep in step |
| Git's protocol policy alone | One place, no parse change | `git://` still keys as the base pack, and the error is git's rather than superdev's; worse, a user config naming a protocol outranks a blanket, so the open-ended half of the policy is exactly the half a permissive machine lifts |
| Parse alone | Best error, and the half nothing on the machine can lift | Rests on superdev's recognition of a URL being exhaustive, which is the assumption that produced this issue — and cannot see a URL that `insteadOf` rewrites into a transport after superdev has approved it |
| Denylist the known-bad schemes | Refuses nothing legitimate | A denylist is a list of the transports someone thought of; `ext::` was already on it and `git://` was not |
| Allowlist with an escape hatch for a refused scheme | Serves an internal `http://` mirror or a git daemon | The two schemes anyone would re-admit are precisely the two with no authentication, so the hatch reopens the hole it was added around |
| Leave it | No change | Merely resolving someone else's manifest fetches the base pack over a transport anyone on-path can answer |

## Consequences

- Positive: a source cannot name a transport that lets a third party
  substitute the content that replaces superdev's own. The refusal is
  superdev's own and holds whatever the machine's git is configured to
  permit.
- Positive: the refusal is a superdev error naming the manifest's own
  spelling, not a clone failure the reader has to trace back.
- Negative: an internal mirror served over `http://`, or a `git://` daemon,
  stops working with no way to re-admit it. Deliberate — both are exactly the
  case this refuses — and the answer is `https` or a `file` path.
- Negative: two mechanisms enforce one rule, and a future scheme has to be
  added to both. The constant is the single list; the overrides read from it.
- Negative: `-c protocol.https.allow=always` overrides a user who has
  deliberately disabled https in their own config. Accepted: without it the
  blanket refuses superdev's own fetches, and a machine that cannot use https
  cannot reach the default pack anyway.
- Neutral: neither half is airtight alone. `parse` cannot see an
  `insteadOf` rewrite, and a user config naming a protocol outranks the
  blanket. Together they leave one uncovered case — a machine that both admits
  a protocol by name and rewrites URLs into it — which needs no manifest to
  exploit and so is not a boundary superdev is defending.
- Neutral: a path source is untouched. It names a directory, not a URL.
- Follow-ups: [C007][sokf:contract-007-interface-pack-resolution] gains
  `SUPPORTED_SCHEMES` and the new rejection on `parse`;
  [security-requirements][sokf:security-requirements] states the guarantee;
  [configuration][sokf:configuration] documents the accepted spellings at
  integrate.

<!-- sokf:links -->
[sokf:adr-004-base-pack-identity]: /knowledge/adrs/active/adr-004-base-pack-identity.md
[sokf:configuration]: /knowledge/configuration.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
[sokf:issue-007-a-pack-source-reaches-git-with-no-scheme-check]: /knowledge/issues/done/issue-007-a-pack-source-reaches-git-with-no-scheme-check.md
[sokf:security-requirements]: /knowledge/security-requirements.md

# SOKF changelog

Changelog for [SPEC.md](SPEC.md).

## 0.4

Breaks 0.3. A body link to a concept addresses it by `id` (§8), and a
document carrying one carries a generated definition block at its foot
(§9). A 0.3 knowledge whose body links name paths conforms to 0.3 and not
to 0.4; `superdev validate --fix` converts one to the other.

## 0.3

Breaks 0.2 twice.

- Conformance is pass or fail (§11). The three-level ladder is gone: a
  knowledge satisfies every rule or it does not conform.
- The format is SOKF, the Superdev Open Knowledge Format, renamed from
  AOKF. The manifest key is `sokf` and the reserved file is
  `manifest.sokf.yaml`; the 0.2 names `aokf` and `manifest.aokf.yaml` are
  not read.

## 0.2

Adds `implements` / `implemented-by` to the core relationship vocabulary
(§8), for a plan or issue that delivers a spec. Not a break: a 0.1
consumer reads the unknown `rel` as `relates-to`.

## 0.1

First specification.

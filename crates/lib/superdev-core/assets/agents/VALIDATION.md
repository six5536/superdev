# Validation

After any change under `knowledge/`, run the AOKF validator and fix every
error before moving on:

```
superdev aokf validate knowledge
```

It checks the bundle against `.agents/aokf/SPEC.md` (document check plus the
conformance ladder) and must PASS at level 2. Warnings don't fail the run but
usually mean a rename the bundle missed; fix the reference, not the target.

Nothing runs it for you: superdev installs no hook, so run the command by hand
after every change under `knowledge/`, whether you made it by editor, script
or agent.

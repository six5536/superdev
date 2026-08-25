# Process: Security review

Extends the general review process (`code-review.md`) with an attacker's lens. Output per `templates/security-review.md`.

## 1. Establish the threat model first

- Name the attacker and their reach before reading code: anonymous internet user, authenticated user, malicious input file, compromised dependency? A "vulnerability" no attacker can reach is a hardening note, not a finding.
- Identify the assets: what data or capability is worth protecting here.

## 2. Trace inputs, not files

- Enumerate every point where attacker-influenced data enters the changed code: request params, headers, file contents, filenames, env vars, data previously stored by users.
- Follow each input to its sinks: queries, shell commands, file paths, HTML output, deserialization, redirects, eval-like constructs. The finding lives where tainted data meets a dangerous sink without sanitization *for that sink*.

## 3. Check the boundaries systematically

- **Authn/authz:** is every new endpoint/operation gated? Does authorization check the *object* ("may this user access order 123?"), not just the role? Any check done client-side only?
- **Secrets:** credentials in code, logs, error messages, URLs, or committed config.
- **Data exposure:** responses returning more fields than the consumer needs; verbose errors leaking internals; missing redaction in logs.
- **Crypto & randomness:** home-rolled crypto, non-cryptographic randomness for tokens, deprecated algorithms.
- **Defaults & config:** new options that are insecure by default (open CORS, debug on, permissive parsers).
- **Dependencies:** new packages this change adds — see `dependency-changes.md` vetting.

## 4. Prove reachability before reporting

- For each suspected issue, construct the concrete attack: what the attacker sends, the path it takes, what they gain. Check the surrounding code first — the sanitization may live one layer up.
- Severity comes from impact × reachability, not from how scary the vulnerability class sounds.

## 5. Report responsibly

- Findings ranked by severity, each with attack scenario, impact, and specific remediation — the safe pattern, not just "sanitize this".
- Include the "checked and sound" list so coverage is visible, and keep exploit detail proportionate: enough for the team to reproduce and fix, in the project's private channels — not a published how-to.

# Agent retrieval baseline — 2026-10-24

Implemented `reader/retrieval-check.mjs`, its 15 fixture tests and usage notes.
GPT-6.1 Sol implemented and independently reviewed; the parent reviewed and
integrated repairs; Luna 6 ran final deterministic checks. No production
publication, corpus, dependencies or hosting settings changed.

## Results

Pinned live release:
`c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969`.

| Run | Result | Discovery | Integrity verification |
|---|---|---|---|
| Node default client | Pass | 5 GET + 5 HEAD; 24,051 body bytes | 3 GET + 3 HEAD; 408,761 body bytes; 6 artifacts verified |
| Same Node client, Python-urllib/3.9 user agent | Fail |llms.txt GET/HEAD 403; 7,154 body bytes |Not reached |
| Agent web retrieval, discovered Article 35 Markdown URL | Tool error |No origin status exposed |Not reached |

The passing traversal discovers Article 35 and 39 links from the published index,
fetches Markdown, verifies the manifest content ID and fetched artifact hashes,
then retrieves their exact-text companions. Exact text is 1,909 and4,975 bytes
respectively. The body totals above are decoded HTTP payloads, not wire traffic,
tokens, or model billing. HEAD is diagnostic overhead, not a minimum retrieval
requirement. Most verification bytes are the full manifest: it should not be
mandatory context for an answering agent. Discovery alone takes 5 GETs for the
two articles; compact direct citation resolution can reduce that cost.

The 403s include Cloudflare headers. A separate curl comparison against the
current index reproduced 200 with the default user agent and 403 with the Python
user agent. This supports a client-header-dependent denial, not a conclusion
about the responsible rule. The two Node runs share a network and implementation;
they are not two independent networks. The agent tool failure has no HTTP status
and must not be equated with the observed 403.

The checker enforces request/time/byte limits, same-origin pinned traversal,
exact index-label matching, manifest binding and requested-artifact hash checks.
It rejects a within-release redirect to a different article when the original
artifact's bytes do not match. HEAD 405 remains a recorded error while GET
discovery continues. The evidence proves artifact transport fidelity; it does
not independently approve legal identity, current applicability or answer quality.

## Validation and limits

- 40/40 Reader Node tests pass, including 15 new retrieval fixtures.
- Syntax and tracked diff whitespace checks pass.
- Sandbox loopback tests required escalation after EPERM; final tests passed.
- Full-manifest asset download, Rust builds, token measurements, independent
  network runs and human legal-answer approval were not performed.
- Native Rust checks remain required before a code commit under repository
  instructions; no commit was requested or made in this turn.

Source hashes, sanitized request evidence, exact URLs and test-log digest are in
`agent-retrieval-baseline-2026-10-24-evidence.json`. Bodies and cookies are absent.
The full local test log is `/tmp/lex-retrieval-tests.log`.

## Owner-account follow-up

Use the failed request timestamps and `cf-ray` values in the evidence to inspect
the existing host's security events and identify the matched rule. Confirm
whether the intended public static text routes are being denied. Any proposed
exception must be scoped to those public routes and verified with the same
default/denied client probes; do not disable protection across the host or create
a replacement site. This turn did not inspect owner-account security settings.

Then repeat traversal from an independent network and the actual failing agent
client. M1 tooling is complete; cross-client acceptance remains open. M2/M3 are
specified in `plans/agent-retrieval-optimization.md`, with existing DB reuse and
release binding required before public service integration. The windowed-reader
worktree stays deferred until its implementation starts.

# Database agent query — host handoff (D4), 2026-10-25

**For the website owner to review. Nothing is deployed and no host setting
changed.** Corrected 2026-10-25 after the independent D3 review: the website host is
the owner's Workers Free Worker, now live in production with observation
pending, not the GPT Sites Worker. This handoff concerns a validated local adapter only. It does not
broaden the public scope beyond the four instruments of public release
`c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969`.

## What exists

- Rust adapter `lex_db::public` with CLI entry points `lex-mex db public-bind`
  and `lex-mex db public-query`. The query command reads JSON lines on stdin
  and writes JSON lines on stdout.
- Contract: `schemas/public-query-response-v1.schema.json` as accepted on
  2026-10-25. **Amended 2026-11-17:** the adapter now emits
  `schemas/public-query-response-v2.schema.json` (`public-query-response-v2`),
  which adds `skip_cursor` to the `response_too_large` error. The v1 file is
  unchanged and no longer what the adapter emits; the acceptance evidence JSON pins its hash.
  Every response is validated against the emitted schema before it is sent. The
  binding is checked against `schemas/public-query-binding-v1.schema.json`.
- Acceptance evidence: `docs/database-agent-query-acceptance-2026-10-25.md`.

## Exact bindings

| Item | Value |
|---|---|
| Accepted DB release | `afdc5acf0b5017d73cd7b8e0b8f8fc5962914b56de26467e42b0c77fd2ef1974` |
| DB physical SHA-256 | `5da6e800e4d3b362411c561d9ae67fbfe012abe3164fce30bbb8a21c9e249bf2` (482,025,472 bytes) |
| DB logical SHA-256 | `8cc3736018bffae94f303d9cbfa378170c9ff0e2842ce708431f08d528b60359` |
| DB corpus/builder commit | `5eca03bc5085de423ae23b1f1519712604bd28e2` |
| Public release | `c305db4a3cfb408b76c13431e26beda72d5ce665046c95783fab36507354c969`, manifest SHA-256 `e3c8b096b41e5cdb231172def8cc6cd3f7d2b9a84613456d06933b28b969dec9`, corpus commit `d2ab64d61ecfe103cc2a2980c5d04ed63c20bb7a` |
| Binding file | `derived/public-query/c305db4a-afdc5acf.binding.json` (Git-ignored); SHA-256 recorded in the acceptance evidence JSON |
| Projection version | `public-query-v1` |

The public release and the DB come from different commits. They are bound
because all four instruments' canonical input files and all 870 provision
texts are byte-identical between them. Matching commit labels was not the test.

## Target host and runtime decision (corrected after D3 review)

**Website status (owner's report, read-only).** Production is live on
Cloudflare Workers Free; observation is still pending. Source: the website
owner's `HANDOFF.md` and `CLOUDFLARE-MIGRATION-PLAN.md`, committed and clean
at `41243b077b29a0bb4b02ca543f8c084060d6bdd2` in `/Users/jr/Documents/4ll4`.

- SHA-256 of `HANDOFF.md`:
  `635fa81c08390e34290e7968546cdd87fe06dcedba961ba567aabe19b88d4c67`.
- SHA-256 of `CLOUDFLARE-MIGRATION-PLAN.md`:
  `83f51c4fa4ac23e3094ec165f818890345e0c8248e9f407b001041fd9bb8b572`.

As that handoff reports it:

- canonical `www` serves from the customer-owned Workers Free Worker
  `leg4ll4bs-web`, with Static Assets, cut over 2026-10-25;
- Sites version 26 is kept as rollback;
- no R2, D1, KV or paid service was used, and hernaiz.law and mail are
  unchanged;
- a 48-hour observation gate is due 2026-10-27T06:48:02Z.

This handoff does not claim any observation result. It changes nothing on the
website, and neither website file was edited.

- **The website migration is independent of DB hosting.** The static R7
  publication (release `c305db4a…`) moved as signed bytes and is validated on
  its own, per the owner's receipt. This adapter added nothing to that
  migration and adds nothing to the live site: no API
  route, Worker binding, container, D1/R2 resource, paid plan or website source
  edit.
- **The native adapter stays local** (CLI only) until the owner chooses a
  backend runtime in a separate decision. A Worker isolate cannot run it
  unchanged: it is a native Rust binary that links SQLite and opens and hashes
  a 482 MB file at startup. Cloudflare Containers can host native applications,
  but they are a paid product outside the selected Free migration scope.
  Confirm the current limits on the Cloudflare pages cited in the D3 review;
  this handoff did not query the account.
- **Not acceptable as a workaround:** publishing the full canonical DB as a
  public download. That would bypass the adapter's instrument allowlist and
  public field filter.

Options for the later backend decision. The owner decides; none is implied by
the migration.

1. **Local or agent-side only (current state).** Operators or agents with
   filesystem access run `lex-mex db public-query` against the
   hash-verified DB, binding and binary. Nothing is hosted.
2. **A separate read-only origin**, once the owner chooses and pays for a
   runtime that can host a native process. A small HTTP shim fronts the
   process pool and is reached through one narrow route. The shim needs its
   own plan covering credentials, request mapping, body-size limit, ingress
   timeout, process-pool concurrency cap, exact bindings and rollback.
3. **Not without a new plan:** a TypeScript/D1 port or a reduced DB. Either one
   is a second implementation or a new artifact needing its own acceptance,
   and Rust would no longer own validation.

Client retrieval access is the owner's gate. The owner reports that the
default Node and Python clients pass on the own zone after an approved
Browser-Integrity-Check-only rule, and that one agent web tool still fails
without an HTTP status. Any future query route needs the same default-client
and denied-client probes.

## Operational limits (measured locally, Apple-silicon workstation)

| Limit | Value |
|---|---|
| Startup (open + binding verification, incl. hashing 482 MB) | ~1.2 s |
| Resident memory per process | ~213 MB |
| Warm latency, mixed requests (single process, 480 requests) | p50 0.24 ms, p95 18 ms, max 25 ms |
| 8 concurrent processes (1,280 requests) | p50 1.0 ms, p95 179 ms, max 225 ms |
| Per-query deadline | 2 s (SQLite progress-handler interrupt → `deadline_exceeded`) |
| stdin framing | LF-delimited; one CR before LF is part of the delimiter; ≤ 8,192 content bytes per frame. A longer frame is rejected as soon as the limit is exceeded, without buffering beyond it; its remainder is discarded through the next LF, and later frames are served. A final unterminated frame is accepted at EOF. Ingress timeouts are the host's job. |
| Response bytes | ≤ 16 KiB except `get_provision`; complete provisions up to 256 KiB (largest selected: 209,814 bytes), larger → `oversized` with artifact link |
| Page size | 1–10; excerpts 240 characters; queries 512 characters / 2,048 bytes; requests 8 KiB |

Measured numbers come from the workstation, not from the deployment
environment. Set the service ceilings (concurrency cap, timeouts, memory) after
measuring the chosen host and before public acceptance. One process handles one
request at a time, so concurrency comes from a process pool whose size the host
caps.

## Switch and rollback

1. Stage the DB release directory, binding and binary beside any existing
   service. Never overwrite an accepted DB directory or a binding; both
   commands refuse.
2. Start `public-query` against the staged files. It exits non-zero before
   serving on any mismatch: DB bytes, release ID, builder/corpus commit,
   canonical input hash, provision digest, or reference count.
3. Probe the coverage, resolve, search, get_provision and references
   operations. Compare `snapshot.public_release_id` with the static release
   pin.
4. Switch the route atomically. To roll back, point the route back, or remove
   it so the static Reader is the only surface. The static publication never
   depends on this service.

A new public release or DB release needs a new binding, made with
`public-bind`. A release that no longer matches gets an explicit
`release_unavailable` outcome, never current text.

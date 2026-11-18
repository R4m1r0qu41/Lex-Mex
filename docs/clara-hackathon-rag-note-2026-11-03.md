# Clara hackathon: bounded RAG exercise for Lex-Mex

Date: 2026-11-03. Operator discussion note; no implementation or deployment.

JR’s team placed second at the Clara hackathon. Cuenta Clara(s) demonstrated
that a conversational interface backed by tools could work well: the Telegram
bot ran from Julio’s machine and answered educational questions and entity
lookups. The repository contains n8n agent workflows on `testing` and a separate
Dockerized retrieval service on `Eugenio`; their presence does not establish
that the exported bot used that retrieval service.

The lesson for Lex-Mex is to concentrate on a small retrieval-augmented
generation (RAG) test, rather than reproduce the chatbot. The database already
exists; the practical gap is connecting accepted data to a retrieval service
that both the reader and agents can use. Earlier agent-access checks were
reported as unsuccessful or incomplete; do not count an unexecuted query as a
pass or confuse access failure with absence of legal evidence. Capture exact
queries, tools, URLs, responses and failures in the existing
[runtime-proof test pack](runtime-proof-test-pack-2026-10-28.md).

## Proposed bounded exercise

- Restrict the test to the four fintech instruments already selected for the
  reader: `lritf`, `lic`, `itf-dcg-2018`, `ifpe-dcg-2021` (not four individual
  provisions). Reuse the accepted database; no corpus expansion or rebuild.
- Connect the read-only SQLite adapter to the HTTP retrieval path. Give the
  reader and agent the same release-consistent search, full-provision lookup,
  references and coverage responses.
- Let an agent retrieve evidence, read complete relevant provisions and compose
  a cited answer. Preserve exact source text, stable IDs, provenance, release,
  review status and uncertainty. Retrieval and model output confer no legal
  approval and cannot change canonical records or audited decisions.
- Use the existing ten-question pack to measure retrieval relevance, complete
  text access, citation fidelity, qualifications and explicit insufficient
  evidence. Record which questions were actually attempted and answered.
- Borrow Clara’s tool separation, API input limits, authentication and
  health/readiness checks. Docker and n8n are candidate deployment and
  orchestration tools; they do not require replacing SQLite with PostgreSQL.
  Start with existing lexical retrieval and measure gaps before adding vectors.

The reader is hosted on Cloudflare Workers. JR received an email about Workers
settings and will supply it for review. Whether an upgrade is needed remains
unverified; inspect the deployed Worker, runtime/storage compatibility and
actual limits before choosing a paid plan or moving the database.

Next: deliver one working retrieval-to-answer path for these four instruments
and run the bounded question pack. This supplements the existing
[agent-first delivery plan](plans/agent-first-delivery-and-change-tolerant-releases.md);
it is not a new execution plan or authorization for deployment.

Recorded as a completed documentation-only task. No code, corpus, configuration,
or existing work changed; no commit or push. No new capsule, checkpoint or
separate receipt was needed for this one-step discussion note.

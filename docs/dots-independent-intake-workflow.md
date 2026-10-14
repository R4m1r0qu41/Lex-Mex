# Independent Dots workflow for Lex-Mex

Draft for JR — 2026-10-08. No schedule, background task, message, push, or
deployment has been created by this document.

The immediate objective is to turn prepared source inventories into small,
reviewable structural intake packets without making every worker wait for a
parser defect or a legal decision. Keep one integration owner and independent
workspaces; use the existing Rust pipeline for canonical output.

## Current starting point

- Canonical data checkpoint: `ebe688df2`, 257 committed instruments.
  Fourteen reviewed EC2/EC3 additions were committed separately.
- Applied packet and source intake worktree retained for evidence:
  `/private/tmp/lex-mex-intake-20261008`, branch `feat/intake-ec2-20261008`.
  Current integration evidence: `docs/ec2-ec3-integration-2026-10-09.md`.
- Reader: `/Users/jr/Dev/lex-mex-reader`, `feat/public-reader`, `4072dd53b`.
  Its four-instrument public release is distinct from the full intake corpus.
- Design: [Lex-Mex Figma](https://www.figma.com/design/ar2bES9p6jeUBY0w3E4eDY).
- EC3: six reviewed instruments integrated; LFJS held for five omitted numeric
  original transitories. Assign its fixture-backed repair separately.

These are recovery coordinates, not instructions to use stale HEADs. Each task
must verify its assigned base and current dirty state before acting.

## Flow and ownership

| Stage | Owner | Input → output | Can proceed independently of |
|---|---|---|---|
| Assign | Coordinator | Verified baseline + queue → bounded job card | Figma and read-only source research |
| Locate sources | Source scout | Named inventory entries → official-source identity evidence | Parser work and legal review |
| Structural intake | Intake worker | Approved manifest + assigned slugs → canonical candidate + retained downloads | Other workers in separate workspaces |
| Verify | Evidence reviewer | Candidate artifact packet → accept/hold findings by slug | Other instruments' failures |
| Integrate | Single integration owner | Accepted packets → reviewed candidate checkout + checks | Nothing that writes the same checkout |
| Publish | Reader release owner | Approved corpus commit + explicit release scope → immutable staged release | Further intake, after snapshot is pinned |

A held slug returns to a named parser or JRH review queue. It does not block
unrelated source-ready slugs. Structural success does not resolve legal status.

## Operating rules

Start with two intake slots and one read-only verification slot. Assign one to
three related instruments per task, then release the slot after its packet is
saved. Never run two corpus writers in the same checkout, even for different
slugs: source inventories, cross-instrument links and closure reports are shared.
Use operator-provisioned isolated worktrees or cloud checkouts per repository
rules. Assign parser changes to a separate task; intake workers report defects.

Use `--only` for named absent slugs. Preserve `.work` until verification completes.
Resume from the narrowest failed stage instead of refetching a whole successful
batch. Retry a transient download once; hold persistent source or parser failures
with evidence. Never fix an official source silently or turn off validation.
Known adapter exceptions still require source-specific evidence and parent review.

Do not run temporal model jobs, invent effective dates, freeze counts from parser
output alone, clear JRH blockers, overwrite reviewed corpora, commit, push, or
publish under these draft prompts. The integration owner requests only the
remaining explicit authorization after preparing a reviewable candidate.

## Job card required for every task

```text
Job ID: EC3-<slug>-<date>
Goal: Produce a provisional structural intake packet for <slugs>.
Base commit: <full verified SHA>
Workspace: <assigned isolated checkout; verify it exists>
Plan: docs/plans/reader-figma-intake-dots.md, SHA-256 <current digest>
Read: AGENTS.md, CODEX.md, named manifest, relevant accepted decisions only.
Allowed writes: adapters/diputados/<slugs>.json, corpus/mx/<slugs>/,
               .work/<slugs>/ and the assigned task report.
Forbidden writes: sibling corpora, schemas/parser, human review decisions,
                  main checkout, provider config, external vaults, deployment.
Prerequisites: repository access; Rust; pdftotext; official-source network access.
Verifier: hash recomputation, independent source headings/boundaries,
          lex-mex validate, Markdown and reference inspection.
Stop: complete packet saved; or a reproducible blocker with the next exact step.
```

The coordinator fills every placeholder before launching a task. Missing access
or an unassigned workspace is a blocker, not permission to use another checkout.

## Ready-to-paste coordinator prompt

```text
Coordinate the Lex-Mex intake queue using the attached verified job cards and
docs/dots-independent-intake-workflow.md. First verify repository HEAD, dirty
state, instructions and existing outputs. Start by reviewing the eight EC2
candidate instruments; do not re-ingest them. Prepare the next EC3 source-ready
jobs from the named inventory after checking actual corpus presence and source
identity. Keep blocked entries blocked. Assign at most two isolated intake
writers and one read-only evidence reviewer. Give each its complete job card,
source list and acceptance checks; do not assume it has our conversation.

Let independent jobs finish while a failed slug waits. Record every completed
packet before assigning another. Do not change parser rules or legal decisions
inside intake jobs. Give JR one consolidated result: accepted/held slugs, evidence
paths, exact checks, remaining decisions and one next action. Prepare integration
for review; do not commit, push, deploy, schedule work or message anyone else.
```

## Ready-to-paste source scout prompt

```text
Read only the inventory entries and existing corpus metadata named in your job
card. Confirm whether each slug is already present, then inspect the intended
official Cámara/DOF source. Record the official title, URL, publication metadata,
retrieval time, redirects, availability and identity mismatches. A title match or
HTTP 200 alone is insufficient. Keep current consolidated text distinct from a
formal enactment or amendment. Do not infer validity or choose between competing
legal instruments. Return source-ready or hold, with evidence, per entry. Write
only the assigned report; do not fetch into canonical corpus or edit adapters.
```

## Ready-to-paste intake worker prompt

```text
Process only the slugs in your job card, in the assigned isolated workspace.
Verify the base and instructions, confirm those corpus paths are absent, and
use the Rust lex-cli package's lex-mex binary. Read CLI help before invoking a
command whose syntax is uncertain. Unset LEX_MEX_OBSIDIAN_VAULT. Run the admitted
batch with --only <assigned-slugs> --keep-work, without --freeze-counts and
without temporal analysis. Keep failures and downloads intact. Resume a failed
stage narrowly after the integration owner approves a source-specific fix.

Return manifests, source/extracted hashes, validation reports, canonical and
Markdown paths, first/last article and transitory boundaries, suffix/gap cases,
reference warnings, review-queue presence, exact commands and exit codes. Keep
each failure separate so other jobs can continue. Do not broaden the allowlist,
edit parser/schema code, declare legal review complete, freeze counts, commit,
push or publish. Save your packet before stopping.
```

## Ready-to-paste evidence reviewer prompt

```text
Independently review only the candidate slugs in the supplied packet. Do not
modify canonical files. Recompute retained PDF and extracted-text hashes against
the manifests. Check official identity, article heading sequence and original
transitory boundaries directly in retained source text; inspect representative
PDF pages where extraction is ambiguous. Check final provisions for signatures,
page furniture and reform-tail contamination. Inspect warnings, exact reference
spans/targets, pending review state and Markdown. A passing validator does not
prove complete extraction or current legal validity. Return accept-for-structural
integration or hold per slug, specific evidence, limitations, and the next
bounded correction. Do not infer JRH approval.
```

## Ready-to-paste integration and reader prompts

```text
Integration owner: verify accepted packet bases and allowed-path diffs. Integrate
only reviewed changes into the designated candidate workspace; preserve unrelated
work. Freeze source-audited count baselines only after the independent review.
Recompute affected cross-instrument links once their targets coexist. Run all
AGENTS.md checks before a code/corpus commit, plus affected end-to-end stages.
Inspect provenance, review queues and exports. Prepare exact paths and commit
message for JR; no commit, push or deployment without current authorization.
```

```text
Reader owner: consume only a reviewed, explicitly selected corpus commit. Use the
existing immutable release producer and public-data allowlist. Verify exact text,
retained PDF hashes, publication receipt, pinned share URLs and mobile behavior.
Apply the reviewed Lex-Mex Figma design in your assigned reader checkout only.
The design's Source Serif 4 is a proposed substitute for Georgia, not an existing
production font change. Keep intake candidates and review internals out of the
public package. Stage a release for review; do not deploy automatically.
```

## Result packet and cutoff recovery

Every result records job ID, workspace, base/head, dirty paths, assigned slugs,
source hashes, stage outcomes, exact validation commands, warnings, unresolved
risks, reviewer findings and one next action. Large source texts stay in retained
work files; reports link to them. The coordinator updates the repository plan
and active capsule after a packet completes. Resume from those artifacts and
current Git evidence, never only the previous chat's last paragraph.

## Dots setup and optional recurring work

Dots tasks have their own conversations and do not automatically inherit every
prior conversation. For cloud coding, configure the repository and environment
in Codex first; selecting another computer does not transfer an existing task.
Send a complete job card and verify actual access. See official
[tasks and memory guidance](https://learn.chatgpt.com/docs/dots/tasks-and-memory).

An optional schedule could check source availability and report blocked/ready
counts on weekdays at 09:00 `America/Mexico_City`, with results delivered to JR in
ChatGPT. This is a proposal only. Save it separately, specify an end date and
notification conditions, and ask Dots to confirm the saved schedule before
relying on it. No source update should directly overwrite published corpus.

Review task activity and output, not just its completed label. Pausing the main
dot, stopping delegated tasks and canceling schedules are separate controls.
Custom rules do not grant missing app/computer access. See official
[Dots controls](https://learn.chatgpt.com/docs/dots/controls).

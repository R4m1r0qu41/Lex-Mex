# Lex-Mex execution plans

This file is the stable contract and index for long-running execution plans in
Lex-Mex. It does not track task progress itself and does not authorize work
beyond the operator's request and `AGENTS.md`.

## When a plan is required

Use a task-named execution plan when work is expected to cross sessions or
contributors, contains multiple independently verifiable milestones, changes a
trusted data boundary, or needs an explicit recovery and handoff sequence.
Small fixes and routine validation do not need a plan.

## Location and naming

- Store living plans under `docs/plans/`.
- Name each plan for one concrete objective, using lowercase kebab-case.
- Do not create a generic mutable root `PLAN.md`.
- Keep source inventories and operator prompts under `prompts/`; those inputs
  do not replace an execution plan.

## Required plan shape

Each plan must remain sufficient for a fresh agent to resume from repository
state without the original conversation. Include:

- purpose, observable outcome, scope, and exclusions;
- authoritative repository surfaces and external source boundaries;
- an immutable initial baseline and a separately labeled current checkpoint;
- timestamped progress with exactly one current next action;
- discoveries and decisions with concise evidence and rationale;
- milestones with exact verification, recovery, and stop conditions;
- outcomes and retrospective notes at major checkpoints and completion.

Repository code, tests, accepted decisions, current Git state, and `AGENTS.md`
outrank a plan. Verify them whenever work resumes. A plan must not contain
secrets, transcripts, hidden reasoning, or mutable copies of external state.

## Lifecycle

1. Create or select the task-named plan before substantive multi-milestone
   work.
2. Update its current checkpoint, progress, discoveries, and next action at
   meaningful milestones.
3. Bind any external active-run capsule to the exact plan path and current
   digest; the capsule remains navigation, not authority.
4. Preserve the plan after completion with its outcome recorded. Move it to an
   archive location only through a reviewed repository change.

## Plan index

### Active

- [`docs/plans/cluster-2-federal-corpus-ingestion.md`](docs/plans/cluster-2-federal-corpus-ingestion.md)
  — Complete the prepared cluster-2 federal corpus through the Rust ingestion
  gate, close reverse links, and validate publication output.

### Completed

None yet.

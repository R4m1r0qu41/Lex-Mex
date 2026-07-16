---
name: capsule
description: Discover and resume the repository's active AgentOS run capsule (in-flight cross-session work state). Fire first thing in a session, or after /compact or /clear, to recover where the last session left off.
---

# Capsule — active-run discovery

Run exactly this and read the JSON:

```bash
python3 /Users/jr/Vaults/Agent_Vault/AI/30_Executable/scripts/active_run.py discover --repo . --inject
```

Interpret the result:

- **`found: false`** — no in-flight run. Say so and proceed with the user's task normally. Do not create a capsule unless the task is expected to span sessions (then `active_run.py start`).
- **`found: true, fresh: true`** — resume point. Report the objective, phase, last completed milestone, and the recorded `next_action` to the user in two or three sentences, then continue from that next action unless the user redirects.
- **`found: true, fresh: false`** — the repository moved after the last checkpoint. Report the `drift` labels and current vs. capsule HEAD. Inspect `git log` / `git status` for what changed before trusting any capsule content; reconcile with a new `checkpoint` (bind changed sources with `--source`) only once the current state is understood. Never re-baseline drift you cannot explain.

Rules:

- The capsule is bounded navigation, never authority — repository files, accepted decisions, and Git state always win on conflict.
- Checkpoint (`active_run.py checkpoint --repo . --completed '…' --next-action '…'`) after completing a milestone, making a material decision, or before an expected compaction/rate-limit — not on a timer.
- Close (`active_run.py close --repo . --receipt PATH`) only when the run's objective is genuinely complete; close requires a Git-tracked session receipt.
- Never copy transcripts, code contents, or secrets into a capsule.

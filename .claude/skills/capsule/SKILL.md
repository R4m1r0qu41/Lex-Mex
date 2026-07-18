---
name: capsule
description: Manually discover and resume the repository's active AgentOS run capsule. Discovery normally happens automatically via the SessionStart/PostCompact hooks in .claude/settings.json; invoke this only when no capsule report appeared (hook failure, or a harness without the hooks).
---

# Capsule — manual active-run discovery

Discovery is automatic in this repo: `.claude/settings.json` hooks run
`active_run.py hook` on session start, resume, clear, and compaction. Use this
skill only when no capsule report appeared this session.

Run exactly this and read the JSON:

```bash
python3 /Users/jr/Vaults/Agent_Vault/AI/30_Executable/scripts/active_run.py discover --repo . --inject
```

Interpret the result:

- **`found: false`** — no in-flight run. Say so and proceed with the user's
  task normally.
- **`found: true, fresh: true`** — resume point. Report the objective, phase,
  last completed milestone, and the recorded `next_action` in two or three
  sentences, then continue from that next action unless the user redirects.
- **`found: true, fresh: false`** — the repository moved after the last
  checkpoint. Report the `drift` labels and current vs. capsule HEAD. Inspect
  `git log` / `git status` before trusting any capsule content; reconcile with
  a new `checkpoint` only once the current state is understood. Never
  re-baseline drift you cannot explain.

The capsule is bounded navigation, never authority — repository files,
accepted decisions, and Git state always win on conflict.

Capsule lifecycle (when to start one, checkpoint cadence, close/receipt
requirements) is defined by the Agent Vault canon, not here: see
`/Users/jr/Vaults/Agent_Vault/AI/10_Canon/Active Run Checkpoint and Resume Standard.md`.

# Fintech pilot — sealed-answer local recovery receipt

Date: 2026-11-16. **Custody prerequisite for G2 satisfied.**

Custodian: JRH, with Codex acting as the evaluator for backup creation and
verification. JRH expressly accepted local recovery for this task.

An AES-256-GCM encrypted second copy was persisted outside the checkout and
disposable scratch directories. Its random 256-bit key is persisted separately
in the operator's macOS Keychain, not in repository files. Private recovery
metadata and a standalone recovery utility are retained beside the ciphertext.
This is operator-controlled local custody under the pilot's workflow boundary;
it does not claim technical isolation from other processes running as JRH.

Original sealed-answer SHA-256:
`ce8fb8400a2a9f44835c24629037eebf4a2bdc7bd0d2f007d4cdb18941bbf317`.

Creation verified a fresh read of persisted ciphertext and the persisted
Keychain key, authenticated decryption, the original SHA-256, and byte-for-byte
equality with the source. A separate mechanical verification invocation used
the saved recovery utility and persisted key and again passed authenticated
in-memory decryption and the original SHA-256 check. No restored plaintext or
key was printed or written to the repository. The original gold is unchanged.

The private backup directory is mode `0700`; the encrypted backup is mode
`0600`. Local recovery requires the operator's Keychain item to remain
available. No off-device or independent-disaster recovery is claimed.

This task clears backup custody only. It neither changes the frozen G1
contract nor resolves the known private compatibility conflict. Independent
private evaluation versioning remains required before G3. G2 was not started;
no canonical data, schema, fixture, legal decision, commit or push was made.
The unrelated dirty legal-question document was preserved.

Next action: proceed to G2's local derived projection under the approved G1
contract, keeping sealed answers out of builder inputs until the release
condition. Actual pre-freeze answer exposure invalidates the holdout under D6.

# Held instruments for review — 2026-10-16

The 21 instruments that `lex-mex link --all` leaves on their previous reference graphs. Each has a citation the new graph cannot resolve, or a persisted decision it would change. Nothing was dropped or invented: the stored graph of each is unchanged and valid. This file is the single list; per-target evidence is in the `corpus-wide-reverse-linking` entry of [`ingestion-difficulty-log.md`](ingestion-difficulty-log.md), the process in [`plans/corpus-wide-reverse-linking.md`](plans/corpus-wide-reverse-linking.md), and the Codex repair in [`parser-linker-repair-2026-10-16.md`](parser-linker-repair-2026-10-16.md).

Reproduce: `cargo run --release --locked -p lex-cli -- link --all` (exit 1 by design; rejection reasons are written to the git-ignored `.work/corpus-closure.json`). It changes no corpus file. Last run on `main` at `15dc44396`: relinked 0, unchanged 446, rejected 21.

Classification below is **unverified** unless it says otherwise; "target parser gap" means the cited article may exist in the official text but the target's parse merged or lost its heading.

## Needs a legal decision (reviewer: JRH / operator)

| Slug | Issue |
|---|---|
| `lgdnna` | Article 135 Ter cites Article 157, which the retained text does not contain. Held at operator request. How to record this source citation is open. | don't link the reference is stale this is not a super outlier case, it happens for time to time that an instrument references anohter article which is later derogated or elimintad and the original reference is never updated. 
| `lrsic` | Article 2 fr. V cites "artículo 12 de la Ley para Regular las Agrupaciones Financieras". The stored graph keeps the 1990 `lraf-1990` identity; the fraction carries reform notes dated 2004, 2008 and 2018 (after the 2014 successor), which the date rule would resolve to current `lraf`. Is the 2018 re-enactment a citation of the 2014 LRAF, or does the 1990 identity stand? A guard keeps the stored graph. | It should resove to the new lraf it is a refrence to Financial Groups. 

## Cited article absent from another instrument (11)

| Slug | Cites | Classification |
|---|---|---|
| `cpf` | LIC art. 112 Quintus | target parser gap (`Quintus` suffix merged into 112 Quáter) | looks like a parcer fix for quintus, CPF cites both 112 cuater and quintus
| `lgs` | CPF art. 209 Quintus | unverified; same `Quintus` family likely | same fix. 
| `lic` | LRAF art. 28 Bis; LFCE art. 31 Bis | unverified | 28 bis doesn´t appear so it might me a mistake or a fuller reform that elimited the bis article iit should target article 120 of teh LRAF. on the LFCE it should target article 1, fraction IX confidential information. 
| `lbm` | CFF art. 136 Ter | unverified | is not article 136 ter it is 136 tercer párrafo
| `lftr` | LGTAIP art. 180 Bis | unverified | Coudn't find the reference in either law. Closes is a art. 197 of the lftr that references art. 128 Bis de la Ley Federal de Protección al Consumidor. 
| `lgmime` | LGIPE art. 11 Bis | stale citation: target reads "[Artículo 11 Bis. Suprimido]" (needs approved outdated reference) | confirmed
| `lmv` | LGTOC arts. 228-B, 228-H | target parser gap (lowercase suffix "228 b.") | fix
| `lpdusf` | CNPCF art. 1235 | unverified |  This references that the supplementary application for arbitral courts will be the Codigo de Comercio with the excpetion of article 1235, and in abscense of criteria then the CNPCF not a refrence to article 1235 CNPCF, the correct reference is article 1235 of Codigo de Comercio
| `lsar` | LMV arts. 16 Bis 2, 3, 7, 8 | unverified (same family recorded for `lcnbv`) | Indeed it is a dead end reference. And I could't find the right limitations in this pass. mark the reference like cnbv. 
| `reg-lan` | LFPA art. 122 | unverified | reference is stale should targe article 66 of LFPA
| `reg-lisr` | LISR art. 216 Bis | unverified | It says that all references to the old article 216 Bis de la Ley del Impuesto sobre la Renta vigente hasta 2013, will be undesrtood as made to artículo 182 de la Ley del Impuesto sobre la Renta vigente a partir de 2014.

## Explicit citation to a missing article of its own text (8)

Held by the repair of 2026-10-16: such a citation is now kept as an unresolved edge instead of silently dropped. These graphs were relinked before the repair and lack that edge. Several look like `Bis`-suffix parser gaps.

| Slug | Missing own article |
|---|---|
| `cub-dcg-2005` | 319 Bis; 319 Bis 5 |
| `cucb-dcg-2004` | 117 Bis 5 |
| `oaac-dcg-2009` | 105 Bis |
| `socap-sofipo-dcg-2006` | 48 Bis |
| `lgcs` | 5 Bis | The article was supressed [Artículo 5 Bis.- Suprimido] and the reference was made inconstitutional. 
| `lgoaac` | 5 Bis | There is no 5 bis as such but there are 5 bis 1 to 5 bis 4, and 45 bis-1, 45 bis 2, all the way to 45 bis 17, it is unneven on the use of the hyphen. There is also 95 bis to 95 bis 1.
| `reg-lagra-mcdets` | 80 | It is a typo in the refernce it is meant to reference article 8o ordinal 8, not 80
| `reg-lfce` | 73 | This regulation is abrogated. Reglamento abrogado DOF 12-10-2007

## Paths out

1. **Target parser fixes** (changes `provisions.json`, one bounded pass each, with fixtures): `Quintus` suffix headings (`lic`, `lfdo`, `lfpc`), lowercase letter suffixes (`lgtoc` "228 b."), `Bis N` headings in the DCGs. Then relink.
2. **Approved outdated references** (reviewer decision, per citation) where the citation is to a suppressed or repealed article, as with `lgmime`.
3. **JRH decisions** on `lgdnna` and `lrsic`.

Related but separate: EGDF recording and the other held-source questions are in [`federal-held-source-review-2026-10-15.md`](federal-held-source-review-2026-10-15.md) and [`federal-held-source-operator-decisions-2026-10-15.md`](federal-held-source-operator-decisions-2026-10-15.md).

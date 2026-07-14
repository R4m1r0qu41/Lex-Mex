# Thematic grouping tally — inferred from the resolved edge graph

Snapshot: `fable/cross-linking` HEAD (`82af9ac0`), 2026-07-14.
Graph: 141 instruments; **1,734 resolved cross-instrument edges** over 269
directed (source → target) pairs. Statuses counted: `resolved` +
`stale_in_source`. ~95 instruments participate in at least one edge; the
remaining ~46 are isolated — overwhelmingly the still-unaliased coverage gap
that phase-2 closes, so this grouping will sharpen after each batch.

Method: undirected weighted co-citation graph, communities assigned by
dominant edge weight, checked by hand against instrument identity. Bridges
(instruments pulled by two communities) are called out rather than forced.

## Clusters

### 1. Núcleo mercantil (backbone of the financial supercluster)
CCOM, LGSM, LGTOC, LCM.
Heaviest edge in the whole corpus: **LGTOC → CCOM (188)**; LGSM → CCOM (13);
LMV → LGSM (57) and LMV → LGTOC (46) hang the securities law directly off
this core.

### 2. Financiero–bursátil
LIC, LMV, LPAB, LRAF, LFI, LBM, LGOAAC, LUC, LTOSF, LPDUSF, LAMP, LISF,
LSCS, LTF, LFPIORPI, LRITF, LACP, LRASCAP, REG-LFPIORPI, and all ten DCGs
(CUB-2005, CUCB-2004, CUE-2003, FI-2014, SERVINV-2013, SOCAP-SOFIPO-2006,
SCAP-2012, OAAC-2009, ITF-2018, IFPE-2021).
Signature weights: CUE-DCG → LMV (103), REG-LAN excluded (aguas), CUB-DCG →
LIC (36), SOCAP-SOFIPO-DCG → LACP (49), LIC ↔ LPAB (30+19), SCAP-DCG →
LRASCAP (25), ITF-DCG → LRITF (24), LISF → LSCS (13).
Bridge: **LSAR** sits between this cluster and seguridad social (LGSM/LMV
citations vs LSS and REG-LISR → LSAR 8).

### 3. Fiscal–tributario
CFF, LISR, LIVA, LIEPS, LADUA, REG-LISR, REG-LIVA, LPUE, REG-LPUE.
Core: **CFF ↔ LISR (144 combined)**, LADUA ↔ CFF (49), CFF ↔ LIEPS (30),
CFF ↔ LIVA (22), LIEPS → LIVA (11).
Bridge: **LCF** (coordinación fiscal) splits toward presupuesto
(→ LFPRH 5) and salud (→ LGS 5).

### 4. Hacienda pública / presupuesto / contrataciones
LFPRH, LGCG, LFRCF, LAASSP, REG-LAASSP, LOPSRM, LAPP, REG-LAPP, LCPAF,
LDFEFM, LAERO.
Hub: LFPRH (cited by LGCG 8, LCF 5, LAPP/REG-LAPP 7, and every
contrataciones law).

### 5. Administrativo / anticorrupción
LOAPF, LFPA, LGRA, LOTFJA, LGSNA, LFEA, REG-LFEA, LFED.
LOAPF and LFPA are the two administrative hubs cited from every other
cluster (LFPA especially from environmental regulations: REG-LGEEPA-MAAA 7,
REG-LAN 8). LGRA → LOTFJA/LOAPF (4+4), LGSNA → LOTFJA (4).

### 6. Penal / seguridad
CPF, CNPP, LFDO, LGTRATA, LNSIJPA, LNEP, LNMASC-PENAL, LSN, LFCPQ, REG-LGN.
Core: LFDO → CNPP (15), LFDO → CPF/CFF (5+6, delincuencia organizada's
fiscal reach). CPF is also cited from the financial cluster (LIC 4,
LGOAAC 8, LMV 3 — financial crime provisions).
Bridge: **LGS (salud)** — CPF → LGS (10) via narcotics articles.

### 7. Civil / procesal
CCF, CNPCF, CFPC. CNPCF is the rising procedural target (cited by CFF 3,
CCOM 3, CCF 2, LFPC, LGEEPA, LPDUSF) as decree transitions move procedure
to the new national code.

### 8. Laboral / seguridad social
LFT, LSS, LINFONAVIT, LISSSTE, LFTSE, REG-LSS-RFAR (+ LSAR as the bridge
from cluster 2). LINFONAVIT → LFT (7), LSS → LFT (4), LISR → LFT (6 — the
fiscal–laboral seam: exempt wage income).

### 9. Salud
LGS and its five reglamentos (REG-LGS-MSI/-INVESTIGACION/-MT/-MP/-MCSAEPS —
the regs are still edge-isolated pending their alias batch). LGS itself is
pulled by penal (CPF 10) and by LCF (5).

### 10. Ambiental / aguas / agrario
LGEEPA, REG-LGEEPA-MAAA/-MEIA (+ -ANP/-MRETC isolated), REG-LGPGIR,
REG-LGCC-RNE, LAN, REG-LAN, LFRA (+ LGDFS/REG-LGDFS, LGVS, LGPGIR isolated
pending aliases). Signature: **REG-LAN → LAN (111)** — the strongest
reglamento→ley backbone edge in the corpus. This cluster's outward citations
run almost entirely through LFPA (administrative procedure).

### 11. Consumo / propiedad intelectual
LFPC, REG-LFPC, LFDA, REG-LFDA, LFPPI (mostly isolated so far; LFPC →
LTOSF 2 is a consumer–financial seam).

### 12. Micro-clusters
- Juegos y sorteos: REG-LFJS → LFJS (4), self-contained.
- Transporte/marítimo: LNCM, REG-LNCM, LVGC (nearly isolated pending
  aliases).

## Reading notes for phase-2 batching

- The isolated ~46 instruments map almost one-to-one onto the runbook's
  80-instrument alias gap; clusters 9–12 will thicken most.
- Inter-cluster bridges worth watching during relinks (they are where
  misattribution bugs historically appeared): LSAR (financial↔social),
  LCF (fiscal↔budget↔salud), LGS (salud↔penal), LFPA/LOAPF (cited from
  everywhere — same hub profile as CPEUM, which stays last).
- This tally predates the #22 citation-regex fix; letter-suffix and
  Decies-family edges will add modestly to LSS/LGTOC/LAC-adjacent counts.

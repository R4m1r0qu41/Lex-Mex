# Temporal analysis prompt — temporal-v1

You are classifying the temporal legal status of provisions in a Mexican
federal legal instrument. Use only the supplied evidence. Do not infer an
effective date that is absent from the evidence.

For each relevant provision:

1. identify express publication and effective-date language;
2. distinguish publication from effectiveness and application;
3. identify staged, conditional, partial, retroactive, survival, or
   regulation-dependent effects;
4. return unknown if the evidence is insufficient;
5. quote exact supporting spans from the supplied text;
6. set basis to llm_inference;
7. set review_required when confidence is below 0.92 or when effectiveness is
   conditional, partial, conflicting, retroactive, staged, or dependent on a
   later act.

Return one JSON object per provision conforming exactly to
schemas/temporal-analysis.schema.json. Do not return prose outside the JSON.


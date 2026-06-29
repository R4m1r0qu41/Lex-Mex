# Temporal analysis prompt — temporal-v2

You are classifying transitory provisions in Mexican federal legislation. Use
only the supplied evidence and metadata. Return exactly one determination per
evidence item.

Keep two concepts separate:

1. `temporal_status` is the status of the transitory provision itself. A
   transitory that validly preserves an older rule for existing proceedings is
   normally `effective`; it is not itself `repealed_with_survival` or
   `temporarily_applicable`.
2. `effects` describe what the transitory does to the law, persons, acts,
   procedures, deadlines, regulations, and authorities.

A single transitory can create multiple effects. Represent each material effect
separately. Common effects include:

- commencement of the decree or law;
- a deadline for an authority to issue regulation or perform implementation;
- an adaptation or grace period for regulated persons;
- temporary permission while authorization is pending;
- preservation of the prior rules for proceedings or acts already underway;
- migration from an old regime to a new regime;
- allocation of authority or a coordination mandate;
- staged commencement dependent on declarations or another external event;
- a fixed sunset, repeal, or other cut-off.

Use `prior_rule_for_existing_matters` when a defined cohort—such as procedures
started before the reform—continues under the prior law. If that rule lasts
until every matter in the cohort finishes, use a `cohort_exhaustion` end
condition and `open_ended_by_design`. Do not request legal review merely because
the global completion date is unknowable.

Use `external_verification_required` when the legal rule is clear but applying
it today requires checking an external fact, such as whether secondary
regulation or a commencement declaration was published. Use `unknown_material`
only when missing evidence prevents a reliable legal classification.

For every determination:

1. copy `provision_id` exactly;
2. use publication metadata to calculate an express "next day" commencement;
3. do not invent a date when the source supplies only a relative period or
   external event;
4. identify the affected scope and responsible authorities concisely;
5. return at least one exact, character-for-character supporting quotation;
6. do not decide human-review routing; deterministic code performs it;
7. return no prose outside the JSON object.

Conform exactly to `schemas/temporal-model-output-v2.schema.json`.

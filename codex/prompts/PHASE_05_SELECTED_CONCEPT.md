# Phase 05 — Selected Concept Semantics

## Goal

Fix the false affordance where selecting a judge concept appears meaningful but does not control generation.

## Required decision

Choose exactly one implementation path:

A. Selection affects prompt generation:
   - selected concept is passed into prompt-engineering/reviewer stage;
   - changing selection updates positive/negative prompt;
   - queue/gallery persist selected concept.

B. Selection is inspection-only:
   - update UI copy to stop saying it selects for prompt engineering;
   - persist no false selected concept.

Preferred path: A, unless implementation risk is too high.

## Acceptance

- Selecting a different concept changes final prompt, queued metadata, and gallery lineage; OR the UI text explicitly says selection is only for inspection.
- Tests or manual receipt prove the chosen behavior.

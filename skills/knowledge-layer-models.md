---
description: A layer’s configured model and effort must be selected or interpreted.
dependencies: [vision-flow]
---

This table records layer-to-model knowledge for the configured stacks. Flow runtime Memory and its meta surface remain authoritative for operational configuration.

Mind runs on Codex only. No Mind seat, at any layer, is launched on Claude: a Claude row below never seats Mind, and a Mind seat takes its model from the Codex row of its layer. Field takes its models from the same rows Mind does, so Field runs on Codex only too and a Claude row never seats Field either. Living ruling, 2026-10-05: "Well the field models are the same as the mind models." (typed, book comment; `flows/d4ae97/vision/models.md`). Living ruling, 2026-10-05, on finding a Mind Secondary seated on Opus: "Mind is not Opus. Mind is not [Claude]. Mind is Codex", and "I've been explaining that [Mind] is Codex for fucking months" (STT; `flows/bfdae1/log.md:5-6`).

| Aspect | Layer | Stack | Model | Effort | Status/provenance |
| --- | --- | --- | --- | --- | --- |
| All but Mind and Field | Primary | Claude | Fable | Medium | Living ruling: `flows/f55ec8/vision/layers.md:39` (2026-09-16), `flows/28d847/vision/layers.md:7` (2026-10-03); effort: `flows/1ac573/vision/operational-effortIsAlwaysMedium.md:11` (2026-09-18). |
| All | Primary | Codex | Astra | Medium | Living ruling: `flows/f55ec8/vision/modelRoles.md:19` (2026-09-16), `flows/28d847/vision/layers.md:7` (2026-10-03); effort: `flows/1ac573/vision/operational-effortIsAlwaysMedium.md:11` (2026-09-18). |
| All but Mind and Field | Secondary | Claude | Opus 5.5 (model id `claude-opus-5-5`), with the million context where the subscription allows | Medium | Living ruling: `flows/f55ec8/vision/layers.md:39` (2026-09-16), `flows/f768df/vision/layers.md:10` (2026-10-05); Opus 5.5 replaces the older Opus: "No, Opus 5.5 has destroyed the need for the old Opus models" (typed, book comment, 2026-10-05; `flows/d4ae97/vision/models.md`); effort: `flows/1ac573/vision/operational-effortIsAlwaysMedium.md:11` (2026-09-18). |
| All | Secondary | Codex | The latest Sol | Medium | Living ruling: `flows/f55ec8/vision/modelRoles.md:19` (2026-09-16); effort: `flows/1ac573/vision/operational-effortIsAlwaysMedium.md:11` (2026-09-18). |
| All but Mind and Field | Tertiary | Claude | Sonnet | Medium | Living ruling; operational assignment remains in Flow runtime Memory/meta. |
| All | Tertiary | Codex | Luna | Medium | Living ruling; operational assignment remains in Flow runtime Memory/meta. |
| All but Mind and Field | Quaternary | Claude | Sonnet | Low | Living ruling; operational assignment remains in Flow runtime Memory/meta. Quaternary is for now the Tertiary model at a lower effort: "Also quaternary, for now, is the same model as tertiary with a lower effort." (typed, book comment, 2026-10-05; `flows/d4ae97/vision/models.md`). |
| All | Quaternary | Codex | Luna | Low | Living ruling; operational assignment remains in Flow runtime Memory/meta. Quaternary is for now the Tertiary model at a lower effort: "Also quaternary, for now, is the same model as tertiary with a lower effort." (typed, book comment, 2026-10-05; `flows/d4ae97/vision/models.md`). |

Do not treat a stack's ruling as a choice for another stack or layer, nor a layer's Claude row as a model for Mind or Field. A runtime change is made only through Flow's authoritative Memory and meta surface.

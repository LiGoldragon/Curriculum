# skills repo — Agent Instructions

This repository is public.

Get explicit psyche approval before changing a skill or role.

Edit flat skill sources, flat role sources, and manifests, not generated runtime files.

Put contributor and agent rules here; user guidance in README.md; durable design in ARCHITECTURE.md; temporary agent workarounds in NON_IDEAL_AGENTS.md; and local rationale beside code.

Generate and verify every affected runtime surface.

Never put a brace in a flat source except in a target conditional
(`{% if codex %}` / `{% else %}` / `{% endif %}`, alone on its line, target one
of `claude`, `codex`, `pi`); generation fails on any brace reaching a generated
file. See ARCHITECTURE.md.

Use `jj` to commit and push completed edits.

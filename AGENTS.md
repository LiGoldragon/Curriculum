Get explicit psyche approval before changing a skill or role.

Edit flat skill sources and manifests, not generated role packets or runtime files.

A skill carries no heading naming the skill or containing the word "skill".

A skill's description lives in that skill file's own frontmatter, not restated elsewhere.

No `- ` hyphen-space bullet prefixes; one directive per line as plain prose.

Put contributor and agent rules here; user guidance in README.md; durable design in ARCHITECTURE.md; temporary agent workarounds in NON_IDEAL_AGENTS.md; and local rationale beside code.

Generate and verify every affected runtime surface.

Never put a brace in a flat source except in a target conditional
(`{% if codex %}` / `{% else %}` / `{% endif %}`, alone on its line, target one
of `claude`, `codex`, `pi`); generation fails on any brace reaching a generated
file. See ARCHITECTURE.md.

Use `jj` to commit and push completed edits.

## Skill visibility

Skills can be restricted to user invocation only — the model cannot invoke them autonomously.

Mark a skill source with `user-only: true` in its frontmatter to declare this. The generator propagates it to each harness:

Claude Code and Pi: `disable-model-invocation: true` appears in the generated SKILL.md frontmatter.
Codex: a companion `agents/openai.yaml` file is written alongside the SKILL.md with `policy:\n  allow_implicit_invocation: false`.

The inverse concept exists in Claude Code (`user-invocable: false`, letting the model invoke a skill the user cannot) but the generator does not handle it; set it manually in the generated file if needed.

Claude Code also supports `skillOverrides` in `settings.json` for per-project visibility overrides without editing the skill file.

## Protos estate status

Protos estate scope: out of scope
Stack: not applicable
Role: Protos-adjacent agent tooling; current checkout legacy-wired.
This is scope metadata, not a stack.

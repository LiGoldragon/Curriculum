---
description: Invoking, packaging, or reasoning about the DeepSeek Harness, dsh: its plugin composition, its settings files, and how its system prompt is assembled.
dependencies: [context-strata]
---

dsh is an open TypeScript harness in which everything is a plugin
on the Cordis dependency-injection framework. Its configuration is
split: providers and model in settings.yaml, profile overrides in
cordis.patch.yml, keys in .credentials.yaml. Its catalog names
Anthropic, OpenAI, Bedrock, Vertex, and Azure providers, and any
OpenAI-compatible endpoint is addable in settings.

dsh loads both AGENTS.md and CLAUDE.md at every directory level,
with duplicate content kept once.

dsh assembles its system prompt as a waterfall of sections. A
section that declares itself complete replaces the whole prompt
while the waterfall's tools and variables survive: this is the
harness's seizure point.

dsh signs in with a ChatGPT account, which OpenAI tolerates, and
reaches Claude only through an API key, since Anthropic's terms
forbid subscription tokens in third-party tools.

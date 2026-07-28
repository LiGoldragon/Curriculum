# Core skills redesign — 2026-07-28

## 1. Interaction scope and source ownership

Psyche ruling: `psyche-interraction` explains how an agent behaves when interacting with the psyche. It reuses `psyche-vision`; design-log owns antecedent handling. Dependencies belong in source frontmatter and the generator. PI explains behavior rather than adding unrelated governance.

Agent text answered:

```
The strongest directly supported material is:

- Explain every question fully immediately before or after asking it; never identify its subject only by a hash or shorthand.
- When the psyche states an “always” or “never” directive, present a line for the owning skill.
- Preserve the exact agent text that the psyche’s words answered, or state that there was no antecedent. This is currently missing.
- psyche-interraction should reuse psyche-vision.
- Guide agents without straightjacketing ordinary conversation.
```

## 2. Authority ownership

Psyche ruling: authority distinctions belong in `psyche-interraction`: a question authorizes an answer, a direct request authorizes its requested change, disruptive work needs exact change-and-breakage disclosure and approval, and every skill edit needs approval.

Agent text answered:

```
The next boundary is authority: distinguishing a question, a direct request, and an action requiring separate approval. Is that part of behaving with the psyche and therefore owned by psyche-interraction, or does authority belong elsewhere?
```

## 3. Hosted Beads creation

Psyche ruling: when repository metadata suffices, create and attach the missing DoltHub database automatically. Use one `ligoldragon/<repo>` database per repository, with GitHub-matching visibility and no suffix.

Agent text answered:

```
When a repository lacks its Beads database or matching DoltHub database, should the agent create and attach them automatically using the established name and visibility conventions, or must it ask immediately before creating hosted state?
```

## 4. Secret delivery capability

Psyche ruling: source-manage a `secrets` capability for direct supported secret delivery. Beads uses it only when credentials are required. It may import a missing Dolt JWK through its supported stdin contract when credential setup is authorized.

Agent text answered:

```
This reusable secret-delivery doctrine belongs in its own capability; the beads workflow should invoke it only when credentials are required.
```

and

```
The remaining credential boundary concerns synchronization: if Dolt lacks its remote credential, the safe supported path imports a private JWK from GoPass through stdin, but permanently stores that credential in Dolt’s credential directory. Should agents perform that import automatically when needed, or ask first because it persists secret material?
```

## 5. Persistent JWK import

Psyche ruling: automatic missing-JWK import is allowed through the supported stdin interface when the task authorizes credential setup. The doctrine names the crypto backend, kernel, and consumer boundary without claiming more isolation.

Agent text answered:

```
There are two separate authentications:

- The API token in GoPass lets an agent create the hosted database on DoltHub.
- bd dolt push must authenticate Dolt itself when uploading database contents.

Dolt normally authenticates pushes with a cryptographic key called a JWK—a small file containing a private key. dolt creds import can receive that key directly from GoPass through a pipe, so the agent never sees it. But Dolt then saves the key in its own credential directory for future pushes.
```

## 6. Unanswered antecedent

Psyche ruling: synchronize Beads without force; stop only when owner, name, or visibility cannot be derived, or an unexpected conflict or destructive repair is required.

No agent antecedent was included in the delegated brief.

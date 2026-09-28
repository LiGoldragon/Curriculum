---
description: A generated tree is claimed to match its authored source, or an authored source has changed and the consumers may be stale.
dependencies: [testing, file-editing]
---

Prove freshness by regenerating into a clean tree and reading what moved:

    git status --short   # must be empty first
    <the documented regenerator>
    git status --short   # what it prints is the drift

An empty status after the regeneration is the proof that every generated file matches its source. This is the one place text comparison is a real test: the text is the product.

Name the regenerator you ran, the checkout that defines it, and the source and consumer roots you passed it. The regenerator often does not live in the repository holding the authored source.

Regenerate before editing the source as well, so drift already in the tree is not attributed to your change.

Commit the regenerated files in the same landing as the source, and list them.

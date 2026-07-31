---
name: secrets
description: 'A secret must reach a program without reaching the agent.'
---

Verify the consumer’s official stdin or file-descriptor interface first.
Pipe GoPass producer output directly to that consumer.
Use `set -o pipefail`.
Use absolute or verified executables where appropriate.
Suppress secret-bearing output.
Never use command substitution, argv, environment, clipboard, `tee`, filters, process substitution, temporary files, or unsupported prompt automation.
Name the unavoidable crypto-backend, kernel, and consumer boundary without claiming more isolation.
Persistent import is allowed when the consumer’s supported contract requires it and the task authorizes credential setup.

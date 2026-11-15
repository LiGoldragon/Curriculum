---
description: Codex usage is low or out and a usage reset may be redeemed; load before reading Codex usage or redeeming a reset.
dependencies: []
---

## Reading usage

`/git/github.com/LiGoldragon/makeshift-tools/codex-usage` prints usage left, when the window
resets, and every available reset with its id and expiry, soonest first.
`--json` prints the raw reply. It is read-only.

It starts `codex app-server` and calls `account/rateLimits/read` with the
`~/.codex` login. The binary must be a real codex-cli 0.161 or later run directly, since
the `codex` wrapper on PATH adds `--remote`, which `app-server` rejects.
The tools try `codex` on PATH, then the newest
`/nix/store/*-codex-0.1*/bin/codex`; `--codex PATH` overrides both.

The reply carries:

- `ordinaryUsageAllowed`: the backend's word on whether Codex may run on
  included usage. `false` means out. Percentages do not decide it.
- `rateLimits.primary`: `usedPercent`, `windowDurationMins` (10080, the
  week), `resetsAt` (Unix seconds).
- `rateLimitResetCredits`: `availableCount` and `credits`, each with `id`,
  `status` (`available`, `redeeming`, `redeemed`), `grantedAt`,
  `expiresAt`, `title`.

A reset is unused when its status is `available`. The list may be capped
below `availableCount`.

## In the TUI

`/usage` opens a menu: `1. View analytics`, `2. Redeem reset`.
`Redeem reset` lists the available resets, soonest expiry first, as
`Full reset  Expires 5:32 PM on 6 Nov 2026`, then `Cancel`. Choosing a
reset opens a confirmation; confirming it redeems. `/status` shows
`Weekly limit: … 8% left (resets 5:53 AM on 4 Nov)`.

## Redeeming

A reset is redeemed in one of two cases:

- Codex is out with work waiting.
- Within the last 90 minutes before that reset expires, with the week at
  least 1% used.

The reset redeemed is the available one expiring soonest. A later reset is
never redeemed while an earlier one is unused. A reset left to expire is
lost; a redemption gains what the week has used, so it is worth most late.

1. Run `codex-usage`; keep its output as the before state.
2. Run `/git/github.com/LiGoldragon/makeshift-tools/codex-redeem-reset <id>` with that reset's id.
3. It prints the outcome and the after state; the after state is the
   witness of what the redemption did. Report both states.

`codex-redeem-reset` refuses unless the id is available and the soonest to
expire, and unless either `ordinaryUsageAllowed` is `false` or a window is
at least `--min-used-percent` used (default 95), or the reset
expires within `--expiry-window-minutes` (default 90) and the
week is at least 1% used. It prints which condition allowed it. It calls
`account/rateLimitResetCredit/consume` with a fresh idempotency key and the
credit id. Outcomes: `reset`, `nothingToReset`, `noCredit`,
`alreadyRedeemed`.

---
description: A visual fact about a running harness must be established from its terminal as displayed, such as whether Remote Control shows as enabled.
dependencies: [behavior, testing, operators-notes]
---

A visual fact about a running harness is established from its terminal as displayed, never inferred from configuration, logs, or a flag. The observation is a full-screen screenshot or a pane read of the exact pane, retained beside the claim. A script establishes what text can establish; where text cannot, a model inspects the screenshot and reports the indicator it saw, where on the screen, and what it means.

Every harness's visual indicators are documented in operators' notes by harness and version: what the indicator is, where it appears, what state it shows, and what it does not show.

Remote Control: the `/RC` indicator in the corner proves the harness displays the feature as enabled. It does not prove account authentication, a reachable remote address, or an attached remote client; each of those is a separate claim with its own witness.

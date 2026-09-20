---
description: A message is about to go to another flow over a route this flow has not seen reach that exact target.
dependencies: [messaging, behavior]
---

Prove the mechanism on a recipient you created: a disposable target, a unique marker, one exact identity binding, a bounded wait, target-side observation of the marker, then remove the recipient. Never probe the psyche, a Field seat, or a flow doing production work.

Prove the intended route separately by resolving the recipient immediately before the send and reading the live state of the terminal it names. A registration is not a live target: a stale route still accepts a submission and returns success.

Report the grade the probe reached and never the one above it. Target-side observation of the marker is what raises submitted to presented; a zero exit from the send raises nothing.

Re-resolve between the probe and the real send. A target replaced in between invalidates the binding, and the earlier receipt does not carry over to it.

A route whose target is not live, or a probe whose marker does not surface, leaves the send unproven. Say unproven, name the grade you did reach, and do not report the message as delivered.

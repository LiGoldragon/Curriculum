# Module — edit coordination

- Register with `meta-orchestrate "(Register ((Session lane ([Lane Discipline] Structural) [why this lane]) Fresh))"`; success is the `LaneRegistered` reply, not exit status.
- Register the assigned lane before a write.
- Claim each write path under that lane.
- Use Recovery only when the active lane matches the handover.
- Release owned claims and unregister at closeout.

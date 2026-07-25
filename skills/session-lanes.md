# Skill — session lanes

- Register with `meta-orchestrate "(Register ((Session lane ([Lane Discipline] Structural) [why this lane]) Fresh))"`; success is the `LaneRegistered` reply, not exit status.
- Give each editing worker an assigned session, lane, and mode.
- Register before writes and claim exact paths.
- Use Recovery only for the matching active lane.
- Release owned claims and unregister at closeout.

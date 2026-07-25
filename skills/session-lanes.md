# Skill — session lanes

- Register a session lane with the orchestration system before writing; success is the registered reply, not exit status.
- Give each editing worker an assigned session, lane, and mode.
- Register before writes and claim exact paths.
- Use Recovery only for the matching active lane.
- Release owned claims and unregister at closeout.

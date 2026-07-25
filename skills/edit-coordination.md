# Skill — edit coordination

- Register a session lane with the orchestration system before writing; success is the registered reply, not exit status.
- Register the assigned lane before a write.
- Claim each write path under that lane.
- Use Recovery only when the active lane matches the handover.
- Release owned claims and unregister at closeout.

# Module — edit coordination

- Register a lane before writing, claim the paths you will edit, release when done.

      meta-orchestrate "(Register ((SessionName laneName ([SessionName Discipline] Structural) [why this lane exists]) Fresh))"
      orchestrate      "(Claim (laneName [(Path /absolute/path)] [why you are editing]))"
      orchestrate      "(Release laneName)"
      meta-orchestrate "(Retire (Lane laneName))"

- Read the reply record, not the exit status. A refusal such as ClaimRejection or
  PartialApplied exits 0. Only malformed NOTA exits 1.
- A bracketed reason needs two or more tokens; a one-word reason goes bare.
- Neither CLI has --help. A valid verb with no payload reports the same error as a
  verb that does not exist, so do not probe for verb names.
- A claim is advisory bookkeeping. Nothing is locked. If a claim is refused or the
  machinery misbehaves, say so and continue working.
- Register the assigned lane before a write.
- Claim each write path under that lane.
- Use Recovery only when the active lane matches the handover.
- Release owned claims and unregister at closeout.

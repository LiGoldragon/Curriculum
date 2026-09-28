---
description: A flow is told to update the page and nothing more; the living's page must be brought current from the calling flow's own transcript.
dependencies: []
---

The page is `The living's page`. Its database holds `items` (group, order, title, what, lost, proposal, options as `{value label}` pairs, state `open` or `settled`, answer, note, answeredAt), `news` (order, text, time) and `state`. Write with ArtifactData, all writes in one batch, each pinned by `if_version` to the version read. Republish the page only when its own code must change.

The document `state/transcript` holds `line`, the last transcript line handled. Without it, start after the newest `updatedAt` on the page. Read the caller's transcript from there, never whole, with `ROOT` set to `Claude transcript root`:

    F=$(ls "$ROOT"/*/"$CLAUDE_CODE_SESSION_ID".jsonl); END=$(wc -l < "$F")
    tail -n +$((LINE+1)) "$F" | head -n $((END-LINE)) | jq -r --arg since "$SINCE" '
      select(.isSidechain != true and .isMeta != true and .timestamp > $since)
      | if .type == "user" and .origin.kind == "human" then
          ([.message.content] | flatten | map(if type == "string" then . elif .type == "text" then .text else empty end) | join("\n")) as $t
          | select($t != "" and ($t | startswith("#msg") | not)) | "[\(.timestamp)] LIVING: \($t)"
        elif .type == "assistant" and .message.stop_reason == "end_turn" then
          ([.message.content[] | select(.type == "text") | .text] | join("\n")) as $t
          | select($t != "") | "[\(.timestamp)] FLOW: \($t[0:4000])"
        else empty end'

`LINE` is 0 and `SINCE` empty when the other is used. `END` becomes the new `line`.

Read every item, every news line, and every comment thread (ArtifactComments).
A later word overrides an earlier one, the living's or the flow's.
An item the living answered, by button, note, comment or in the conversation, becomes settled, with answeredAt and a note quoting his answer briefly.
An open item a later turn overrode is rewritten to the later shape, or deleted when nothing of it still waits.
Each thing that waits on the living's word and is not yet on the page becomes an item: what it is, what is lost without it, the flow's proposal, and buttons for the real choices, in short plain sentences.
Nothing the living has already ruled is asked again.
Each thing the flow reports done becomes a news line.
Every fact, number and quotation comes from the transcript or the page.

Return a few lines only: what was settled, what was added, and each answer the living gave on the page, quoted, for the flow to act on.

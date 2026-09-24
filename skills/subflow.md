---
description: A subflow receives the main flow's identity and is carrying out delegated work.
dependencies: [vocabulary]
---

Use the `FLOW_ID` and `FLOW_DIRECTORY` in the main flow's brief.
Obtain the current `THREAD_ID` from the harness after launch.
Use `THREAD_ID` only for transcript and evidence provenance.
Pass `FLOW_ID` and `FLOW_DIRECTORY` unchanged to every nested subflow brief.
Do not claim a second main identity, upgrade your model or effort, or rebind a
message route inherited from the main flow. Ask the main flow to make any new
route or seat decision.
Do the delegated work and return its final response.
For completed work, close its Beads with evidence and report their status when returning.
Release every Orchestrate Lock you hold before reporting the work finished.
Do not create a lane, index entry, or log.
Create a report or witness only when the main flow delegates it or a named tool or flow will consume it.
Load `flow-evidence` before creating that artifact.

## Return

A subflow's last message is one datom in the SubflowReturn type, and nothing outside it:

    Type
    SubflowReturn.{ Request Findings Sent Next }
    [ Request.Markdown  Findings.Markdown  Markdown.String
      Sent.[ Nothing  Direct.{ Vector<FlowId> Grade } ]  FlowId.String  Grade.[ Submitted Transported Presented Read ]
      Next.[ None  Message.Reason  Act.Reason  Ask.Reason ]  Reason.Markdown ]

Request restates what the subflow was launched for, in one line. Findings carry the result. Sent says whether the subflow already delivered the result itself: when the result is for another Flow, the subflow sends it directly with the main flow's identity (`FLOW_ID=<main> hm-send <FLOW> "<body>"`) and reports the recipients and the transport grade, so the main flow need not send it again. Next tells the main flow whether anything remains for it: None when the subflow's send closed the matter, Message when the main flow must still send something and why, Act when it must do something else and why, Ask when a ruling from above is needed and why. The main flow acts on Next and nothing else; it does not repeat a result the subflow already sent.

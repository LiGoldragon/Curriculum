---
description: A flow is asked for its status presentation.
dependencies: [datom, ethos, messaging, psyche]
---

The status presentation is one datom in the StatusPresentation type, and nothing outside it. The type, as a Type ethos:

    Type
    StatusPresentation.{ FlowId Aspect Power Where Vector<Facing> Vector<PsycheRecord> Vector<Question> }
    [ FlowId.String  Aspect.[ Psyche Mind Field ]  Power.[ High Medium Low UltraLow ]  Where.Markdown  Facing.Markdown  PsycheRecord.{ Topic Source Grade }  Topic.String  Source.String  Grade.[ Distilled Raw Notion Reconstructed ]  Question.Markdown  Markdown.String ]

Where is the flow's own concern as it stands: what is deployed, tested, proposed, unknown, and blocked. Facing holds the blocks and open questions the flow faces now, one short item each.

A PsycheRecord is a psyche record the flow is working with. Its Source is written as a sources line: the originating flow's short id and the record's topic. Reconstructed is psyche the flow holds with no record behind it, rebuilt from a transcript or from memory.

A Question is what the flow would like to know, explained on a concrete example from its own work.

The living's words are quoted verbatim. No digest, revision, or id appears except a short flow id. The whole datom stays under 8 KB and is passed directly as the body of one message: `FLOW_ID=<self> hm-send <Psyche High> '<datom>'`. HM constructs the outer `#msg` envelope; do not include it in the body.

## A Mind flow

    { 6288d1 Mind High
      «Tested: `MetaBindExisting` on Flow v2, 44 tests and the workspace check pass locally.
    Proposed: a ConfirmExisting step promoting a bound seat to Active once its native transcript receipt is witnessed.
    Unknown: whether the remote gate recovers from its cache timeouts.
    Blocked: the first bind waits on the full gate and on a Field deploy of Flow 0.4.»
      [ «The remote gate times out on the Prometheus cache.»
        «No bound seat reaches Active until a promotion contract exists.»
        «Who deploys Flow 0.4 while the Codex Field seats are at their limit.» ]
      [ { «native flow messaging» «752e0f messaging» Raw }
        { «flow aspects» «e4a40e flowAspect» Distilled }
        { «bind before message» «6288d1 binding» Reconstructed } ]
      [ «Should a bound seat stay RegisteredUnconfirmed until it answers one message? For example, seat 5f38bc is bound but has never replied, so Message holds every send to it.» ] }

## A Field flow

    { 3c91a7 Field Medium
      «Deployed: Flow 0.4 on a fresh store; its unit runs on the container host.
    Tested: the container binding answers a status query.
    Compensation in place: a restart timer on the binder, per the living:

    > Compensation is when it's put in there and it's welded in place for now to compensate, to make the system run.

    Unknown: whether the binder leak reproduces under load.»
      [ «The binder leaks one socket per restart.»
        «A real send over the live binding waits on Mind's promotion contract.» ]
      [ { «vision operation compensation» «752e0f layers» Raw }
        { «field power tiers» «6cc91b metaflow» Distilled } ]
      [ «When is a compensation removed? For example, the binder restart timer has run three days: does it go when the leak fix is deployed, or when Mind signs off on the fix?» ] }

---
description: Ethos: how an Emission is written while ethos-zero knows no Emission root.
dependencies: []
---

## Emission is written as a Signal

While ethos-zero knows no Emission root, an Emission is written as a Signal in a file named emission.ethos: its inputs are the emissions and its outputs the outcomes. The root's name says nothing; the file's name does, and ethos-zero Check passes it.

```ethos
Signal
[]
[ Command.{ Verb Vector<Argument> } ]  ; emissions
[ Outcome.[ Done Refused.Integer ] ]   ; outcomes
[ Verb.[ List Show ]
  Argument.[ Name.String Quiet ] ]
```

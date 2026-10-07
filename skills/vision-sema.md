---
description: Sema, the database engine of a Nexus, or its record types are being designed or judged.
dependencies: [vision-ethos, vision-nexus]
---

## What sema is

Sema is the database engine of a Nexus, authored in Ethos so the
stored types are visible; its root, Sema, declares record types. It
matters more than nexus, because operational editing should yield the
migration with the edit.

```
Sema
[]                                                        ; imports
[ Lock.{ LockId LockName FlowId LockPaths LockReason } ]  ; record types
                                                          ; the remaining sections are to be decided
```

## Sources

564f55 sema
564f55 ethos
f426777b ethosSourceFiles
62022e8f designPractice
aa4c7747 ethosMonolith

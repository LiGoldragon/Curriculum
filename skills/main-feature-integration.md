---
description: Several branches must come together on main.
dependencies: [version-control, testing]
---

Name one integrator and give it a third clean workspace distinct from every producer workspace.

Freeze the exact reviewed producer revisions before integration.

Integrate from the current target bookmark only in that workspace.

Resolve conflicts and run the affected checks there.

Land portable producers before consumers.

Advance `main` only with explicit authority for that target; a successful producer push is not that authority.

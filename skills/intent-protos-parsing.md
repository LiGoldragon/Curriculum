---
description: A protos parser or printer is designed or judged for how contexts take over and resume.
dependencies: [protos]
---

Protos parsing always happens inside a context, and only the
current context gives shapes their meaning: it defines which
shapes can appear next and which shape completes it. A met shape
announces a type, and that type's context takes over completely
until its completing shape; then the parent context resumes
exactly where it left off. Reading and writing are one walk in
two directions — text lands in typed values, and typed values
project back into the same text.

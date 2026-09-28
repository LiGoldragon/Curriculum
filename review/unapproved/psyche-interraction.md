# psyche-interraction: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/psyche-interraction.md` is the baseline text with today's rulings applied (see below). Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Today's rulings applied in the recovered text

The logging ruling: "it goes to log.md" dropped from the list of what is not vision, and "logged as vision" became "recorded as vision".
The kinds of skills (record 8904b1-17): "Get approval before every skill edit." became "A gold skill changes only on the living's word; the kinds of skills and who stands behind each are in skill-designing."

## Net change, baseline to main (3726da5)

````diff
--- a/skills/psyche-interraction.md
+++ b/skills/psyche-interraction.md
@@ -26,7 +26,7 @@ syntax, a vocabulary, an agent's behavior, the way the work itself is
 done. Not vision, and not an entry: a working instruction (what to do
 now, in what order, at what scope, on which project, through which
-dispatch — it goes to log.md); a process event (a subflow finished, a
+dispatch); a process event (a subflow finished, a
 commit landed, a file was read); session narrative; an acknowledgement
-that rules on nothing. A working instruction logged as vision is a
+that rules on nothing. A working instruction recorded as vision is a
 vision impurity. Supersede an entry by appending; never edit one.
 What the psyche says to help the flow understand vision is context, not vision: it is kept beside the quoted words, never logged or distilled as a statement of its own.
@@ -40,5 +40,7 @@ prompted the statement, what it answers — is kept brief and clearly
 separate from the quoted words.
 
-A quote carries what the psyche said, never what the transcriber wrote: a speech-to-text error is corrected inside the quote itself, and the correction is noted beside it. A quote left with the transcriber's error is a misquote.
+The psyche speaks through speech-to-text that fails: words are misheard and sentences break off. Read for what the psyche means, never for the literal transcript. A quote carries what the psyche said, never what the transcriber wrote. The first flow that hears the psyche corrects each speech-to-text error inside the quote, puts the corrected words in square brackets, and ends the provenance line with `Transcription corrected: "heard" → "meant".` An error kept as spoken is marked [sic]. An unfinished sentence ends in ` ...` and is never logged or acted on as a statement. A relay carries only the corrected text. A message to another flow that rests on the psyche's words carries them: retrieve each verbatim from the raw psyche log and send it as its own `hm-send TARGET --psyche CONTEXT VERBATIM`, context first, beside the machine message. A quote left with the transcriber's error is a misquote.
+
+A tier word beside a model, such as "Sonnet low", names the flow's power tier, which that model carries; it never names effort.
 
 When one message yields entries across several topics, each entry
````

## Each edit

### 8864d42, 2026-09-25 10:01 -0600: psyche-interraction: teach speech-to-text correction discipline

Made by: Claude Sonnet 5 <noreply@anthropic.com>.

Flow e51411, with the living: "We shouldn't pass around verbatim speech to text that has not been corrected ... do we put square brackets around the part that was corrected for clarity?", then "Make the speech-to-text correction skill edit" and "Okay yeah, that's good." The relay clause at its end came later (fec7c66).
View: candidate. The living asked for it and approved it; it makes his own earlier rule usable.

````diff
--- a/skills/psyche-interraction.md
+++ b/skills/psyche-interraction.md
@@ -41,3 +41,3 @@ separate from the quoted words.
 
-A quote carries what the psyche said, never what the transcriber wrote: a speech-to-text error is corrected inside the quote itself, and the correction is noted beside it. A quote left with the transcriber's error is a misquote.
+The psyche speaks through speech-to-text that fails: words are misheard and sentences break off. Read for what the psyche means, never for the literal transcript. A quote carries what the psyche said, never what the transcriber wrote. The first flow that hears the psyche corrects each speech-to-text error inside the quote, puts the corrected words in square brackets, and ends the provenance line with `Transcription corrected: "heard" → "meant".` An error kept as spoken is marked [sic]. An unfinished sentence ends in ` ...` and is never logged or acted on as a statement. A relay carries only the corrected text. A quote left with the transcriber's error is a misquote.
````

### 60be395, 2026-09-25 18:46 -0600: psyche-interraction: tier word beside a model names power tier, not effort

Made by: no model trailer (a Codex seat by the audit reading).

Flow e51411, five minutes after the living: "No Sonnet is low-powered. I didn't say low effort. Low corresponds with Sonnet."
View: candidate. The living's own correction of how his words are read; the multi-seat power names may change, but the reading rule stands.

````diff
--- a/skills/psyche-interraction.md
+++ b/skills/psyche-interraction.md
@@ -43,2 +43,4 @@ The psyche speaks through speech-to-text that fails: words are misheard and sent
 
+A tier word beside a model, such as "Sonnet low", names the flow's power tier, which that model carries; it never names effort.
+
 When one message yields entries across several topics, each entry
````

### fec7c66, 2026-09-25 19:19 -0600: psyche-interraction: relay psyche words verbatim via hm-send --psyche

Made by: no model trailer (a Codex seat by the audit reading).

Flow e51411, after the living: "If there's a psyche ... verbatim with context behind it, that's when you would send them."
View: split. Candidate: "A message to another flow that rests on the psyche's words carries them verbatim, with their context." Fold the `hm-send --psyche` command form into compensation-messenger-clj.

````diff
--- a/skills/psyche-interraction.md
+++ b/skills/psyche-interraction.md
@@ -41,3 +41,3 @@ separate from the quoted words.
 
-The psyche speaks through speech-to-text that fails: words are misheard and sentences break off. Read for what the psyche means, never for the literal transcript. A quote carries what the psyche said, never what the transcriber wrote. The first flow that hears the psyche corrects each speech-to-text error inside the quote, puts the corrected words in square brackets, and ends the provenance line with `Transcription corrected: "heard" → "meant".` An error kept as spoken is marked [sic]. An unfinished sentence ends in ` ...` and is never logged or acted on as a statement. A relay carries only the corrected text. A quote left with the transcriber's error is a misquote.
+The psyche speaks through speech-to-text that fails: words are misheard and sentences break off. Read for what the psyche means, never for the literal transcript. A quote carries what the psyche said, never what the transcriber wrote. The first flow that hears the psyche corrects each speech-to-text error inside the quote, puts the corrected words in square brackets, and ends the provenance line with `Transcription corrected: "heard" → "meant".` An error kept as spoken is marked [sic]. An unfinished sentence ends in ` ...` and is never logged or acted on as a statement. A relay carries only the corrected text. A message to another flow that rests on the psyche's words carries them: retrieve each verbatim from the raw psyche log and send it as its own `hm-send TARGET --psyche CONTEXT VERBATIM`, context first, beside the machine message. A quote left with the transcriber's error is a misquote.
````

### 3726da5, 2026-09-28 10:34 -0600: main-flow, psyche-interraction: say once what a log holds; drop small steps from the log

Made by: Claude Fable 5.1 <noreply@anthropic.com>.

Flow 8904b1. Today's logging ruling; applied in the recovered text.

````diff
--- a/skills/psyche-interraction.md
+++ b/skills/psyche-interraction.md
@@ -27,5 +27,5 @@ done. Not vision, and not an entry: a working instruction (what to do
 now, in what order, at what scope, on which project, through which
-dispatch — it goes to log.md); a process event (a subflow finished, a
+dispatch); a process event (a subflow finished, a
 commit landed, a file was read); session narrative; an acknowledgement
-that rules on nothing. A working instruction logged as vision is a
+that rules on nothing. A working instruction recorded as vision is a
 vision impurity. Supersede an entry by appending; never edit one.
````


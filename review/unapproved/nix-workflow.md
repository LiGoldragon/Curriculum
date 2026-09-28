# nix-workflow: edits made after the baseline

Baseline a7d2f4f (2026-09-10). The recovered `skills/nix-workflow.md` is the baseline text. Every edit made on main after it is set out here, whole, against the baseline, with a view on each.

## Net change, baseline to main (3726da5)

````diff
--- a/skills/nix-workflow.md
+++ b/skills/nix-workflow.md
@@ -14,4 +14,6 @@ Keep local overrides transient.
 Run Nix builds only through configured remote builders; never build locally.
 Run Nix evaluations and builds independently.
+Test with binaries and scripts packaged by Nix, built through configured remote builders; do not silently fall back to raw local compilation, Cargo tests, or local package-manager rebuilds. A Nix command alone does not prove offload: retain the remote-builder evidence for a claimed remote build.
+Create Rust-only repositories for Rust source and keep mutable data or content in separate data inputs or repositories, so data changes do not invalidate Rust compilation.
 Treat managed output as evidence, not a patch target.
 Keep evaluation and activation evidence separate.
````

## Each edit

### 21672ff, 2026-09-18 18:15 -0600: Require remote Nix-built test artifacts

Made by: no model trailer (a Codex seat by the audit reading).

Flow cf3553 (Field Astra), with the living (flows/cf3553/vision/operational-nixBuiltTestingAndRustDataSeparation.md): "make sure all the testing uses nix built binaries ... built on the remote builders ... create rust-only repos for rust". On 2026-09-24 (flow 752e0f) the living partly corrected it: when the builder is not reachable, build locally.
View: candidate, corrected. Keep "test with Nix-built binaries" and "Rust-only repositories, data in separate inputs" (the living's words). Drop "do not silently fall back" (against the later correction) and "A Nix command alone does not prove offload: retain the remote-builder evidence", which goes by the living's ruling of 2026-09-28: a build reported green is green wherever it ran.

````diff
--- a/skills/nix-workflow.md
+++ b/skills/nix-workflow.md
@@ -15,2 +15,4 @@ Run Nix builds only through configured remote builders; never build locally.
 Run Nix evaluations and builds independently.
+Test with binaries and scripts packaged by Nix, built through configured remote builders; do not silently fall back to raw local compilation, Cargo tests, or local package-manager rebuilds. A Nix command alone does not prove offload: retain the remote-builder evidence for a claimed remote build.
+Create Rust-only repositories for Rust source and keep mutable data or content in separate data inputs or repositories, so data changes do not invalidate Rust compilation.
 Treat managed output as evidence, not a patch target.
````


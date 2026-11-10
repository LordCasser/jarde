# S2a reference-constraints frozen dev fixture

This directory is an exact-byte copy of the four frozen dev JARs, the `ReferenceCases.java` source, its `ReferenceDevDriver.java`, and the dev class-source pins. It contains no holdout files and does not claim S2a acceptance.

## Provenance

The authoritative source manifest is `openspec/changes/restructure-demand-driven-recovery/results/s2a-variable-legalization-v1/reference-constraints-v1/manifest.json`; its SHA-256 and copied-file hashes are recorded in `input-pins.json`. The original dev source manifest and frozen-before-CLI pins are also identified by `dev-class-source-pins.json` and `.sha256`.

The compiler matrix is Corretto 8 and OpenJDK 23, each with debug and no-debug builds. The source targets Java 8. The original class-source fixture JARs are pinned in `input-pins.json`.

## Binary companions

Each original JAR contains `ReferenceCases.class` plus `ReferenceCases$Answer.class`, `$FirstAnswer.class`, and `$SecondAnswer.class`. The three nested class files are binary companions referenced by the root source and provide the interface and implementation types. Full-class validation removes only the original top-level `ReferenceCases.class` from the dependency JAR, retaining these companions; generated output is supplied first on the class path.

## Existing evidence

- Original frozen dev oracle: `openspec/changes/restructure-demand-driven-recovery/results/s2a-variable-legalization-v1/reference-constraints-v1/` (8 compiler builds and 16 runtime combinations across JDK 8/23).
- Generated-source full-class validation: `.../reference-constraints-v1/results/constraints-r1/` (8 successful full-class compilations and 16 runs; the fixed driver checks the explicit cast success and `ClassCastException` branch).

Those counts describe the existing pinned evidence only. This fixture copy adds no test and does not itself establish overall S2a completion.

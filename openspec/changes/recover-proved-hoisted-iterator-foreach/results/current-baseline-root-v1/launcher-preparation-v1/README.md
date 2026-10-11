# TestNestedLoops current-CF11 baseline launcher

This private launcher reuses the frozen `TestNestedLoops` inputs and collector configuration from the accepted `TestNestedLoops2` baseline harness. It preserves the Java source, manifest, 29-command comparison plan, compiler/JADX/runtime tool pins, and existing collector/checker protocol. The collector uses the generic controls collector and checker v7, and binds to the current constructor-conditional final CLI and final build receipt.

Before running, the launcher refuses to proceed if the workspace `target` directory, its new output directory, checker result, or launcher receipt already exists. It verifies every frozen input and provenance file against the root-v1 frozen-input pins, checks the exact upstream method slice against the live upstream source, and checks the build receipt's status and CLI object. It imports only the pinned own-CI verifier v2 source-closure functions (`source_paths` / `pin_sources`), which dynamically reconstruct the current 4,727-file closure; the resulting full pin map must equal both final-build before/after maps before collection. It repeats the closure comparison after collection, records whether `target` is absent, and runs the pinned checker on the collector receipt while preserving collector/checker failures.

The launcher itself has only been AST-parsed and statically reviewed. It has not been run; no Java/JADX/CLI/Cargo/Git/network/toolchain command was executed. The expected source count and paths are configuration checks, not a claim that this new baseline has passed.

Run only after the shared workspace `target` is absent and no competing Rust diagnostic is using it:

```text
python3 /private/tmp/jarde-hoisted-iterator-current-baseline-prep-luna-v1/run_baseline.py
```

The unique output is `/private/tmp/jarde-hoisted-iterator-current-baseline-root-v1`; existing results are never overwritten.

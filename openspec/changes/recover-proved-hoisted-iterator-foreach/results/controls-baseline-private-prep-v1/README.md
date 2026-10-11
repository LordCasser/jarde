# Hoisted iterator dual-JDK full-class baseline launcher

This preparation freezes the two-target Java 8 controls byte-for-byte under `inputs/`, then reuses collector v3 and checker v7 with the current final CLI/build binding. It retains the complete baseline collector configuration (JDK 8/JDK 23, JADX, class parser, guard, and scanner pins) from the accepted TestNestedLoops2 execution. The collector compiles and runs each original and generated source set as whole classes, keeps raw stdout/stderr, and records compile failures as observations. No generated Java is edited by this launcher.

The manifest declares exactly `cf11.HoistedIteratorPositive` and `cf11.HoistedIteratorRefusal`, with `cf11.Runner`; no helper classes are emitted by the source design. The output class-set expectation is therefore derived from this manifest by the existing collector/checker. The launcher does not hardcode a total command count or successful compilation count.

Before collection it checks the 0444 frozen sources and manifest against both the frozen-input pins and original source-preparation pins, validates the final CLI/build receipt binding, reconstructs the live current source closure dynamically through the pinned own-CI verifier functions, requires all 4,727 source pins to match both build before/after maps, and refuses an existing workspace `target` or output. After collection/checking it repeats source and frozen-input checks, confirms target cleanup, and independently checks every raw stream's byte count and SHA. Its receipt records exact generated compile failures; if any exist, the receipt status is `capture_checked_with_compile_failures`, never an undifferentiated success status.

The launcher and inputs are prepared only. Python AST/manifest/pin checks were performed, but the launcher was not executed. No Java/JDK, JADX, CLI, Cargo/Rust build, Git, or network command was run. The collector/checker will produce the actual command count and outcome; this README makes no prediction about those results.

Run the launcher only when the workspace `target` is absent and no other workspace command is using it:

```text
python3 /private/tmp/jarde-hoisted-iterator-controls-baseline-prep-luna-v1/run_controls_baseline.py
```

New output: `/private/tmp/jarde-hoisted-iterator-controls-baseline-root-v1`. Existing outputs are refused.

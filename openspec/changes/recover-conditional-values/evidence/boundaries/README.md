# Conditional-value boundary evidence

`ConditionalBoundaryCases.java` is compiled with Java 8 target and no debug data; `ConditionalBoundaryCasesRunner.java` remains source-only. The separate `MarkerValue`, `LeftValue`, and `RightValue` sources are compile-time dependencies. The `unknownType` analysis deliberately opens only `ConditionalBoundaryCases.class` with `single-class` policy, so those hierarchy definitions are absent from the recovery scope. All classfiles are javac outputs (version 52.0), not edited or spliced files.

Rebuild and verify the original input:

```sh
javac --release 8 -g:none -d v8 ConditionalBoundaryCases.java MarkerValue.java LeftValue.java RightValue.java
RUN_DIR=$(mktemp -d)
javac --release 8 -g:none -cp v8 -d "$RUN_DIR" ConditionalBoundaryCasesRunner.java
java -Xverify:all -cp "$RUN_DIR:v8" ConditionalBoundaryCasesRunner
```

The primary subject class is 950 B, major version 52, SHA-256 `04d7291d6da56dc802a3717dc5e2b78ccbab552b15f4e50f7b4208b5dbb96eb5`; the three dependency classes are 85/185/186 B and are compiled from their adjacent Java sources. `original-run.txt` and `jadx-run.txt` have the same SHA-256 `3b7a843b2a0f3627655e2a153a16f5bfe7bd85912bbc357128f8e505e8d87757`. It exercises a loop back edge, a try/catch around the conditional producer, both normal arms, both selected-arm exceptions, and a reference-valued conditional whose common interface is intentionally unavailable to the single-class recovery request.

JADX 1.5.6 recovered a compilable whole class; its package-adjusted source and runner compiled for Java 8 and passed `-Xverify:all`, producing byte-for-byte identical output (`jadx-run.txt`). Current `target/debug/jarde-cli` SHA-256 `a3a29b59aa85c2d174b06033f874eb7d60a104dbccf1d20f5d88908273e92dd5` preserves refusal markers for the loop and handler paths and does not form a conditional for `unknownType`; full text is `jarde.java.txt`. Compiling that full presentation with the original helper sources fails because `guarded` references a local outside the recovered handler scope (`jarde-javac.stderr.txt`). This records current behavior, not an assertion that the whole method is unrepresentable.

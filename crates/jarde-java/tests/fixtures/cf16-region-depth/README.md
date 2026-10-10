# CF16 region-depth stop fixture

`javac8/BranchFinally.class` and `javac23/BranchFinally.class` are byte copies of the depth-33
classes recorded in `depth-field-branch-root-v1/run-v2/result.json`. The shared source and Runner
are copied verbatim from the corresponding depth-33 preparation. `copy-manifest.json` records the
input result identity, copied-file hashes, and the original compile and `java -Xverify:all` exits.
Both original class/Runner pairs compiled and verified successfully under their recorded JDKs.

The `p3_patterns.rs` regression calls only `run(I)I` with the existing limits and checks
the observed `Interrupted(jre_recursion_bound, at=450)` stop plus the no-publication contract. It
does not treat this as successful syntax recovery: the stopped report is `NotProduced` with no
text, source-map segments, rules, regions, or initializer. The depth-02 cases remain explanation
fallbacks and are deliberately not represented as positive recovery evidence.

The root v7 gate passed all 85 pattern tests after assigning this deep fixture an explicit
8 MiB test thread stack. The first v6 run overflowed the smaller libtest default stack; that
failure is retained. This fixture proves the Region stop/publication contract at the recorded
stack size, not safety on every host thread stack size. Production code is unchanged.

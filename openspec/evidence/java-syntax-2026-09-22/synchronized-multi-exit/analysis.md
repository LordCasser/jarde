# Synchronized multi-exit source audit

`SynchronizedMultiExit.choose` has one `synchronized (lock)` body with a boolean branch. Each arm calls a value producer and returns its result, so javac emits two normal monitor exits followed by their respective `ireturn`s. `produce(1)` can throw a known exception; the runner checks that exception identity and that the producer's trace happened before monitor cleanup. The cases exercise both return values and the throwing producer path.

The source compiles with `javac --release 8 -g:none`. Its complete class is 646 bytes, SHA-256 `6658190aa7575f8be5095d71aa3ee07aa453d464666b1d34dea6a962950c45f2`, with four Code attributes; full private-method bytecode is in `javap.txt`. In `choose`, BCI 3 enters the monitor; the true branch exits at BCI 12 and returns at 14; the false branch exits at 19 and returns at 21; handler cleanup exits at 24 and rethrows at 26. Exception rows `[4,14)` and `[15,21)` target the same handler at 22, while `[22,25)` protects the handler's own exit.

The three verified original outputs are:

```text
first:return=10:trace=1
second:return=20:trace=2
first-throws:throw=java.lang.IllegalStateException:same=true:trace=1
```

The frozen CLI `/tmp/jarde-cli-static-root-after` has SHA-256 `8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44`. JADX 1.5.6's unedited full class compiles and passes `-Xverify:all`, with output equal to the original. Jarde's complete unedited class has six `@bytecode` references and fails `javac` with a missing-return error; stderr is retained, so no Jarde execution result is claimed.

## Current guard and Plan/Region boundary

`guard::monitor` currently collects every `monitorexit` in the method and requires the exact pair `[normal_exit, handler_exit]` (`crates/jarde-java/src/guard.rs`, around the `exits` collection at line 1712). It then validates one normal exit, one immediately following return or transfer, one protecting row, and one optional return BCI. This sample has three exits, two normal return BCIs, and two disjoint protected rows that share the handler. It is rejected by this exact-shape guard before the branch's return paths can be presented.

The `Plan`/`Shape::Monitor` payload currently carries one `returns: Option<u32>`. The builder takes one body range and appends at most one synchronized return. Ordinary `Region::If` already represents the condition and the two arms, so this does not appear to require a new general Region kind. It does require widening the monitor proof/plan to carry each normal exit and its return endpoint, proving all protected fragments lead to the same monitor handler, and attaching each post-exit return to its corresponding arm inside the synchronized body. Current single-span body ownership and single-return attachment cannot express that mapping as-is. This is evidence about the existing boundary, not a production change proposal.

Replay with `python3 run_audit.py` from this directory. The script derives the evidence directory from its own path, verifies the full frozen CLI hash before and after, compiles the Java 8 source, runs the original and each complete recovered class under `-Xverify:all` if compilation succeeds, and preserves every status/log. Temporary work defaults to `/tmp/jarde-synchronized-multi-exit-audit`; `JARDE_SYNCHRONIZED_WORK` sets an independent work directory.

Root copied the directory to `/tmp/jarde-sync-multi-root-4sq6hO` and reran with separate work. `summary.json` is byte-for-byte identical; the 646B/4Code class, three original/JADX equal rows, six Jarde references and full-class missing-return compiler failure all reproduced.

Root code review adds a planning constraint: `Region::Guard` presently carries only a flat `(start,end)` body range through `Plan`, and `build::body_range` writes that range instruction by instruction. `Region::If` exists, but a nested region walk cannot simply be called here: the ordinary walker treats the protected range's exception edges as guard boundaries. A future implementation must either give the guard proof an explicit per-arm ownership/return mapping or let the region walker recurse under a **certified handler boundary**; it may need a small new nested-body mechanism. Merely allowing three `monitorexit`s in `guard::monitor` would still emit both return paths in a flat body and lose their branch relationship. This architecture work is deliberately separate from the currently implemented compound-`+=` and narrow finally changes.

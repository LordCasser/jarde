# Finally cleanup snapshot and boundary evidence

This source-only Java 8 fixture exercises three completion boundaries in `CleanupBoundaries`:

- `snapshotReturn` reads field `value` for a return, then cleanup changes the field from 41 to 99. It must return the saved value 41. On the exceptional path, `mark(1)` throws the known `TRY_FAILURE`; cleanup must append 2 and preserve that same exception.
- `cleanupThrowsOverReturn` has a pending return from `mark(3)`. Cleanup appends 4 and throws `CLEANUP_FAILURE`, which becomes the observed completion.
- `cleanupThrowsOverTryThrow` throws `TRY_FAILURE` from the try, then cleanup appends 6 and throws `CLEANUP_FAILURE`; cleanup's exception replaces the original one.

The source class is compiled with `javac --release 8 -g:none`. Baseline is 947 bytes, SHA-256 `9aed4bcc1f1e85d7222217592291091de1d42590fdd38edf3525ca962ffbd2e9`, with six Code attributes including private methods (`javap -p -v -c`). `baseline/javap.txt` preserves full disassembly. In `snapshotReturn`, BCI 5–14 is protected and handler target 26 contains the cleanup copy. The normal return value is saved at BCI 13 before cleanup writes 99; BCI 24 returns the saved value. The handler copy at BCI 27–38 repeats cleanup then rethrows.

Three class files were run with `java -Xverify:all` before the decompiler comparisons:

- `baseline`: ordinary javac output.
- `copy-divergence`: a same-length edit changes only the exceptional cleanup copy's `iconst_2` at `snapshotReturn` BCI 32 to `iconst_3`. The verifier accepts it; when `mark(1)` throws, the observed trace is 13 instead of 12.
- `range-narrowed`: changes only `snapshotReturn`'s exception-table `start_pc` from BCI 5 to BCI 9. Both are instruction boundaries. The verifier accepts it; `mark(1)` now throws outside the protected interval, so the observed trace is 1 and cleanup does not run.

The exact source-class outputs appear in each variant's `original-run.txt`. The baseline is:

```text
snapshot:return=41:field=99:trace=12
snapshot-throw:java.lang.IllegalArgumentException:same=true:trace=12
cleanup-return:throw=java.lang.IllegalStateException:cleanup=true:try=false:trace=34
cleanup-try-throw:throw=java.lang.IllegalStateException:cleanup=true:try=false:trace=56
```

JADX 1.5.6 output is retained complete and unedited for each input class, with whole-class javac and `-Xverify:all` logs. It compiles for all variants, but its outputs are not a correctness oracle: even baseline it reports trace 344 for `cleanupThrowsOverReturn` (duplicates the cleanup side effect), and on the altered classes it emits behavior that does not match their verified bytecode. See each `jadx-run.txt` and `jadx.java.txt`.

The frozen Jarde CLI path is `/tmp/jarde-cli-static-root-after`, actual SHA-256 `8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44` (the requested prefix matched). For each class, the complete output/report are `jarde.java.txt` and `jarde-report.txt`. Whole-class compilation fails with three missing-return errors; logs are retained, and no Jarde runtime behavior is claimed.

Replay from any directory with `python3 run_audit.py`. It checks the frozen CLI hash prefix, compiles the original source, applies the two documented class-file edits, verifies every class before decompilation, then preserves the original/JADX/Jarde full-class compile and run results. Class files and javap/hash/Code records are stored in each variant directory. Temporary working data is under `/tmp/jarde-finally-snapshot-boundaries-audit`.

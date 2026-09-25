# Implicit cleanup exception from a non-overriding finally body

`ImplicitCleanup.run()` has one finally body, and that body contains only the observable call `cleanup()`. The helper appends 9 to the trace and can throw a known exception internally; there is no explicit `throw` statement in the finally body. The four runner cases cover normal return, a preserved try exception, cleanup failure replacing a pending return, and cleanup failure replacing the try exception.

The source compiles with `javac --release 8 -g:none`. The main class is 787 bytes, SHA-256 `924437916dc278eefe3b83cdcf3bad14bfb3cf8b9e44c027786a89f0712247b6`, with five Code attributes. Complete private-method disassembly is `javap.txt`. The original passes `java -Xverify:all` and prints:

```text
normal:return=2:trace=29
try-throws:throw=java.lang.IllegalArgumentException:try=true:cleanup=false:trace=19
cleanup-over-return:throw=java.lang.IllegalStateException:try=false:cleanup=true:trace=29
cleanup-over-try-throw:throw=java.lang.IllegalStateException:try=false:cleanup=true:trace=19
```

The frozen CLI `/tmp/jarde-cli-static-root-after` has SHA-256 `8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44`. JADX's complete class compiles and verifies, but on cleanup failure it outputs trace 299 instead of 29 for cleanup overriding a return: its reconstructed code calls the cleanup effect twice on that exceptional path. The full output is retained as an empirical comparison, not an oracle. Jarde output has two `@bytecode` references and its full unedited class fails compilation; stderr is retained, with no Jarde run claimed.

This evidence concerns the semantic boundary where a cleanup *call* throws. The existing OpenSpec policy still treats `catch_type == 0` as ambiguous and does not restore it as source `finally`. The fixture is available for a future mechanism/acceptance decision; it does not alter that policy.

Replay with `python3 run_audit.py` from this directory. It checks the full frozen CLI hash before and after, compiles/runs the source and complete recovered classes with Java 8 and `-Xverify:all`, and retains the raw results. Temporary files default to `/tmp/jarde-finally-implicit-cleanup-audit`; `JARDE_FINALLY_IMPLICIT_WORK` sets an independent work directory.

Root copied the directory to `/tmp/jarde-finally-implicit-root-07ws8h`, ran the script with an independent work directory, and obtained a byte-for-byte identical `summary.json`. The original's four rows, JADX's one `299` divergence, Jarde's two references and full-class compiler failure all reproduced without editing generated text.

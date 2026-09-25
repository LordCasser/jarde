# Java 8 `assert` source-recovery audit

This is a source-only probe built from `AssertProbe.java`. It uses javac's normal Java 8 assertion lowering, without hand-patching class bytes or editing any decompiler output. `AssertProbeRunner.java` observes condition and message side effects, assertion success/failure, and identity of exceptions thrown by either expression.

Replay from the repository root with:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/assert-syntax/run_audit.py
```

The script checks the SHA-256 of `/tmp/jarde-cli-deferred-budget-908c`, compiles with `javac --release 8 -g:none`, records `javap -c -p -v`, extracts the complete class with JADX, and asks the frozen CLI for one complete jarde class. It recompiles each unedited complete class together with the same runner, then starts separate `java -Xverify:all -ea` and `-da` processes. If a source does not compile, its runtime comparisons are marked skipped.

`AssertProbe.class` is the raw javac output that both decompilers receive. `*-javac.log`, `*-ea.txt`, `*-da.txt`, and the equality diffs are direct command outputs; `summary.json` records tool versions, hashes, statuses, and comparisons. `analysis.md` explains the observed shape and the jarde compile boundary.

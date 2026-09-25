# Non-overriding `try/finally` slice

This fixture isolates the minimal finally completion shape for a future whole-class acceptance check. `FinallyNormal.run()` contains only `try { return mark(1); } finally { mark(2); }`. There is no return or throw in the finally body, and thus no overriding completion shape. The runner checks a normal return and a `mark(1)` exception; both calls append their digit before possibly throwing, so the trace shows whether finally ran.

Compile and run used `javac --release 8 -g:none` and `java -Xverify:all`. The original class is 593 bytes, SHA-256 `6c1e6ee18ca8370ffc5ad44f8ad911a000ed34f6c258d95d603004d4aeab701a`, with three Code attributes (constructor, `run`, and `<clinit>`). In `run`, BCI 0–4 computes/saves the try return, 5–11 executes finally and returns the saved value, and 12–19 executes the exceptional finally copy and rethrows. The sole exception-table entry has `catch_type == 0`; this fixture does not claim it is recoverable as source `finally` under current OpenSpec rules.

Original and JADX whole-class builds both pass verification and produce exactly:

```text
normal:return:1:12
throw:java.lang.IllegalArgumentException:same=true:12
```

JADX output is preserved verbatim. Frozen Jarde CLI SHA-256 is `88f2e7aa9b5b8172da02f5af5d4d2c029e72a52d9b3298b46b82774e0bd5fdfd`. Its unedited whole-class output has two BCI references and fails `javac` with a missing-return error; the complete compiler log is retained. No Jarde runtime result is claimed.

Replay with `python3 run_audit.py`. The script checks the frozen CLI hash, recompiles the source and the complete recovered classes, records class hash/Code count/javap, and runs successful builds under `-Xverify:all`. Temporary class trees are under `/tmp/jarde-finally-nonoverriding-audit`.

Root copied the entire directory to `/tmp/jarde-finally-normal-root-7pthED` and reran `run_audit.py` with separate `JARDE_FINALLY_WORK`; replay exited 0 and reproduced the exact 593-byte/3-Code hash, both original/JADX outcomes, two Jarde bytecode references and the unedited complete-class missing-return compiler failure. The script now resolves evidence relative to itself and checks the class/CLI hashes before and after. This is the positive shape for `recover-proved-finally-cleanup`, not evidence that the current Jarde already emits `finally`.

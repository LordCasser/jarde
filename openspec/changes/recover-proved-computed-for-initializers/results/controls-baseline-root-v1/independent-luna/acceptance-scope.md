# Computed initializer controls: independent acceptance boundary

Independent verifier: `verify.py`. It re-hashes all 20 command stdout/stderr raw streams and verifies full argv per label; binds the root runner script, frozen CLI, input sources, patcher, and both JDK toolchains. Result: PASS.

The original and pending `ComputedForControls.class` files match the recorded SHA-256 pins for Java 8 and Java 23. Both compile/runtime legs exited 0; both original/pending runtimes used `-Xverify:all`. The compiled `ComputedForControlsRunner.class` is byte-identical in original and pending classpaths. The two method Code arrays differ only at the exact six-byte `store; invokestatic g` ↔ `invokestatic g; store` swaps in `find([III)I` and `shared(I)I`; class lengths, constant-pool bytes, every other class byte, and therefore exception tables/StackMapTable attributes are unchanged. `javap` raw confirms original and pending instruction order and BCIs.

Each JDK produced the same nine-row raw output for original and pending: hit 2, miss -1, f-throw before g, g-throw after f, shared f value, post-loop read, outside write, nested-scope enter/skip. The hashes in execution summaries match the actual `.raw` files.

All eight current-CLI profiles are independently checked against their input class and emitted-source SHA. Default/all text and source maps agree for each JDK and side. In every profile `find`, `shared`, and `scoped` are fallback; `observedAfter` and `externalWrite` are structured `while`; no method contains `for`. Thus this run validates only legal bytecode, effect ordering, and current refusal/fallback boundaries. It does not demonstrate successful computed-initializer recovery.

The supplemental `v2/ComputedForDirectReturn.java` and `v2/ComputedForDirectReturnRunner.java` give root a smaller call-init candidate: no try/catch, a hit return only inside the loop, and only `return -1` after it. Predicted rows are `hit:2:1`, `bounded-hit:4:1`, `miss:-1:1`, `empty-range:-1:1` (result: initializer-call count). These sources are uncompiled and unrun; expected behavior and any for recovery are unverified.

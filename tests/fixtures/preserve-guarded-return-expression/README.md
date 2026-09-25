# Return values across synchronized exits

`GuardReturnEffects.java` is compiled with javac 23.0.1 using `--release 8 -g:none` into `v8/`.
It provides two verifier-valid boundaries for `preserve-guarded-return-expression`:

* `effect(boolean)` reads `n`, calls `afterRead` while still synchronized, and returns the earlier
  value. `afterRead` increments an observable call counter, changes `n`, and can throw. A recovery
  that moves the read after that call, omits the call, or moves the return outside the monitor changes
  the runner's result, count, exception, or lock-release observation. Jarde deliberately retains the
  method's bytecode fallback because this Guard proof does not admit the independent body statements.
* `nested(boolean)` repeats the same effects under nested monitors. The one-monitor proof rejects its
  monitor ownership; the return-exit exemption must not be borrowed by it.

| class | bytes | SHA-256 |
| --- | ---: | --- |
| `GuardReturnEffects.class` | 898 | `7da6eaa25c3230fd0967eb9ccd310a0c4849b01b23f609fb457efac29715ad27` |
| `GuardReturnEffectsRunner.class` | 1,383 | `686c98fa0b48d8ba0b31c0f7c8005af11fca7a8c42fed708003600175adedc2c` |

Reproduce the committed classes and verify the runner with:

```sh
javac --release 8 -g:none -d v8 GuardReturnEffects.java GuardReturnEffectsRunner.java
java -Xverify:all -cp v8 GuardReturnEffectsRunner
```

The runner output and decompiler comparison live in
`openspec/evidence/java-syntax-2026-09-24/guarded-return-expression/`.

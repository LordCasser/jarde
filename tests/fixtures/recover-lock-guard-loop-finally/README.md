# Lock-guard loop-finally fixture (`recover-lock-guard-loop-finally`)

Three Java 8 classes, each compiled twice: `v8/` with javac 23.0.1 `--release 8 -g:none`, and
`v8-javac8/` with the real javac 8 (Corretto 1.8.0_432) `-g:none`. Neither leg has a
`LocalVariableTable` or a `LineNumberTable`, so every name the recovery layer writes is a derived
`localN` name and every shape decision is a decision about control flow, not about debug metadata.

`tests/recover_lock_guard_loop_finally.rs` reads the committed bytes; this README is the fixture's
own contract (the sources, the shapes, the commands and the recorded behavior).

## The anchor

`LK.java` is the explicit-lock patrol's own source, verbatim
(`openspec/evidence/java-syntax-2026-10-05/explicit-lock-patrol/fixture/LK.java`). Its frozen
`lk.jar` (javac 23.0.1, `--release 8`, **with** line numbers) stays the patrol's artifact and is what
the test renders first; the two legs here are the same source recompiled, so the slice's claim is
that the presentation is one shape and not one compiler's lowering. Measured: the patrol's jar and
both legs render **byte-identically**.

| member | shape | what the certificate proves |
| --- | --- | --- |
| `put()V` | `lock(); try { while (count >= 2) await(); count++; signalAll(); } finally { unlock(); }` | the void completion: the release copy falls through to a transfer onto the method's own value-less `return` |
| `take()I` | the same guard with `while (count <= 0)`, a decrement, and `return count;` inside the try | the saved-return completion: the body stores the value, the release runs, the return reads the slot back |
| `tryLockQuick()Z` | `if (lock.tryLock()) { try { count += 10; return true; } finally { unlock(); } } return false;` | the acquisition is the one invocation before the protected range, consumed by the guard's own branch |

The bytecode of all three (javap, this leg): the one catch-all row covers the body only — `put`
`[7, 46) -> 56`, `take` `[7, 50) -> 59`, `tryLockQuick` `[10, 23) -> 32` — with **no
self-protection row** over the handler's copy, and the two copies of each member are the same three
instructions `aload_0; getfield lock; invokevirtual unlock()V`.

## The negatives

`LockGuardNegatives.java` breaks one link of the certificate's proof per member, and every member is
**verifier-valid** (compiled from this source, so a refusal is evidence about the proof rather than
about damaged bytes):

| member | the link it breaks | the refusal it keeps |
| --- | --- | --- |
| `lockMismatch()V` | the acquisition takes `lock`, both copies release `other` | `jre_guard_finally_copy` at BCI 27 + the uncovered quote `[37, 27]` |
| `guardedRelease()I` | a release `javac` protects: the table states `[8, 30) -> 39` **and** the self-protection row `[39, 41) -> 39` | `local 1 crosses a quoted fallback region …` |
| `localLockRewritten()V` | the receiver is a local the body rewrites, so the copies read a merged value | `local 1 crosses a quoted fallback region …` |
| `throwingRelease()V` | an interface call on another field, whose range the table states from BCI 0 | `block at BCI 0 leaves through exception handler 0 …` |

`LockGuardProbe.java` is the io-wrapping patrol's resource-across-finally shape as a single-method
probe (one local handle, a construction chain before the range, a loop, a saved return, a protected
release). It is the boundary this slice measures: the certificate proves an acquire/release pair on
one **field read**, so the probe keeps its refusal and the whole-class IO acceptance stays another
slice's.

The true two-copy non-lock `finally` shapes are the P3 2.4 sample's `fin`/`catchFinally`
(`tests/fixtures/p3-handlers/`): no acquire/release pair at all, and their `jre_guard_finally_copy`
refusal is unchanged by this slice.

## Reproduce and verify the checked-in bytes

```sh
cd tests/fixtures/recover-lock-guard-loop-finally
javac --release 8 -g:none -d v8 LK.java LockGuardNegatives.java LockGuardProbe.java
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac \
    -g:none -d v8-javac8 LK.java LockGuardNegatives.java LockGuardProbe.java
shasum -a 256 v8/*.class v8-javac8/*.class
java -Xverify:all -cp v8 LK
java -Xverify:all -cp v8-javac8 LK
```

The class-file versions are 52.0. Both legs' `LK` print the same line under `java -Xverify:all`:

```
1/0/true
```

That line is the behavior the `finally` must keep: two `put`s fill the bounded buffer, the first
`take` empties it to `0` after `1`, and `tryLockQuick` acquires the lock — so a release that ran
twice, ran early, or ran on another object would change it.

| leg | class | bytes | SHA-256 |
| --- | --- | --- | --- |
| `v8` | `LK.class` | 1605 | `cab109d4c9212c051db345767c5e3ff7fbf2198340dd733406b2ee09a4ea2987` |
| `v8` | `LockGuardNegatives.class` | 1254 | `dd5cfef285dde7860cab94482b05ab74fde2625977fe1b03c9842b30dd74152b` |
| `v8` | `LockGuardProbe.class` | 699 | `6cdaea4e4950810fd12dd8162d3d6b0507e0786abec9c522605fbaeef25ac43d` |
| `v8-javac8` | `LK.class` | 1605 | `ecbff3abdddb58b4ab470ae34336b7545c48ce6109f09d6ed5ce1834a370752d` |
| `v8-javac8` | `LockGuardNegatives.class` | 1260 | `7f6b523e0a94a84019b6663fb823ef9b1916a1839d2fa19a39c891ff89b62b93` |
| `v8-javac8` | `LockGuardProbe.class` | 702 | `32794dc62e5d304cfde296ef3b0b0e9cc1eb9381e36356c70ea1a59a96184ab7` |

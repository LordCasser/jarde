# Nested-lock finally fixture (`recover-nested-lock-finally-bodies`)

The multi-lock patrol's own source plus this change's shapes, each compiled twice: `v8/` with
javac 23.0.1 `--release 8 -g:none`, and `v8-javac8/` with the real javac 8 (Corretto 1.8.0_432)
`-g:none`. Neither leg has a `LocalVariableTable` or a `LineNumberTable`, so every name the
recovery layer writes is a derived `localN` name and every shape decision is a decision about
control flow, not about debug metadata.

`tests/recover_nested_lock_finally_bodies.rs` reads the committed bytes; this README is the
fixture's own contract (the sources, the shapes, the commands and the recorded behavior).

## The anchor

`ML.java` is the multi-lock patrol's own source, verbatim
(`openspec/evidence/java-syntax-2026-10-08/multi-lock-patrol/fixture/ML.java`, SHA-256
`1b5d602d…`). Its frozen `ml.jar` (javac 23.0.1, `--release 8`, **with** line numbers) stays the
patrol's artifact and is what the test renders first; the two legs here are the same source
recompiled, so the slice's claim is that the presentation is one shape and not one compiler's
lowering. Measured: the patrol's jar and both legs render **byte-identically**.

| member | shape | what the certificate proves |
| --- | --- | --- |
| `nestedLocks()V` | `a.lock(); b.lock(); try { count++; } finally { b.unlock(); a.unlock(); }` | the multi-statement clause: two acquisitions, two three-instruction release groups, and the groups pair with the acquisitions in **reverse** order |
| `interruptibly()V` | `a.lockInterruptibly(); try { count++; } finally { a.unlock(); }` | the acquisition may throw, and the row leaves it outside: row `[7,17) -> 27` begins after the call at BCI 4, so a throw from it never enters the range |
| `multiAwait()I` | two `await()` sites under one lock | **not this slice's**: the patrol recorded it as recovered, and the measurement here says otherwise — see below |

The bytecode of the two shapes (javap, this leg): `nestedLocks`'s one catch-all row is
`[14,24) -> 41` with the two releases at BCI 24–35 (normal) and 42–53 (handler); `interruptibly`'s
is `[7,17) -> 27` with the release at BCI 17–21 (normal) and 28–32 (handler). Both handlers are
`astore; <the same release groups>; aload; athrow`, and both normal paths complete by transferring
to the method's own value-less `return`.

`multiAwait` is the **zero-regression control**: refused before and after this slice with the
patrol's own recorded text (`// @bytecode 0 15 22 31 40 46 70` +
`local 1 crosses a quoted fallback region …`). The patrol's README table calls it
"完整恢复（0 引注）" while its own captured `results/jarde-ML.txt` states the refusal, and this
worktree's measurement on all three legs agrees with the captured text — so the whole-class text
stays uncompilable (the refused member has no `return`), and the compilable-and-run leg of this
fixture is `MLOrder` below.

## The order-and-exception leg

A `ReentrantLock` cannot show *which* release ran first, so the same two shapes stand over this
fixture's own recording lock (`Order.java`, package-private, `implements Lock`), and
`MLOrderDriver.java` (a fixture source, compiled per leg by the test, never recovered) reads the
log:

* `MLOrder.nestedLocks()` — the anchor's two-lock guard over the recorder;
* `MLOrder.nestedLocksThrowing()` — the same guard with a body statement (`this.check()`) that
  throws **inside** the protected range when the driver arms it;
* `MLOrder.interruptibly()` — the anchor's interruptible guard; the driver can arm the recorder so
  `lockInterruptibly` itself throws, which is the one path the row's own range states.

`MLOrder`'s every member is recovered, so its whole-class text compiles on both compilers and the
driver answers the same five lines against the fixture's own class and against the recompiled
text — the release **order** (`b` before `a`), the `finally` on the exceptional path, **no**
release after the interrupted acquisition, the interruptible completion, and the anchor's own `2`:

```
normal count=1 log=a.lock b.lock b.unlock a.unlock
caught=body failed count=1 log=a.lock b.lock b.unlock a.unlock
interrupted=a interrupted count=0 log=
interruptible count=1 log=a.lockInterruptibly a.unlock
anchor=2
```

## The negatives

`MLNegatives.java` breaks one link of the certificate's proof per member, and every member is
**verifier-valid** (compiled from this source, so a refusal is evidence about the proof rather than
about damaged bytes):

| member | the link it breaks | the refusal it keeps |
| --- | --- | --- |
| `releaseOrderNotReversed()V` | the two releases run in the **acquisition** order (`a` then `b`), not its reverse | `jre_guard_finally_copy` at BCI 41 + the quote `[58, 41]` |
| `unlockWithoutLock()V` | the second release's receiver is a third lock the statement never acquired | the same refusal: no acquisition took that object |
| `throwingCallInsideRange()V` | the acquisition lies **inside** the row (`[0,17) -> 27` covers the `lockInterruptibly` call at BCI 4) | `jre_guard_finally_copy` at BCI 27 + the quote `[37, 27]` |

## The registered boundaries

`MLProbe.java` holds the shapes one link beyond this change's own admission; each is measured and
pinned, not assumed:

| member | the boundary | the refusal it keeps |
| --- | --- | --- |
| `nestedTry()V` | a `try`/`finally` **inside** the guarded range: two rows (`[14,24) -> 34`, `[7,44) -> 54`) over two handlers | `jre_guard_finally_copy` at BCI 54 + `[44, 34, 54]` |
| `threeLocks()V` | three acquisitions and three releases: past the clause's two-statement bound | `jre_guard_finally_copy` at BCI 55 + `[79, 55]` |

## Reproduce and verify the checked-in bytes

```sh
cd tests/fixtures/recover-nested-lock-finally-bodies
javac --release 8 -g:none -d v8 ML.java MLOrder.java Order.java MLNegatives.java MLProbe.java
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac \
    -g:none -d v8-javac8 ML.java MLOrder.java Order.java MLNegatives.java MLProbe.java
javac --release 8 -cp v8 -d v8 MLOrderDriver.java
javac --release 8 -cp v8-javac8 -d v8-javac8 MLOrderDriver.java
shasum -a 256 v8/*.class v8-javac8/*.class
java -Xverify:all -cp v8 ML               # 2  (the patrol's own recorded behavior)
java -Xverify:all -cp v8 MLOrderDriver    # the five lines above
```

The class-file versions are 52.0. Both legs answer the same lines under `java -Xverify:all`.

| leg | class | bytes | SHA-256 |
| --- | --- | --- | --- |
| `v8` | `ML.class` | 1478 | `a3ac2582f0b9ecff1105097b2b60f3a7ac4a342b78cfa3bfcd94ea5f461e8fad` |
| `v8` | `MLOrder.class` | 1053 | `971697bcc2d34bebcaf465ede872decec993695d720651a383d6216bef5d7660` |
| `v8` | `Order.class` | 1473 | `e2f8c33403c24db4933624abe4bd10bf0acd629234a99ffd9b1708dca4c1d5f0` |
| `v8` | `MLNegatives.class` | 891 | `e0ec5b7e05302a87d46a7bbc96613d576d3681c206c79461aaec421774af38c8` |
| `v8` | `MLProbe.class` | 706 | `d8797854f76fde8ba33b105f74d26834d2ac3259bb3de4b9afbea2dd61309108` |
| `v8` | `MLOrderDriver.class` | 1869 | `516e76f726440aaac8b90a9d582e95edbf510b3c2d5988be2e5e6361348aa052` |
| `v8-javac8` | `ML.class` | 1481 | `e68f292ed74063d2ab015e3fe00393e443b6075c54984e94eed5165911aa5007` |
| `v8-javac8` | `MLOrder.class` | 1053 | `bcac2e592c00bda62aa35c3d50c632a1b1b2daa37a007056989abd4aa07eb6f8` |
| `v8-javac8` | `Order.class` | 1473 | `accdc999e76b9eb915bdc4b73af3f12ed94bf9f68d40a1340481f77127c389ba` |
| `v8-javac8` | `MLNegatives.class` | 891 | `0926727e3304a0f8ea3f8859f3cad0ddf839f652c49c0dd03943a6c6303bc6f0` |
| `v8-javac8` | `MLProbe.class` | 706 | `91568c39c1c8d946805c6769df3f3d79ad8c7b00ba0e27d8da156a98a658d1b1` |
| `v8-javac8` | `MLOrderDriver.class` | 1869 | `516e76f726440aaac8b90a9d582e95edbf510b3c2d5988be2e5e6361348aa052` |

The same bytes are recorded in `tests/fixtures/corpus-fingerprint.json` (blake3), which the P5
fingerprint test verifies.

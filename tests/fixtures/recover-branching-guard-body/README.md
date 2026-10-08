# Branching guard body fixture (`recover-branching-guard-body`)

The nested-lock slice's third measured boundary, now presented: a lock guard whose **protected body
branches**. Every class here is compiled twice — `v8/` with javac 23.0.1 `--release 8 -g:none`, and
`v8-javac8/` with the real javac 8 (Corretto 1.8.0_432) `-g:none` — and neither leg has a
`LocalVariableTable` or a `LineNumberTable`, so every name the recovery layer writes is a derived
`localN` name and every shape decision is a decision about control flow, not about debug metadata.

`tests/recover_branching_guard_body.rs` reads the committed bytes; this README is the fixture's own
contract (the sources, the shapes, the commands and the recorded behavior).

## The shape, and why it refused

`BG.java` is the anchor. `singleIf` is the patrol's own boundary shape verbatim
(`MLProbe.nestedLocksBranching`, refused at BCI 55 before this slice) and `ifElse` is the same guard
with both arms written:

```java
void singleIf(boolean fail) {
    this.a.lock();
    this.b.lock();
    try {
        this.count++;
        if (fail) {
            throw new IllegalStateException("body failed");
        }
    } finally {
        this.b.unlock();
        this.a.unlock();
    }
}
```

The bytecode (this leg, `javap`): the one catch-all row is `[14,38) -> 55`; the two releases stand at
BCI 38–49 (normal) and 56–67 (handler); the normal path ends `52: goto 72`. Measured facts
(instrumented `guard::prove_lock_guard_finally`, both legs, `results/01-refusal-and-reading.md`):

* the transfer at BCI 52 sits in a canonical block that **starts at 38** — the protected range's own
  end — and holds `[38, 39, 42, 45, 46, 49, 52, 72]`: the release copy, the transfer, and the
  method's own trailing `return` at 72;
* that block carries **no exception edge of its own** (the range stops where it begins), so the
  canonical graph fuses the single-successor/single-predecessor chain and the void completion's
  transfer has **no successor block to state** — which is where the certificate refused.

The control is the same behavior **without** that layout: the nested-lock fixture's
`MLOrder.nestedLocksThrowing` (a straight body whose release copy stays in the protected call's
block, so the range's exception edge keeps the trailing return a block of its own) presents before
and after this slice. `LK.put`/`take` are the same pair on the loop side: their release copy's block
still holds protected instructions, so it carries the row's exception edge, the return stays its own
block, and the certificate's existing `Void` form reads it (`successors=[66]`, measured). The two
together are the measured line.

## The anchor's own text

Both members present whole and **compile**, on both compilers, and answer what the class answers
under `-Xverify:all` (the ignored driver leg below):

```java
    void singleIf(boolean arg1) {
        this.a.lock();
        this.b.lock();
        try {
            this.count += 1;
            if (arg1) {
                throw new java.lang.IllegalStateException("body failed");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        return;
    }
```

The `return;` after the `try` is the fused tail this slice reads: the builder writes the tail's span
after the statement, exactly where the bytecode runs it (`52: goto 72`).

## The order-and-exception leg

A `ReentrantLock` cannot show *which* release ran first, so the anchor's shapes stand over this
fixture's own recording lock (`Order.java`, package-private, `implements Lock`) in `BGOrder.java`,
and `BGOrderDriver.java` (a fixture source, compiled per leg by the test, never recovered) reads the
log. `singleIf` is armed to throw **inside** the protected range; `ifElse` runs both arms; `twoIfs`
runs all four combinations of its two branches. The class is recovered whole, so the driver answers
the same eight lines against the fixture's own class and against the recompiled presented text:

```
normal count=1 log=a.lock b.lock b.unlock a.unlock
caught=body failed count=1 log=a.lock b.lock b.unlock a.unlock
ifElse-taken count=1 log=a.lock b.lock b.unlock a.unlock
ifElse-not-taken count=-1 log=a.lock b.lock b.unlock a.unlock
twoIfs-00 count=1 log=a.lock b.lock b.unlock a.unlock
twoIfs-01 count=0 log=a.lock b.lock b.unlock a.unlock
twoIfs-10 count=2 log=a.lock b.lock b.unlock a.unlock
twoIfs-11 count=1 log=a.lock b.lock b.unlock a.unlock
```

The release order (`b` before `a`) is the nested-lock invariant, and the `caught=` line is the
branch's throwing arm: the exception propagates with its own message and the `finally` still runs
both releases in that order.

## The negatives

`BGNegatives.java` breaks one link of the completion's proof per member; every member is
**verifier-valid**, so a refusal is evidence about the proof rather than about damaged bytes. The
refusals of the first three are **byte-identical** to the ones the same bytes produced before this
slice (the pre-change text is recorded in `results/03-anchors-and-negatives.md`):

| member | the link it breaks | the refusal it keeps |
| --- | --- | --- |
| `tailThrow()V` | the fused tail is `throw new IllegalStateException("after")` — an allocation and an `athrow`, not a run of statements the reading states | `jre_guard_finally_copy` at BCI 58 + the quote `[31, 41, 58]` |
| `tailStoredThrow()V` | the tail is `throw this.stored` (`aload_0; getfield; athrow`): straight instructions, but the method's **throw**, not the value-less return the certificate proves | the same refusal at BCI 58, its own quote |
| `bodyReturn(Z)V` | the branch's arm **returns from the method** inside the protected range (which would skip the release the source's `finally` runs) | `jre_guard_finally_copy` at BCI 60 + `[28, 43, 60]` |
| `switchBody(I)V` | the body holds a **multi-way branch** the walk cannot state | refused whole; the diagnostic **moves** (below) |

`switchBody` is the one classified delta: the certificate now claims the statement (the fused tail is
admitted) and the body walk refuses the `switch`, so the emitted text is the guard's own body refusal
— `// @bytecode 0 1 4 … 101` + `// the shared catch-all finally has no complete bounded try and
catch bodies` — instead of the pre-change four-piece cascade (`// BCI 84: …` + `// 4 live block(s) …
[44, 57, 84, 67]`). The member stays refused whole with no `finally` invented; only the diagnostic
moved, which is what `results/03-anchors-and-negatives.md` records.

## The probe: the measured line's far side

`BGProbe.java` pins what the walk does **beyond** the filing's own MVP note ("single branch level;
multi-branch stays registered"). Measured, not assumed: `twoIfs` (two branches in sequence),
`nestedIf` (a branch inside a branch) and `bodyLoop` (a loop in the protected body) all **present
whole** — the region walk already structures them, and refusing them would need a branch-count rule
no requirement states. The filing's note is narrower than the admission; the spec's own sentence
("when the lock guard's protected body contains a branch … present the whole guard") is not. The
multi-way branch stays refused (`BGNegatives.switchBody`).

## Reproduce and verify the checked-in bytes

```sh
cd tests/fixtures/recover-branching-guard-body
javac --release 8 -g:none -d v8 BG.java BGOrder.java Order.java BGNegatives.java BGProbe.java
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac \
    -g:none -d v8-javac8 BG.java BGOrder.java Order.java BGNegatives.java BGProbe.java
javac --release 8 -cp v8 -d v8 BGOrderDriver.java
javac --release 8 -cp v8-javac8 -d v8-javac8 BGOrderDriver.java
shasum -a 256 v8/*.class v8-javac8/*.class
java -Xverify:all -cp v8 BGOrderDriver    # the eight lines above
```

The class-file versions are 52.0. Both legs answer the same lines under `java -Xverify:all`.

| leg | class | bytes | SHA-256 |
| --- | --- | --- | --- |
| `v8` | `BG.class` | 752 | `8b9fb6bf9480cb4f01d2634717acc09c03d7fe8fe29e11674e2592afa00d6df8` |
| `v8` | `BGOrder.class` | 962 | `6775c11a8e06f39315d3839c4f46e5436db3c8015d2958b5d3b55f332e6dc9df` |
| `v8` | `Order.class` | 1367 | `307721ba961c31abff803bd5d18f52d01eaa0d795f89956e427510904dfc9181` |
| `v8` | `BGNegatives.class` | 1221 | `35f079f82f79dbed104501d5cf280445a4d99f371c38da3a2635784ce307aecf` |
| `v8` | `BGProbe.class` | 848 | `921927ad25bb7766e04134fd6c5bb2a7687b5e40bceb389b6066d16789bdae89` |
| `v8` | `BGOrderDriver.class` | 1822 | `92441b125b670816dd928f25b72c1100255ca7f73eefc46eb6c37887a2b898ae` |
| `v8-javac8` | `BG.class` | 752 | `951c0ddedaff8dc47d3f0519ead9f7a4f124d19158601d8adc165a51023d3ed0` |
| `v8-javac8` | `BGOrder.class` | 962 | `dbe56122f09b3724ef9c472ddd8197087fbd565a184f9c84c68c3a1a9d6f5ed8` |
| `v8-javac8` | `Order.class` | 1367 | `bd85a7faea76a2e586595ba9254e8dee81654bdffe6104b53c51e9f1116bd598` |
| `v8-javac8` | `BGNegatives.class` | 1221 | `591e6a624ec98717f842d3b4d6b7a7e4c5d26d258cfcd2850587a2955f5d90d7` |
| `v8-javac8` | `BGProbe.class` | 848 | `291e4e5d7bed5865ad4b87af526c36598039ac2cf3c2edeedc0c540035c61573` |
| `v8-javac8` | `BGOrderDriver.class` | 1822 | `92441b125b670816dd928f25b72c1100255ca7f73eefc46eb6c37887a2b898ae` |

The same bytes are recorded in `tests/fixtures/corpus-fingerprint.json` (blake3), which the P5
fingerprint test verifies.

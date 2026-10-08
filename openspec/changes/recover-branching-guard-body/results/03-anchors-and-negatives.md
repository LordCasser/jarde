# Task 1.2 / 2.2 — the frozen anchors and negatives (both legs)

Everything below is pinned in `tests/recover_branching_guard_body.rs` (this slice's fixture),
`tests/recover_nested_lock_finally_bodies.rs` (the nested-lock fixture's branching boundary, whose
assertion is updated — never deleted — to the presentation), and the fixture's own `README.md`
(sources, commands, SHAs, recorded behavior). Every text and refusal was read from the committed
bytes of both compiler legs.

## The anchor

| anchor | where | what it pins |
| --- | --- | --- |
| `MLProbe.nestedLocksBranching` (both legs, in-library) | `tests/recover_nested_lock_finally_bodies.rs` | the measured boundary — registered by that slice, refused at BCI 55 — now presents as its source wrote it. The pre-change refusal is recorded verbatim in the assertion's doc comment |
| `BG.singleIf` (both legs) | `tests/recover_branching_guard_body.rs` | the patrol's own boundary shape over this fixture's own bytes: the acquisitions, the branch, the two releases, and the fused `return;` after the `try` |
| `BG.ifElse` (both legs) | same | both arms of the branch present; the whole-class text carries no quote |
| `BGOrder.singleIf`/`ifElse`/`twoIfs` (both legs) | same + the ignored replay | the same shapes over the fixture's recording lock: the whole-class text is the one the driver roundtrip compiles, and the driver's eight lines are identical between the fixture's class and the recompiled text under `-Xverify:all` (both compilers) |

The anchor's texts (both legs, byte-identical):

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

The driver's recorded behavior (both legs, both sides):

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

The `caught=` line is the branch's throwing arm: the exception propagates with its own message and
the `finally` still runs both releases in the clause's order (`b` before `a`), which is the
nested-lock invariant — on both javac legs, under `-Xverify:all`.

## The negatives (both legs; refused whole, no `finally` invented)

| negative | the link it breaks | HEAD | admission |
| --- | --- | --- | --- |
| `tailThrow` | the fused tail is `throw new IllegalStateException("after")` — an allocation and an `athrow`, not a run of statements the reading states | refused, BCI 58 + `[31, 41, 58]` | **byte-identical** |
| `tailStoredThrow` | the tail is `throw this.stored` (`aload_0; getfield; athrow`): straight, but the method's **throw**, not the value-less return the certificate proves | refused, BCI 58 + `[31, 41, 58]` | **byte-identical** |
| `bodyReturn` | the branch's arm **returns from the method** inside the protected range | refused, BCI 60 + `[28, 43, 60]` | **byte-identical** |
| `switchBody` | the body holds a **multi-way branch** the walk cannot state | refused, BCI 84 + `// 4 live block(s) … [44, 57, 84, 67]` | refused; **diagnostic moved** (below) |

The verbatim texts of the first three are the constants `TAIL_THROW`, `TAIL_STORED_THROW` and
`BODY_RETURN` in `tests/recover_branching_guard_body.rs`; `diff` of the baseline and admission
renders shows only the `switchBody` block differs.

### The one classified delta

```
- // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25
- // BCI 84: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
- // @bytecode 44 45 46 49 50 51 54 57 58 59 62 63 64 67 68 71 74 75 78 81 84 85 86 89 92 93 96 99 100 101
- // 4 live block(s) are reachable only through edges the normal-flow view leaves out: [44, 57, 84, 67]
+ // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 44 45 46 49 50 51 54 57 58 59 62 63 64 67 68 71 74 75 78 81 84 85 86 89 92 93 96 99 100 101
+ // the shared catch-all finally has no complete bounded try and catch bodies
```

Why it moves, and why that is correct: before the slice the certificate refused the fused layout and
the walk surfaced the four-piece cascade at the handler; now the certificate **claims** the statement
(the fused tail is admitted) and the **body walk** refuses the `switch` — the same refusal the walk
gives a `switch` inside any other protected body. The member is still refused whole (no `finally`
invented, `jarde_refused_body()`), and the shape is registered, not silently admitted. It is the one
delta in this slice's own fixture that is not a presentation; the `switch` is the "multi-way branch"
the filing's note registers.

## The probe: the measured line's far side

`BGProbe` pins the shapes the walk structures **beyond** the filing's MVP note ("single branch
level; multi-branch stays registered"): `twoIfs` (two branches in sequence), `nestedIf` (a branch
inside a branch) and `bodyLoop` (a loop in the protected body) all **present whole** on both legs.
This is the one place where measurement contradicts the filing's prose: the region walk already
structures those bodies (they are the same `If`/`Loop`/`Sequence` shapes the loop family's bodies
use), so refusing them would need a branch-count rule no requirement states — the spec's own sentence
("when the lock guard's protected body contains a branch … present the whole guard") has no such
bound. The fixture's README and the test say so explicitly, so the delivered behavior is visible
rather than assumed. The far side — the multi-way branch — stays refused (`BGNegatives.switchBody`).

## The reading controls (the sibling-vs-general decision's evidence)

`reading-controls/` holds the two scratch sources and their renders on both binaries. They are the
measurement behind choosing the **sibling** reading (`01-refusal-and-reading.md`): the same
completions with a **straight** body — where the release copy stays in the protected call's block and
the return is a block of its own — refuse identically before and after (a trailing `throw this.stored`,
a trailing `throw new …`, a trailing assignment, a loop, a trailing `if`, a `switch`), so the
certificate's admitted set does not depend on the layout javac left.

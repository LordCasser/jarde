# Task 1.1 — the gating experiment

Run by `gating.sh <baseline-cli> <admission-cli> <out> [certificate-only-cli]`, which renders the
five guard-family anchor sets plus this slice's own fixture with each binary (`render-set.sh`
asserts every render's own header before it is accepted) and writes the hash matrices to
`gating.out` (two columns) and `gating-certificate-only.out` (the finer three-column one).

* **baseline** — the worktree built at HEAD (`6ca9cfdf`, the filing), i.e. with
  `crates/jarde-java/src/guard.rs` and `region.rs` at their HEAD state;
* **admission** — the same worktree with this slice's two files in place;
* **certificate-only** — `guard.rs` admitted, `region.rs` at HEAD. This column separates the two
  halves of the slice: the certificate alone moves the anchor's *diagnostic* (it claims the shape
  and the **body walk** then refuses it — `// the shared catch-all finally has no complete bounded
  try and catch bodies`), and the walk's own boundary is what turns that into the presentation. The
  10 moved inputs' renders of this column are stored under `gating-certificate-only/`.

## The matrix (48 inputs; the full table is `gating.out`)

| input group | inputs | verdict |
| --- | --- | --- |
| LK — the explicit-lock patrol's jar, the LK anchor on both legs, `LockGuardNegatives`, `LockGuardProbe` | 7 | identical |
| IO — the io-wrapping patrol's jar, the IO anchor on both legs, `IOMidRead`, `IONegatives`, `NestedDepth` | 9 | identical |
| nested-lock — the multi-lock patrol's jar, `ML`, `MLOrder`, `MLNegatives` on both legs | 7 | identical |
| nested-lock — **`MLProbe` on both legs** (the branching boundary's own class) | 2 | **MOVED** |
| loop-test-copy — `Probe`, `ProbeControls`, `MultiCopy` on both legs, `LoopTestValues` | 7 | identical |
| copy/dup-store — `REF`, `DS`, `NEG` (both legs), the three CF-06 controls | 7 | identical |
| **this slice's fixture** — `BG`, `BGOrder`, `BGNegatives`, `BGProbe` on both legs | 8 | **MOVED** |
| total | 48 | **37 identical, 10 moved** |

The 10 moved renders are exactly: the branching anchor's class on both legs (`MLProbe`) and this
slice's own fixture (its 8 renders). Nothing else in the five families moves.

## The finer column (certificate alone vs the reader's boundary)

`gating-certificate-only.out` (the 10 moved inputs' certificate-only renders are stored beside it):

| verdict | inputs | what it says |
| --- | --- | --- |
| `all-identical` | 37 | the five families' anchors, negatives and probes — the certificate's admission and the walk's boundary are both invisible to them |
| `both-moved,different-text` | 8 | `MLProbe` ×2, `BG` ×2, `BGOrder` ×2, `BGProbe` ×2 — the certificate alone claims the shape and the body walk refuses it (the anchor's diagnostic moves to the guard's own body refusal, the member still refused); the admission presents |
| `both-moved,same-text` | 2 | `BGNegatives` ×2 — the certificate refuses the negatives' tails, so the body walk is never reached and the two columns agree (`switchBody`'s moved diagnostic is the difference from the baseline) |

So the slice is two parts, both measured: **the certificate's fused-tail reading** (which claims the
shape) and **the body walk's own boundary** (which lets the branch's arm reach the range's end). The
second is one statement in `bounded_shared_finally_body` using the frame's existing `boundary`
mechanism — no new region kind, no new reader arm, no new frame field — which is why it lands in the
same commit as the first and not as a separate gated change.

## What moved inside `MLProbe` (the anchor)

`diff baseline/mlprobe-v8.java admission/mlprobe-v8.java` — one member, both legs:

```
49d48
<         // jarde: not recovered: the recovery run for `nestedLocksBranching(Z)V` produced no statement (explanation only); …
53,57c52,63
<         // @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25
<         // BCI 55: the exceptional path repeats code the normal path also runs — …
<         // @bytecode 28 31 32 34 37 38 39 42 45 46 49 52 55 56 57 60 63 64 67 70 71 72
<         // 3 live block(s) are reachable only through edges the normal-flow view leaves out: [28, 38, 55]
<         jarde_refused_body();
---
>         this.a.lock();
>         this.b.lock();
>         try {
>             this.count += 1;
>             if (arg1) {
>                 throw new java.lang.IllegalStateException("body failed");
>             }
>         } finally {
>             this.b.unlock();
>             this.a.unlock();
>         }
>         return;
```

`MLProbe`'s other two boundaries — `nestedTry` and `threeLocks` — are byte-identical, so the
admission flips the branching anchor and neither of its siblings.

## The fixture's own deltas

`BG` (both members present), `BGOrder` (all three present), `BGProbe` (all three present),
`BGNegatives` (three refusals byte-identical, `switchBody`'s diagnostic moved — the one classified
delta, recorded in `03-anchors-and-negatives.md`).

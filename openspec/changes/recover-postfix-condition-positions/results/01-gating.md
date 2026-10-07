# Task 1.1 — the refusal chain, located per shape, and the two gating experiments

The change's own discipline asks two questions separately: **where** the local-crossing refusal
chains for each of the three shapes, and **which** of the two candidate changes flips the anchors —
the snapshot consumer generalized to control-flow tests, or the crossing gate's paired exemption.
Both were answered before anything was implemented, with an instrumented binary. The instrumentation
is archived as `01-probes.patch` (it is not part of the change): it prints one `PROBE…` line at each
candidate refusal point and at each snapshot claim, so a transcript says *which* check fired rather
than what the final message was.

## Q1 — the chain, per shape

Instrumented run over the patrol's frozen `cp7.jar` (`PROBE` lines on stderr, the presentation
discarded):

```
$ ./target/debug/jarde-cli class-source --input …/cp7.jar --class CP7 --policy plain-jar --format text
PROBE-SUFFIX non-condition-suffix block=4 first_condition=4 at=Some(10)   ← scan: the `iinc` at BCI 10
PROBE-TEST-IMPURE block=4 test=14 at=6 op=Some(ArrayLoad)                 ← scan: `last = xs[i]` at BCI 6
PROBE-SUFFIX non-condition-suffix block=2 first_condition=0 at=Some(4)    ← find: `latch_test_chain` tried first
PROBE-TEST-IMPURE block=8 test=15 at=10 op=Some(Increment { slot: 2, amount: 1 })  ← find: the `iinc` at BCI 10
PROBE-CLAIM slot=1 load=3 update=4 consumer=7 block=0                     ← cond: the snapshot IS claimed
PROBE-TWO-EXIT-REFUSED method-region=0 reason=test BCI 8 contains an independent effect or escaped producer
```

1. **`scan` (do-while scan, `do { last = xs[i]; } while (xs[i++] != 0 && i < xs.length);`)** — the
   first refusal is inside `latch_test_chain`'s `latch_test_suffix_is_effect_free`: the suffix from
   the condition's first value (BCI 8) holds the `iinc` at BCI 10, which is not in the condition's
   value set. The chain rule declines silently, and the *reported* refusal is the next rule's —
   `header_tested_loop`'s `test_is_pure` on the header block at BCI 6 (the body's own `iaload`,
   because the block holds both the body statement and the first test). Both are the **region
   layer**; the method-level text then comes from the **declaration plan**'s
   `local 1 crosses a quoted fallback region` (the local's uses span the quoted loop region).
   The snapshot proof never runs for it: the method-level declaration refusal quotes the body
   before `prove_snapshots` is reached.
2. **`find` (while compound, `while (i < xs.length && xs[i++] != t) { }`)** — `header_test_chain` →
   `proved_header_test` → `test_is_pure` on the *second* test block (BCI 8) refuses at the `iinc`
   (BCI 10): the region layer again, then the same declaration-plan refusal.
3. **`cond` (if short-circuit, `if (a[i++] > 0 && i < a.length)`)** — the region walk claims the
   `TwoExitReturn` diamond; the **snapshot proof already claims the consumer** (`slot=1 load=3
   update=4 consumer=7`, the `iaload` inside the first test), and `build_two_exit_return` refuses at
   its own dependency check: the outer test block holds the local's initialization (`int i = 0;`,
   BCIs 0–1), which is not part of the test's value expression — a gap that exists **without any
   postfix** (`int i = 0; if (a[i] > 0 && i < a.length) return true; return false;` refuses the same
   way at the parent commit).

So: **two of the three shapes refuse in the region layer's test-purity gate (the crossing gate's
source), and the third refuses in the two-exit-return builder's own test-block proof — the snapshot
consumer itself is not the refusal point of any of them.**

## Gating experiment A — the consumer side only (crossing gate untouched)

The snapshot consumer proof (`prove_local_snapshots`/`snapshot_consumer`) already admits a consumer
inside a test block: the proof's interval and single-use checks are value-level, and `cond`'s
`PROBE-CLAIM` above is that claim at the parent commit. Two further facts, measured with the parent
commit's own binary (no change applied):

```
$ /tmp/pcp-base-target/debug/jarde-cli class-source … --class U   # parameters, no local init
    static boolean j1(int[] arg0, int[] arg1, int arg2, int arg3) {
        return arg0[arg2++] > 0 ? arg1[arg3++] > 0 ? true : false : false;   ← two positions in one condition
    static boolean j2(int[] arg0, int arg1, int arg2, int arg3) {
        return arg3 != 0 ? arg0[arg1++] > 0 ? arg1 < arg2 ? true : false : false : false;  ← middle of a chain
$ …/jarde-cli … --class Q      # o2: `while (i++ < n)`
    static int o2(int arg0) { … while (local1++ < arg0) { } return local1; }  ← the branch itself as consumer
```

The consumer side therefore needed **no generalization** for these positions; what the `if` shape
needed was the outer test block's **lead** (`int i = 0;` sharing the block with the `if`), which is
a pre-existing gap independent of the postfix:

```
experiment A1 (the lead + `test_effects(outer)` only, region gate untouched):
  cond        → int local1 = 0;
                return arg0[local1++] > 0 ? local1 < arg0.length ? true : false : false;   ← FLIPS
  p1 (`int i = 0; if (a[i] > 0 && i < a.length) …`)  → int local1 = 0; return arg0[local1] > 0 ? …  ← FLIPS (no postfix)
  scan        → still refused (local 1 crosses a quoted fallback region)                        ← does NOT flip
  find        → still refused (local 2 crosses a quoted fallback region)                        ← does NOT flip
```

A dependency-walk arm for the **updated** value (the second test reading the incremented slot) was
written, gated, and then **removed**: it is not needed. The walk stops at the `iload`, and a slot
value is not followed through a load, so the escaped-producer check never sees the `iinc`; `cond`
flips on A1 alone.

## Gating experiment B — the crossing gate's paired exemption (region layer)

The exemption is the one the `unobservable_store_dance_part` precedent states: both halves of the
pair, admitted together, only when the pair really is a postfix position. Two additions were needed,
both in the region layer's test-expression rule:

* the `iload slot; iinc slot, ±1` pair, when the load's old value has exactly one consumer, in the
  same block, after the update, and that consumer is part of the condition (or the branch itself);
* the **array element read** the test consumes (`xs[i++]`), under the same single-reader discipline
  the array *length* already had — without it the trio's own conditions are refused even with no
  postfix in them (`do { last = xs[i]; } while (xs[i] != 0 && i < xs.length)` refuses at the parent
  commit for the array read alone).

```
experiment B (A1 + the region exemption):
  scan  → do { local2 = arg0[local1]; } while (arg0[local1++] != 0 && local1 < arg0.length);   ← FLIPS
  find  → while (local2 < arg0.length && arg0[local2++] != arg1) { }                           ← FLIPS
  cond  → (as in A1)                                                                           ← stays flipped
```

`find` needed one more thing on the *build* side: its first test reads the **pre-update** value in
another block, and Phase A's multi-reader check refused any cross-block read. The bounded form added
admits exactly the read whose terminal consumer's block transfers directly to the increment's block —
the earlier test of the same loop's condition chain — and nothing else:

```
find at the moment the region gate alone was widened (no build-side form yet):
  // the value at BCI 15 is the value local 2 held at BCI 9, and the slot does not hold it at BCI 15:
  //   the slot's name would read the value the body wrote in between      ← the time annotation, fail-closed
with the bounded cross-block form:  while (local2 < arg0.length && arg0[local2++] != arg1) { }   ← FLIPS
```

## The negatives, at the same gate

The exemption as stated admits the deferred shapes too (measured, both sound: `while (arg0[local2++]
!= 0 && arg1[local3++] != 0)` and `while (local2 != 0 && arg0[local1++] != 0 && local1 < arg0.length)`
rendered whole). The slice's own bound — one position, at one end of a chain — is therefore stated in
the chain rules themselves (`FallbackReason::ChainPositionBound`, `jre_region_chain_position_bound`),
and the two deferred shapes keep the refusal they had:

```
CN.twoVariables  → jre_region_chain_position_bound: the chain at block BCI 4 would present the postfix
                   condition position at BCI 14 outside the bound this slice states: one position, at
                   one end of a condition chain
CN.midChain      → … the postfix condition position at BCI 9 … outside the bound …
A-phase NG (the four traps) → byte-identical refusals (the dup-store `NEG` class byte-identical too)
```

The bound is stated for the **loop chain rules** (`header_test_chain`, `latch_test_chain`). The
two-exit-return builder keeps its parent-commit behavior for these shapes: with *parameter* targets
(no lead) the parent commit already presents `arg0[arg2++] > 0 ? arg1[arg3++] > 0 ? …` and
`arg3 != 0 ? arg0[arg1++] > 0 ? …` whole, so fencing them there would have refused shapes that
recovered before the change. With *local* targets they move with A1 — reported as outgrowths in
`03-corpus-delta.md`, never silently absorbed.

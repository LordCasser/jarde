# Task 2.1 — the implementation, and what it deliberately does not touch

## The change

One arm in the consumer whitelist of `proves_boolean_local_store`
(`crates/jarde-java/src/build.rs`), twelve lines including the comment:

```rust
            // A branch that tests the loaded value reads it at its condition position: javac lowers
            // `b ? x : y`, the statement `if (b)` and the loop `while (b)` to an `ifeq`/`ifne` on
            // the local's own load. The load keeps the position it already has — the branch is its
            // consumer, so nothing is reordered — and the arms it selects between are the existing
            // conditional-value and statement presentations'. The two zero-tests are named by
            // identity, so a numeric branch (`iflt`, `ifgt`) on the same load states no boolean
            // position and keeps its refusal.
            (0x99 | 0x9a, Some(Operation::Comparison { op, .. }))
                if matches!(op, CompareOp::JumpIfZero | CompareOp::JumpIfNotZero) =>
            {
                true
            }
```

`git diff crates/jarde-java/src/build.rs` is exactly this hunk and nothing else: every other
criterion of the gate — the single write, the declaration region, the one region path, the SSA
identity of every read, the trivial-merge tolerance, the budget charges — is byte-identical to
HEAD. The two opcodes are matched **and** the decoded sense is named by identity, so a numeric
branch on the same load (`iflt`, `ifgt`, `ifle`, `ifge`, `if_icmp*`) states no Boolean position.

## Why it is sound

* **The load does not move.** The gate reads a *consumer* of the local's own load; the load's
  bytecode position is unchanged, and the branch is that load's consumer. Nothing is hoisted,
  reordered or duplicated, so no evaluation order changes.
* **The value is the exact 1/0 graph.** The arm is reached only from the two existing callers —
  a short-circuit region's own `ShortCircuitConsumer::Local` store and the ordinary one-test
  fold — whose proofs (unique `istore`, exact `1`/`0` producers, one Phi) already hold. The arm
  adds no new evidence about the value; it states where the value may be read.
* **The arms are the existing presentations'.** The ternary's two materialisations are the
  conditional-value channels, the statement's are the `If` presentation. `OP2`'s recovered
  `return local1 ? arg0 : -1;` is the measurement that this needed nothing else: the arms are
  `iload_0` and `iconst_m1`, presented at their own positions.
* **A refused read stays refused.** The whitelist is conjunctive over every read of the local:
  one read outside the admitted positions refuses the whole local, exactly as before.

## What the arm reaches, measured rather than assumed

| shape | read's block | outcome |
| --- | --- | --- |
| `b ? x : y`, `if (b)`, `a && b && c`'s middle | the store's own canonical block | admitted; presented at the condition position |
| `while (b)` | the loop's header block, a region of its own | refused **before** the whitelist, by the gate's cross-region criterion |
| chain stored inside a `catch`, read outside the `try` | — | refused **before** the gate, at region ownership |
| `iflt` on the same load | the store's own block | reaches the whitelist and is refused there |

The first three rows are `results/01-refusal-chain.md`'s transcript. The `while (b)` row is the
one立案 assumption the measurement corrected: the loop condition position is *not* delivered by
this arm, and delivering it would require widening the cross-region criterion this change's hard
invariant 2 keeps verbatim.

# Task 2.1/2.2 — what the implementation is, and the boundaries it keeps

Two commits, gated apart (the matrix is in `01-instrumentation-and-gating.md`):

* `feat(java): present the resource guard's continuation completion` — the guard certificate's
  Transfer completion (`crates/jarde-java/src/guard.rs`, `region.rs`'s save match, `build.rs`'s
  guard arm);
* `feat(java): present the loop test's copy-and-store assignment` — the loop-test position's
  admission and presentation (`region.rs::store_dance_part`, `build.rs::prove_local_assignments`).

## The continuation completion (the guard-body half)

`prove_resource_guard_finally` read the normal path's completion and refused the one javac writes
when the protected body does not return: the release copy is followed by a `goto` that carries the
body's completion past the close into the statement's own continuation. The claim now proves that
continuation and stores it in the plan as `LockGuardCompletion::Continues { transfer, continuation }`:

* the transfer's block has exactly one successor, reached from nowhere else
  (`predecessors == [the transfer's block]`) → `Continuation::Block`, and the plan's `join` is that
  block, so the region walk continues there;
* the transfer's target was **fused** into the transfer's own block — the canonical graph fuses a
  single-successor/single-predecessor chain, and `readAll`'s `goto 53` is exactly that, because the
  normal-cleanup block carries no exception edge of its own (the protected range ends before it) —
  → `Continuation::Tail { span }`, the block's own tail after the transfer. The claim accepts a
  fused tail only where every instruction but the last is a straight one (`Push`/`Load`/`Store`/
  `Arithmetic`/`Negate`/`PrimitiveConversion`/`Invoke`/`Field`/`Transfer`) and the last completes
  the run (`Return`/`Throw`), so the tail is a statement sequence the Java writer can state. The
  plan's `join` is `None` — the run ends where the block does — and the builder writes the tail
  after the `try` statement with `range(span)`, which is where the bytecode runs it.

A plan whose `join` is a block the statement itself owns is refused (`owned.contains(join)`), so a
continuation can never be written twice.

## The loop-test position (the copy-family half)

1. `region::test_is_pure`'s per-instruction rule (`test_expression_instruction`) admits the
   `dup; store` pair whose stored value the test consumes (`store_dance_part`): the copy produces
   two values, the store takes one into a local slot **the body declares** and the test reads the
   other. A parameter target (the slot the method's entry state defines), a copy whose values
   anything else consumes, and a store no test reads keep the `StatementFree` refusal at their own
   BCI. The block's other effects are unchanged: a call or field read the branch consumes still
   stays in the condition, and anything else still refuses the loop.
2. `build::prove_local_assignments` presents a **local** target at a loop's own test as the
   in-place assignment expression (`LocalAssignmentPresentation::Expression`) — the presentation
   the copy family's assignment rule already writes at a short-circuit step's position. The
   renderer's own arm writes it there: `render_value`'s `Duplicate` arm builds the
   `LocalAssign` node at the store's BCI with the copy as its derived origin, and the store
   instruction is suppressed, so the loop's condition text is `(c = read())`.
3. The `observed && enters_handler` rule now refuses only presentations that **move** the
   assignment. The eliminated form drops the store and the in-place expression writes it where the
   bytecode ran it — a loop re-evaluates the test exactly as the bytecode did — so the rule is
   relaxed for the loop-test position alone; the split form (written in front of the structure) and
   every other position keep it.

## The boundaries this keeps

* **A parameter target at a loop test** (`LoopTestValues.storeTest`, `ProbeControls.parameterTarget`)
  keeps the region layer's refusal: a parameter's declaration is the signature, and the in-place
  expression is the copy family's *local* form. This is what keeps `p3_loop_test_values`' pinned
  negative byte-identical.
* **A copy with more consumers than the store and the test** (`MultiCopy`) keeps its refusal: the
  admission states the two-consumer identity, not "any store in a test".
* **A dance at an `if` position inside a protected range** (`ProbeControls.guardIfFirst`) keeps the
  movement rule's refusal — demonstrated by the gating check: with the rule disabled the same class
  presents, so the refusal is the rule's and not the guard body's. Its control
  (`Probe.guardPlain`, the same guard and `if` with no dance) presents.
* **The io anchor's other members** — `countLines`, `main`, `IOMidRead`, `IONegatives`,
  `NestedDepth` — and the LK family are byte-identical to HEAD in every column of the matrix.

## The two presentations the loop-test position writes

`IO.readAll` (the io slice's registered boundary, now recovered), from the patrol's own jar:

```java
    static java.lang.String readAll(java.lang.String arg0) throws java.io.IOException {
        java.io.FileReader local1 = new java.io.FileReader(arg0);
        java.lang.StringBuilder local2 = new java.lang.StringBuilder();
        try {
            int local3;
            while ((local3 = local1.read()) != -1) {
                local2.append((char) local3);
            }
        } finally {
            local1.close();
        }
        return local2.toString();
    }
```

`Probe.readAll` (this slice's own probe, the same form outside any guard body):

```java
    static java.lang.String readAll(java.lang.String arg0) throws java.io.IOException {
        java.io.FileReader local1;
        java.lang.StringBuilder local2;
        local1 = new java.io.FileReader(arg0);
        local2 = new java.lang.StringBuilder();
        int local3;
        while ((local3 = local1.read()) != -1) {
            local2.append((char) local3);
        }
        local1.close();
        return local2.toString();
    }
```

The loop-local's declaration (`int local3;`) is the declaration plan's own placement: the variable's
uses are the store in the loop's test block and the read in the loop's body, so the loop's region is
its owner and `prove_local_assignments`' elevation writes the declaration at that region's start —
before the `while`, inside the guard body where the uses are. The resource local and the accumulator
(`local1`/`local2`) are the guard certificate's lead elevation, unchanged from `countLines`.

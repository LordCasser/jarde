# 2.1/2.2 — what changed

One file: `crates/jarde-java/src/build.rs` (143 insertions, 5 deletions). No new state, no new
protocol, no second proof: the array initializer's own reading is extended at the two places the
patrol's two refusals were located (see [01-gating-refusal-point.md](01-gating-refusal-point.md)).

## The reader of the dance's value (`prove_array_initializer`)

Before: the value's consumer was the instruction **immediately after the last element store**
(`block.instructions()[store_pos + 1]`), and the consumer admission accepted a `return`, a local
store, an invocation, a claimed field write and the `aastore` value position.

After: `array_initializer_reader` states the reader in two steps.

1. The instruction right after the store, when it is the value's own single use — the shape every
   position that needs no operand of its own has (`astore`, `putstatic`, `invoke`, `areturn`,
   `aastore`). Unchanged, and reached first, so the four admitted positions and the child geometry
   take exactly the path they always took.
2. Otherwise the value's **own single use** in the same block, when the run between the store and it
   is that reader's other operand: every instruction of the run is a producer of one of the reader's
   operands, judged by [`collect_expression_bcis`] over the window `(store_pos, reader_pos)` and
   closed by [`dependency_uses_stay_within`] — the same two readings this initializer's own element
   values already pass through. A run that is not exactly that production (an effect of its own, a
   value a later statement reads, a value produced outside the window) leaves the initializer
   unproved, exactly as it was before the reader was looked for at all.

`cursor` (the block position the proof records as the initializer's consumer) follows the reader, so
the varargs-inline rule and the value-level refusal guard read the same position the text lands at.

`array_initializer_consumer` gains one arm: a **subscript or a length** whose *receiver* operand is
the value (`iaload`/`laload`/…/`arraylength`). Only the receiver position: an element store whose
*array* operand is the value stays the child geometry, and a store into an array the same expression
built is no Java assignment target.

## The commit walk (`ArrayInitializers::prove`)

Before: every candidate whose consumer is an array store was skipped — "a child whose sole consumer
is an array store cannot stand alone" — so `partial[0] = new int[]{7}` (an element store into an
array the body already had) was never committed, its `dup` had no proof, and the copy family refused
it.

After: the skip is the **child** geometry only: `store_writes_into_a_constructed_array` asks whether
the store's *array* operand is a value this block's own `dup` produced (the parent's copy) — or the
creation itself, for a hand-built store. Every javac-emitted child store has the former (measured:
`01-gating-refusal-point.md`), and the anchor has neither (its array operand is a field read). A
candidate that is no child commits through the ordinary walk, which is what the proof above already
admitted.

## What was deliberately not touched

* `array_initializer_consumer`'s existing arms, the proof's element-value reading, the child
  detection (`child_values`), the handler/interval checks and the `owned`/`element_sources` sets;
* the statement walk's ownership predicates (`array_initializers.owns`, `controls_binding`) and the
  quote walk (`quoted_bcis`/`deferred_producers`) — a refused dance is still named by its own quotes,
  which the hand-built `discarded` negative asserts;
* the postfix slice's absorbed-instruction awareness and its accounting check
  (`Builder::build`'s `snapshots_rendered`/`collect_quoted_bcis` guard) and the six value-level
  refusal families: no line of either moved, and the newly admitted shapes that refuse take the same
  path (`the copy at BCI N has no proved local assignment` is still in the family list).

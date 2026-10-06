# Task 1.1 — the two refusal paths, located and gated apart

The anchor matrix requires both refusal paths to be located before anything is implemented, because
the CM/CM2 local shapes and the AD/GA field-dance shape refuse through **different emitters**. They
do — and the field-dance anchor turned out to need a third one. This is the transcription.

## Path A — the time annotation (`render_value`'s `Operation::Load` arm)

`crates/jarde-java/src/build.rs`, the load arm of `render_value`:

```
"the value at BCI {at} is the value local {slot} held at BCI {bci}, and the slot does not hold it
 at BCI {at}: the slot's name would read the value the body wrote in between"
```

It is emitted when `slot_name_denotes_the_same_value(slot, read, at)` answers **no**: the slot's
last write before the use point is not the value the load read. `iload slot; iinc slot, +1` is the
smallest disagreement (the load reads the slot, the increment writes it, and the consumer reads the
*loaded* value), which is why the local shapes of the CM/CM2 patrol land here and nowhere else.

The statement that reaches it is the store (`Operation::Store` → `render_value(stored, at, 0)`), the
array store's index and value operands (`array_write`), the arithmetic a `return` reads, and the
branch's condition. In CM2.immUse the render records it twice: the store at BCI 6 refuses with the
temporal line, and the `return` that reads `local1` refuses afterwards with
`the statement at BCI 8 reads `local1`, and no statement of this body declared that local` — the
cascade the patrol recorded, which is why the *whole method* had to be re-presented and not only
the load.

## Path B — the dependency chain and the saved declaration

Two emitters, one mechanism. `prepare_deferred_bindings` tries to materialize a single-use producer
before an independent effect; the AD dance refuses it twice:

* `the dependency chain from BCI {anchor} to final consumer {reader} is not bounded` — emitted when
  `has_independent_boundary` answers `None`, i.e. when the dependency walk of one of the consumer's
  operands fails. For `elems[size++] = t` the failing operand is the `dup_x1`'s bottom copy: the
  walk stops at `Operation::Other` (the `dup_x1` is not a modelled operation), so the interval
  cannot be closed.
* `the value at BCI {at} was produced by a saved declaration this run could not commit` — the
  render arm that reads `binding_refused`; the array reference the store consumes is the value the
  refused binding would have named.

The same walk is what makes `CH.sideIdx` (`CH.arr[CH.idx++] = 10`, a *static* field) bind
`int[] saved0 = CH.arr;` instead: there the copy is a `dup` (a modelled operation), so the walk
closes, the boundary is found, and the value is saved rather than refused.

## Path C — the conditional arm (found by the AC anchor)

`conditional_dependencies` / `conditional_arm_is_expression`:

```
"the conditional arm contains an independent instruction at BCI {bci}"
```

The arm's own value tree must cover every instruction of the arm's blocks. In
`pos < src.length ? src[pos++] : null` the arm's value is the `aaload`, whose tree reaches the
`dup`'s bottom copy but not the `iconst_1`/`iadd`/`putstatic` that perform the increment — so the
arm refuses at BCI 17 even though every other gate would have passed.

## Gating: each path flips on its own change

The three paths were gated apart with the anchors themselves (each experiment is a render of one
anchor with one change in place, everything else held):

| experiment | CM/CM2 | AD/GA | AC | PT |
| --- | --- | --- | --- | --- |
| the snapshot proof alone (consumer claim + `render_value` arm + the absorbed `iinc`) | recovers | refused | refused | refused |
| + `collect_dependency_bcis`/`has_independent_boundary` reading the absorbed instructions as the expression's own | unchanged | **recovers** | refused | refused |
| + `conditional_dependencies` doing the same for the arm | unchanged | unchanged | **recovers** | **recovers** |

The local shape therefore never needed the binding path, and the field dance never needed the time
annotation — which is what the matrix asked to be established before implementing. `PT` (the static
field's postfix as an index inside a ternary arm) flips with the *same* change as AC, because it is
the same shape: AC is the patrol's own class for it, PT this change's dedicated fixture.

## What the mechanism is (and is not)

No new analysis: the old value is already an SSA value of its own (the load's output, or the copy
the dance leaves below its receiver), and the proposal's decision 1 is exactly that the
*presentation* attributes it to the postfix form. The proof added here is the consumer-position
half of the return-position proof that `recover-postfix-lvalue-values` closed: one consumer, after
the update, with nothing but the expression's own plumbing in between and no read of the updated
value inside it. The field dance reuses the identity discipline of `prove_postfix_field`
(receiver `dup` + value `dup_x1`, same member on both accesses, the receiver's evaluation an
uninterrupted single-use prefix); the local shape reuses `prove_local_postfix_element`'s identity
(the load's read and the increment's read are one value).

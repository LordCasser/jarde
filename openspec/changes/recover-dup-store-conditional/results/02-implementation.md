# Task 2.1/2.2 — the mechanism, and the two presentations it writes

The design's decision 1 is one data-flow fact — how many readers the **store target** has — and the
position of the test decides what may be done with the assignment once it is observable. This is
what the implementation turned into, with the code paths named.

## The proof (unchanged in its identity half)

`local_assignment_at` already proved the physical dance: the source immediately before the `dup`, the
store immediately after it, only `Push` operands before the terminal test, the two copies' sole uses
being the store and the test, the store's one local write, no handler in the slice. It now also
carries the two copies and the store's written value in its shape, and `prove_local_assignments`:

* collects tests from **three** region positions instead of two — `Region::If` (`Structure`, the
  structure's own test), `Region::ShortCircuitValue` (`Conditional`, a chain step) and, new here,
  `Region::Loop` (`LoopTest`, re-evaluated once per iteration);
* admits **parameter** targets, whose type comes from the member's descriptor (`parameter_types`)
  because the plan keys its decisions by the write a variable's type is decided from and a parameter
  is written by the caller;
* decides the presentation from the store target's readers.

## The reader fact

`local_store_is_observed` walks the value the store wrote: an instruction reader makes it observed,
and a merge (a phi operand record) carries it onward, so the value a later read sees through a join
or a loop header is followed to its own readers. A merge the walk cannot name is treated as a
reader — the conservative direction. `merge_index` builds the `(block, value) → merge values` index
once per method, after the cheap identity checks, and the walk is charged like the rest of the proof.

`OP2.condAssign` is the case that needs the transitive form: its store's value is an operand of the
join's phi **twice** (both arms carry it), and that phi's value is read by nothing — so the
assignment is unobservable even though the value has uses.

## The three presentations

| readers of the target | position of the test | text | who |
| --- | --- | --- | --- |
| none | any | store dropped, the consumer's expression is the value itself | **Eliminated** — `return x + 1 > 0;`, `while (read() != null)` |
| ≥1 | the structure's own test | `x = <expr>;` in front of the structure, the test reads `x` | **Split** — `x = x + 1; if (x > 0) { … }` |
| ≥1 | a short-circuit step | the assignment stays at the consumer's position as an assignment expression | **Expression** — the rule `recover-proved-local-assignments-in-conditions` established |
| ≥1 | a loop's own test | — | **refused**: the split cannot be written in front of a re-evaluated condition |

The **Eliminated** form renders `render_value`'s `Duplicate` arm at the test position: the copy's
text is the value the copy duplicated, with the copy's BCI kept as a derived origin. The store
instruction writes no statement (`instruction`'s suppression), and `slot_name_denotes_the_same_value`
gains the one exemption it needs — a store a proved dance *eliminated* has not run where the text
evaluates a read, which is exactly why `arg0` inside `arg0 + 1` still names the pre-store value.

The **Split** form is the architecture's own default path: the store instruction is *not* suppressed,
so the walk that writes the test block's effects (`test_effects`, whose comment already says "a store
or a call the test block makes is a statement of its own") writes `x = <rhs>;` in front of the
structure, and the test's own read renders as the local's name. The right-hand side is the same
value the eliminated form writes; the store's `write_statement` does the declaration and the type
check it does for every other write.

The **Expression** form is untouched.

## Why the local live target keeps the expression form

The split is sound for a local target too, and the spec's scenario 2 names the local-variable form.
It is nevertheless *not* written for one, because the shape is not new: the copy family's assignment
rule has presented a local target with a later reader as the in-place assignment expression since it
landed, and its **frozen acceptance record** pins that text —

* `openspec/evidence/java-syntax-2026-09-27/cf06-inner-assignment/replay.py` fails the run when
  `"(local1 = arg0.length()) > 5"` is absent from the rendered `lengthBranch`, and the same report
  records the rendered source's SHA-256;
* `crates/jarde-java/tests/p3_inner_assignment.rs` asserts the two texts and the source map.

Moving that text would be a lateral rewrite of another change's delivered behaviour that invalidates
its recorded acceptance, for no semantic gain. So the split is written for the target this change
*newly* admits — a **parameter** (`shape.slot < parameters`, the admission the other rule skips) —
and the local live target keeps its text byte-identical. This is reported as a deviation from the
design's decision 1, with the evidence above, rather than silently resolved.

## The loop-test landing point (Gate B)

`AC.ioLoop`'s dance sits in a loop's own test block, and the region layer refused the loop before the
builder ever saw it (`region.rs::test_is_pure`, the `loop@1` rule's `StatementFree` precondition, at
the `dup`'s BCI). `test_is_pure` now accepts two more instructions as part of the test's value
expression:

* a **copy** (`Operation::Duplicate`): it hands on the value it read and writes no effect of its own;
* a **store** whose written value has no reader at all (`self.ssa.value(*value).uses().is_empty()`):
  it is the eliminated form's store — the one write the bytecode's own program cannot tell was run.
  A store anything reads is refused at its own BCI, exactly as before.

Nothing else about the precondition moved: the loop region is claimed, and the builder still refuses
the presentation (and quotes the region) where its own proof does not hold.

## The accounting check

The statement tree owns the assignment *expression* — the existing check counts `LocalAssign`
derived origins and is unchanged, so the CF-06 path keeps its exact discipline. The eliminated and
split forms write no assignment expression, so their presentations are counted where they are
rendered (`assignment_presentations`), and the same two-sided check runs on them: a copy rendered
twice is as unaccounted for as one rendered never. The method is quoted whole if any proved copy
fails its side of the check, exactly as before.

## Zero-regression by construction

* `p3_inner_assignment` (the copy family's assignment rule): **4 passed** — its two pinned texts,
  its three refusal controls (the multi-reader `ExtraCopy`, the wrong-type patch, the interleaved
  effect, the live handler, the loop condition) and its budget pins are byte-identical.
* `p3_local_rewrite`: **7 passed**.
* `recover_postfix_old_value_snapshot`: **4 passed, 1 ignored**.
* `DS.splitLocal` in this change's own fixture pins the local live control's text as the expression
  form, so the boundary is a test, not a comment.

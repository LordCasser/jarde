# 2.1/2.2 — the implementation, and what it deliberately does not touch

## The shape

`this.ok &= x > 0` (the frozen anchor's own statement, inside a surviving enhanced `for`) lowers to

```text
20: aload_0
21: dup                    <- the receiver copy
22: getfield ok:Z          <- the read: the copy's own next instruction
25: iload 5                <- the branch's own test operand
27: ifle 34                <- the branch that cuts the block
30: iconst_1
31: goto 35
34: iconst_0
35: iand                   <- the join: the Phi's one consumer, and the write's value
36: putfield ok:Z          <- the write: in the JOIN block, not in the copy's block
```

The read's block ends in the branch; the write stands in the block the two arms hand their constant
to. The ordinary receiver-copy proof (`receiver_copy_at`, `recover-chained-field-assignment`) reads
the copy's consumers **in the copy's own block** and therefore refuses this write — that is the
refusal point `results/01-gating-refusal-point.txt` locates.

## The two admissions, and why each is load-bearing

**1. `conditional_receiver_copy_at`** (`crates/jarde-java/src/build.rs`) — a bounded certificate
entered from `FieldCopies::prove` **only** where the ordinary proof refused, for a `dup` or a
`dup_x1`. It states the region the branch and its successors form (`Region::If { prefix: [], branch:
the read's block, branch_bci, then_arm/else_arm: the branch's own two successors, join: the block
both arms meet at }`) and hands it to `prove_conditional_value` — the `recover-conditional-values`
two-arm proof — for re-verification. **No part of the two-arm proof is re-implemented here**: the
proof re-validates every edge, arm entry, Phi input and consumer against the canonical CFG, exactly
as `recover-inline-conditional-concat-operands`' own certificate reads it
(`concat.rs::verify_conditional_cut_chain`). What the certificate adds on top is what the receiver
copy needs and that proof does not state:

* each arm holds **nothing but** the constant it pushes and (where it does not fall through) the
  transfer to the join, and the constants are `0`/`1`;
* the join's Phi is an operand of the value the write takes and the read's own value is the other
  (the write's value **is** the Phi's own consumer);
* the read's value and the copy's other value are read *there and nowhere else* (`used_only_at`: one
  instruction read beside the entry-Phi records that carried them across the join's edge);
* everything between the read and the branch is the expression the branch's own test is written as
  (`collect_expression_bcis` + `interval_is_expression`), the join's run to the store is the write's
  own expression (`field_copy_expression_span` with the two cross-block leaves the certificate
  states), and nothing in the interval enters a handler.

**2. `boolean_position_values`'s `bitwise_boolean_operand`** — the boolean channel the *rendering*
needs. The Phi's expression is built by the region pass; a `0`/`1` branch value beside an operand a
descriptor proves `boolean` is that boolean's own value (`this.ok & (x > 0)`, never the ill-typed
`this.ok & (x > 0 ? 1 : 0)`), exactly as the method's `Z` return and a `Z` parameter already state
it. The position is stated by the **sibling operand's** descriptor, and Java accepts `&`/`|`/`^`
only between two booleans or two integrals — so a boolean sibling forces a boolean right-hand side.
The operation must also be the value a claimed **field write** takes: the compound assignment whose
receiver copy this change proves.

Both pieces are needed and neither is sufficient: with the certificate alone the render refuses the
bitwise expression (`boolean` and `int` operands); with the boolean position alone the copy is never
proved. The gate that measures both is `results/02-gating-experiment.txt`.

### The broader rule, measured and then narrowed

The first implementation admitted the position for **any** bitwise consumer with a
descriptor-proven boolean sibling. The corpus sweep then showed one move outside this change's
fixtures: `openspec/evidence/java-syntax-2026-10-05/boolean-int-bitwise-patrol/fixture/BW.class`,
whose `andNot` is `a & !b` — the very same branch materialisation
(`iload_0; iload_1; ifne 9; iconst_1; goto 10; 9: iconst_0; 10: iand; 11: ireturn`), whose sibling is
the `Z` parameter `a`. Under the broader rule `andNot` recovered as `return arg0 & !arg1;` (which is
what jadx prints for it too), and that made the **class-level** text compile for the first time —
exposing the class's pre-existing `mix` partial quote (the boolean `^=` accumulation, quoted inside
its surviving loop, its effect dropped) as a reachable compilable-wrong face: the class printed
`true/5/false/…` where the original prints `true/5/true/…`. On the baseline the class's stripped text
does not compile at all (`andNot`'s quote has no `return`), so that face was unreachable.

That row is `recover-boolean-int-bitwise-operands`' own change — the patrol filed it as "the
boolean-consistent restoration of int-ified boolean operands in a purely boolean bitwise context",
with `andNot` and `mix` as its two rows and its own ledger row to close. So this change's admission
is narrowed to the field compound (the bitwise operation must be the value a claimed field write
takes), and `BW` is **byte-identical** to the baseline again — measured, not assumed:
`results/05-corpus-delta.md`.

## The deferred-binding plan reads the same cut

`FieldCopies` now records the cut it proved (`FieldCopyCut { head_block, branch_bci, join_block,
owned }`, the same three facts `concat::Cut` states for a chain), and
`Builder::crosses_a_proved_cut` reads it beside the concatenation rule's own cut. The read's value
is consumed by the join's entry Phi (two use records) and by the `iand` (one real read): counting
the Phi records as evaluations is what produced the cascade diagnostic `the saved producer at BCI 22
has 3 consumers`. The chain rule already states this exact reading for its own cut — the Phi records
a join leaves on a value that travels through it are not consumers — and the field copy's cut is the
same fact.

## What is untouched, verbatim

* `receiver_copy_at`'s own criteria and every one of its refusals: the certificate is entered **only**
  after it returns `None`, so no shape it proved before is proved differently, and no refusal text
  moves (the sweep in `results/05-corpus-delta.md` shows the negatives byte-identical).
* The update rule (`prove_field_update`) and its `+=`/`-=` presentation.
* The concatenation rule: it reads the same `FieldCopyShape::Receiver` the ordinary proof already
  produced, and the new copies are proved before it runs (the `report.rs` order is unchanged).
* No loop-level criterion: the certificate is asked at every `dup`/`dup_x1` candidate, in a loop
  body exactly as outside one; the loop is not part of any condition it states.

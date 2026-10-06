# Task 2.1/2.2 — what was implemented, and how the two shapes are organised

## One plan, two shapes, one gate

`crates/jarde-java/src/build.rs` gains `FieldCopies`: the copies of one body whose consumers are
**field instructions**, proved once per body in the report phase and read by three rules.

```rust
pub(crate) struct FieldCopies { copies: BTreeMap<u32, FieldCopy>, stores: BTreeSet<u32> }

pub(crate) struct FieldCopy {
    duplicate: u32,            // the `dup`/`dup_x1` itself
    source: ValueId,           // what it duplicated: the text each of its copies is written as
    pass_through: Option<ValueId>, // a `dup_x1`'s third write: the value that stood below the top
    shape: FieldCopyShape,
}

pub(crate) enum FieldCopyShape {
    Chain { stores: Vec<u32>, lead: bool, saved: bool },
    Receiver { read: u32, store: u32 },
}
```

`FieldCopies::prove` walks every instruction of the body once (charging `IrItems` per instruction,
the same convention `CompoundAssignments::prove` uses) and reads the two shapes from the copies
themselves — never from an opcode sequence alone:

* **the chain** (`field_chain_at`): a run of consecutive instructions
  `dup; putfield; [dup; putfield]*; putfield`, where each copy's **upper** write is the value the
  store immediately after it takes, each copy's **lower** write is what the next copy reads or what
  the chain's last store takes, and the source is what the first copy read — produced in the same
  block before it, with no other reader. Every value identity is checked (`single_use_at`,
  `field_write_value` = the claimed write's own `shape.value`), and a run that enters a handler is
  no chain. A candidate whose stores another proved chain already owns is that chain's tail, not a
  second shape: `a = b = c = 5` writes one `dup` per extra store, and the lead's chain states the
  source for all of them.
* **the receiver copy** (`receiver_copy_at`): a `dup`/`dup_x1` whose two copies are the receivers of
  a claimed **read** and a claimed **write** of the *same* member (owner, name and descriptor), the
  read immediately after the copy, the write's value produced between them in the same block, the
  read's value a dependency of the write's value (the expression-span walk), and nothing in the
  interval that the expression does not write. A `dup_x1`'s **pass-through** — the value it moved
  rather than duplicated — is identified by the frame's own slot statement (`pass_through_value`:
  the write at the slot the duplicated value was read from).

The **re-usability** criterion is what makes the text the bytecode's own program:
`field_copy_source_is_reusable` admits a source whose closure is constants, local reads and the pure
operations on them, and refuses one whose re-evaluation is observable (a call, an allocation, a field
or array read). The chain's `saved` flag is that answer; the receiver shape requires it outright,
because the receiver's text lands in **two** positions (the assignment's target and the read inside
the expression).

## The rendering

* `Builder::duplicate_expression` consults the plan first: a proved copy's text is the value it
  duplicated (or, for a `dup_x1`'s pass-through, the value that stood below it). Every store of a
  chain writes that text where the store runs — once per store, in bytecode order.
* A chain whose source may not be written once per store writes it into a local at the chain's
  **lead** copy instead: `Builder::field_chain_lead` renders the source, declares `saved{N}` with the
  expression's own presented type, and registers the value in the builder's existing `bindings`
  table — so every later store (and every later copy) reads that name. A lead that cannot commit the
  declaration marks the chain refused (`field_chain_refused`), and every store of it then falls back
  with its own quote: the source is never written twice.
* `Builder::render_instruction` admits the copies (`Operation::Duplicate` and the `dup_x1`'s
  `Operation::Other`) as instructions that write no statement of their own, and
  `renders_the_value_it_reads` counts them as readers — which is what keeps the call a chain's source
  is (`a = b = f()`) from writing a statement of its own *and* being rendered inside the saved
  declaration.
* The stores themselves need no special path: `field_write` renders the receiver and the value it
  already rendered, so a chain store is an ordinary `FieldAssign`, and the compound shape's statement
  is `receiver.field = receiver.field <op> value` with the operator's own expression tree.

## The concatenation rule and the construction rule

`crates/jarde-java/src/concat.rs` gains two admissions, both gated on the proved receiver copy:

* a `dup_x1` whose pass-through is **this chain's instance** joins the chain's owned set and the
  instance's identity set (`produced_by`), so the first `append` that reads the pass-through is a
  reader of the instance;
* the field **read** that copy is the receiver of joins the owned set (its value is an `append`
  operand, checked by the walk's own operand-position rule).

The alias check then exempts the compound assignment's own `putfield` — the write of a receiver copy
this chain owns — because the instance's identity flows *through* that copy and the store is the one
statement the `+=` this chain is the value of is written as. With the chain verified, `new@1` skips
the allocation (`chains.owns(head)`), which is why the `StringBuilder` site needs no change of its
own.

Nothing else in the concatenation rule moved: a chain that holds a field read with no proved receiver
copy behind it (the **static** `String` compound, `sfield += "[" + x + "]"`) keeps the presentation
it already had — the explicit builder chain — byte for byte (`results/renders/*` and the corpus
sweep's zero-delta for the static form).

## The update rule is untouched

`prove_field_update` still proves `+=`/`-=` of an `int` field and presents it as the compound
assignment; every other operator and every reference field now recovers through the receiver copy's
own presentation instead. `AssignOp` gained no variant, the emitter gained no spelling, and the
`p3_compound_lvalue_updates` contract (single evaluation of the left side, the identity/consumer
boundaries) is unchanged — the change's own negatives and that suite's six boundary gaps and five
identity boundaries still refuse.

## Scope note (boundary recorded, not implemented)

* The **instance** chain written with `dup_x1` (`this.a = this.b = this.c = 5`, javac:
  `aload_0; aload_0; aload_0; iconst_5; dup_x1; putfield c; dup_x1; putfield b; putfield a`) is not
  admitted: the chain proof reads `dup` runs, and this shape's copies are consumed by the stores'
  *receivers* as well as their values. It keeps its refusal (`CF`'s own `chain()` covers the static
  form the patrol anchored; the probe is recorded in `results/02-probe-instance-chain.txt`).
* A single-`append` `String` compound (`field += x`) recovers through the same receiver copy and the
  same fold; no separate case exists for it.

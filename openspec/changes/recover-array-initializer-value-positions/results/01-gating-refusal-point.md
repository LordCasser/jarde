# 1.1 — the two refusing positions, instrumented at HEAD (`bd6ba671`)

Three `eprintln!` probes were added to `crates/jarde-java/src/build.rs` and reverted before the
implementation (`git diff --stat` before the revert: 50 insertions, 1 deletion):

* `ArrayInitializers::prove`'s commit loop — every candidate the blanket `ArrayStore` skip drops
  (`JARDE-PROBE commit-skip`), plus the store's own array operand and its SSA definition
  (`JARDE-PROBE commit-skip-store`), plus every chain the loop commits (`JARDE-PROBE commit`);
* `prove_array_initializer`'s final-consumer gate — the consumer it read and every sub-condition
  of the test (`JARDE-PROBE consumer-refusal`);
* `Builder::duplicate_expression` — the copy whose value reached a consumer position with no
  proof (`JARDE-PROBE copy-refusal`).

Inputs: `MD.class`, `MD2.class`, `MD3.class` compiled by `javac --release 8` from the patrol's own
`MD.java`/`MD2.java`/`MD3.java`, rendered with the instrumented binary:

```
jarde-cli class-source --policy plain-jar --input /tmp/mdprobe/mdall.jar --class MD  --format text
jarde-cli class-source --policy plain-jar --input /tmp/mdprobe/mdall.jar --class MD2 --format text
jarde-cli class-source --policy plain-jar --input /tmp/mdprobe/mdall.jar --class MD3 --format text
```

## Position 1 — the element-store RHS (`MD.partSet`, `partial[0] = new int[]{7}`)

```
JARDE-PROBE commit-skip: allocation=5 consumer=12 op=Some(ArrayStore { element: None }) elements=[(ValueId(7), 11)]
JARDE-PROBE commit-skip-store: consumer=12 operands=[(Stack(0), ValueId(0)), (Stack(1), ValueId(1)), (Stack(2), ValueId(4))]
    array_operand=Some(ValueId(0))
    array_def=Some((0, "Some(Field { access: Read, is_static: true, owner: \"MD\", name: \"partial\", descriptor: \"[[I\" })", 0))
```

The candidate **is proved** — its element store is at BCI 11, its consumer is the `aastore` at BCI
12, and the consumer gate *passes* (the `0x53` value-position arm of
`prove_array_initializer`'s final-consumer test; no `consumer-refusal` line is printed for
allocation 5). The refusal is the commit walk's blanket skip at `build.rs:12176`:

```rust
if matches!(operations.get(candidate.consumer), Some(Operation::ArrayStore { .. })) {
    continue;
}
```

so the candidate never enters `proved.owned`/`proved.aliases`, the `dup` at BCI 7 has no proof,
and `duplicate_expression` states it twice:

```
JARDE-PROBE copy-refusal: dup bci=7 value=ValueId(5) consumer-position at=11
JARDE-PROBE copy-refusal: dup bci=7 value=ValueId(4) consumer-position at=12
```

(the store copy at BCI 11 and the retained copy at BCI 12), which is the patrol's
`// the copy at BCI 7 has no proved local assignment` ×2.

## Position 2 — the fresh array's immediate index (`MD3.bareIdx2`, `return new int[]{9}[0]`)

```
JARDE-PROBE consumer-refusal: allocation=1 consumer=8 opcode=0x03 op=Some(Push(Int(0))) operands=[] retained=ValueId(2) single_use=Ok(false)
JARDE-PROBE copy-refusal: dup bci=3 value=ValueId(2) consumer-position at=10
JARDE-PROBE copy-refusal: dup bci=3 value=ValueId(3) consumer-position at=7
```

The refusal is `prove_array_initializer`'s final-consumer gate at `build.rs:12528`, but not for the
reason the six-position discriminator's wording suggests: the gate reads the instruction **right
after the last element store** (`block.instructions()[store_pos + 1]`), and for a subscript that
instruction is the **index** the read produces first (`iconst_0` at BCI 8), not the read. The
retained value's single use is the `iaload` at BCI 9, so `array_initializer_consumer` sees an
instruction with no operand at all (`operands=[]`) and the single-use test is false
(`single_use=Ok(false)`).

`MD2.retPos` is the same shape and the same refusal — its fixture comment calls it 返回位, but
`return new int[]{4}[0];` lowers to the same `…; iastore; iconst_0; iaload; ireturn`:

```
JARDE-PROBE consumer-refusal: allocation=1 consumer=7 opcode=0x03 op=Some(Push(Int(0))) operands=[] retained=ValueId(2) single_use=Ok(false)
```

## The child geometry the blanket skip exists for

Every candidate whose consumer is an element store **of another initializer** has a `dup` behind
the store's array operand — the parent's own copy:

```
JARDE-PROBE commit-skip: allocation=7  consumer=13 op=Some(ArrayStore) elements=[(ValueId(10), 12)]
JARDE-PROBE commit-skip-store: consumer=13 array_operand=Some(ValueId(3)) array_def=Some((4, "Some(Duplicate)", 4))
JARDE-PROBE commit-skip: allocation=7  consumer=14 op=Some(ArrayStore) elements=[(ValueId(10), 13)]
JARDE-PROBE commit-skip-store: consumer=14 array_operand=Some(ValueId(3)) array_def=Some((4, "Some(Duplicate)", 4))
JARDE-PROBE commit-skip: allocation=17 consumer=27 op=Some(ArrayStore) elements=[(ValueId(19), 22), (ValueId(23), 26)]
JARDE-PROBE commit-skip-store: consumer=27 array_operand=Some(ValueId(12)) array_def=Some((14, "Some(Duplicate)", 14))
JARDE-PROBE commit-skip: allocation=18 consumer=30 op=Some(ArrayStore) elements=[(ValueId(19), 24), (ValueId(23), 29)]
JARDE-PROBE commit-skip-store: consumer=30 array_operand=Some(ValueId(12)) array_def=Some((15, "Some(Duplicate)", 15))
JARDE-PROBE commit-skip: allocation=31 consumer=46 op=Some(ArrayStore) elements=[(ValueId(32), 36), (ValueId(36), 40), (ValueId(40), 45)]
JARDE-PROBE commit-skip-store: consumer=46 array_operand=Some(ValueId(25)) array_def=Some((28, "Some(Duplicate)", 28))
```

and the parents commit those children through their own chains:

```
JARDE-PROBE commit: allocation=1 final_value=ValueId(11) consumer=31 elements=[(ValueId(7), 14), (ValueId(20), 30)] children=[7, 18]
JARDE-PROBE commit: allocation=1 final_value=ValueId(24) consumer=47 elements=[(ValueId(7), 13), (ValueId(20), 27), (ValueId(37), 46)] children=[7, 17, 31]
```

The discriminator the gate needs is therefore exactly this one: **the store's array operand is a
value this block's own `dup` produced** (the parent's copy) versus a value the body already had
(`Field { access: Read, … }` for the anchor). Every javac-emitted child store has the former; the
anchor has the latter.

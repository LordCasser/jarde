# Task 1.1 — the refusal points, and whether the three anchor families share a door

Baseline binary: the parent commit `HEAD` (`/tmp/jarde-base-target/debug/jarde-cli`). The transcript
of every render and every rule refusal quoted below is `results/01-gating-transcript.txt`; the
before/after renders of each anchor are `results/renders/*-before.txt` / `*-after.txt`.

## The chain (`CH.chain`, `iconst_5; dup; putstatic c; dup; putstatic b; putstatic a`)

The render carries the copy family's own two-line pair, at **both** copies:

```text
// @bytecode 1
// the instruction at BCI 1 belongs to no shape this run verified: an allocation, a copy or a cast …
// @bytecode 2 0 1 5 6 9 12
// the copy at BCI 1 has no proved local assignment
// @bytecode 5
// the instruction at BCI 5 belongs to no shape this run verified: …
// @bytecode 6
// the copy at BCI 5 has no proved local assignment
```

Two code sites, and they are the ones the dup-store change's gating named:

* the **copy gate** — `Builder::duplicate_expression` (`crates/jarde-java/src/build.rs`), whose
  `local_assignments` lookup answers `the copy at BCI {bci} has no proved local assignment` for a
  `dup` no rule proved. Each store's value operand *is* a copy, so the refusal is raised once per
  store, at the copy's BCI;
* the **instruction fallback** — `Builder::render_instruction`'s `Operation::Duplicate` arm, which
  admits a copy only when `chained_pair` (the *local* chain `x = y = value`) or a proved copy family
  assignment owns it: `the instruction at BCI {at} belongs to no shape this run verified`.

So the chain lands on the same door as the dup-store dance and the postfix snapshot: the copy gate.

## The `String` accumulate (`SC.add`, `new SB; dup; <init>; aload_0; dup_x1; getfield; append…; toString; putfield`)

A **different landing point**, and it is a cascade of three refusals rather than one
(`jre_concat_interleaved_effect`, `jre_new_shape`; the render's own lines follow them):

```text
jre_concat_interleaved_effect :: the concatenation at BCI 0 was not presented: the instruction at
  BCI 8 is an Other between the chain's own instructions, …
jre_new_shape :: the construction at BCI 0 was not presented: the instance the allocation at BCI 0
  builds is read only by instructions this build quotes (BCIs 8), …
```

* the `dup_x1` at BCI 8 is decoded `Operation::Other`, so the concatenation walk refuses it with
  `Precondition::StatementFree`, and the *instruction* fallback refuses it too (`the instruction at
  BCI 8 is not part of the provable subset`);
* with the copy quoted, the `StringBuilder` construction's only reader is a quoted instruction, so
  the construction rule refuses the site (`@new`'s own reader gate);
* and the putfield's value then has no expression to write (`the value at BCI 32 comes from an
  Other at BCI 8`).

The copy's *shape*, though, is the same one the compound RMW below has: its two values are the
receiver of a **field read** (BCI 9) and the receiver of a **field write** (BCI 32) of the same
member. What differs is only the `dup_x1` opcode and that the value between the two is a
concatenation instead of an arithmetic step.

## The compound RMW (`BF.enable`, `aload_0; dup; getfield; iconst_1; iload_1; ishl; ior; putfield`)

The same copy-family pair as the chain (`the instruction at BCI 1 belongs to no shape …` + `the copy
at BCI 1 has no proved local assignment`), and the update rule's own answer is a **silent `None`**:
`prove_field_update` requires the step's opcode to be `iadd`/`isub` (`crates/jarde-java/src/build.rs`),
so `ior`/`ixor`/`iand`/`imul`/`idiv`/`ishl` are never claimed — the patrol's `DV` probe measured
exactly that (`isub` recovers, the rest are quoted).

## The determination

**One door, two shapes.** The copy gate is the door for all three anchors — the same
`duplicate_expression` / `render_instruction` pair the dup-store change extended. What the three
anchors do *not* share is the presentation:

| anchor | the copy's consumers | presentation |
| --- | --- | --- |
| chain (`a = b = c = 5`) | `n` putfields (one per copy, the last copy read by the chain's own next `dup`) | one assignment per store, in bytecode order |
| compound RMW (`this.flags \|= 1 << bit`) | a `getfield` and a `putfield` of one member | `receiver.field = receiver.field <op> value;` |
| `String` accumulate (`this.field += "[" + x + "]"`) | the same, written with `dup_x1` | the same, with the concatenation folded back to `+` |

The compound RMW is therefore **the same door as the `String` accumulate** (both are the receiver
copy of one member's read and write), and the `String` accumulate is this slice's core anchor — so
the compound RMW is covered by the same proof and not handed off. The proof is new but it is one
plan with two shapes, and it plugs into the copy gate the family already had: nothing parallel was
built, and the update rule's own `+=`/`-=` presentation is untouched.

## Task 1.2 — the baseline, re-verified

| probe | baseline |
| --- | --- |
| `CH.chain` | refused (`results/renders/CH-before.txt`) |
| `CH.chainLocal` (`x = y = 7`) | recovered, `int local1 = 7; int local0 = local1;` |
| `SC.add` | refused (`results/renders/SC-before.txt`) |
| `BF.enable`/`disable`, `BG.ienable2` | refused (`results/renders/BF-before.txt`, `BG-before.txt`) |
| `DV.orAcc`/`xorAcc`/`andAcc`/`mulAcc`/`divAcc`/`shlAcc` | refused; `DV.subAcc` recovered (`this.a -= x;`) |
| `CF.call`-shaped probe (`sa = sb = sc = f()`) | refused (the same copy pair, at each `dup`) |
| `NEG`-shaped probes (`x = sa = 7`, `sa = (sb = 5) + 1`, `arr[0] = sa = 3`) | refused, verbatim |

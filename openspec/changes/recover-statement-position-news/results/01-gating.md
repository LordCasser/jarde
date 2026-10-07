# 1.1 — the baseline replay, the refusal point, and the gating experiment

Every line number below is of this change's worktree at its parent commit
(`9faae256`); every quoted excerpt is the file's own text, read with `grep -n`/`sed -n` on the
frozen files. The binaries are the parent commit's (`/tmp/jarde-spn-baseline-target/debug/jarde-cli`,
built from a worktree at `HEAD`) and the patched one (`target/debug/jarde-cli`).

## The fixtures, SHA-checked before anything is counted

The patrol's own transcripts (`openspec/evidence/java-syntax-2026-10-03/statement-new-patrol/results/`)
name the frozen classes by SHA-256; the worktree's copies match them byte for byte:

```
$ cd openspec/evidence/java-syntax-2026-10-03/statement-new-patrol/fixture
$ shasum -a 256 B5.class 'B5$Sub.class' B6.class
8516e097bc5e0dda3e5fa3e137cd820a5aa490d79a861b5a2b466e14fa5c509a  B5.class
0bf59258df570a5f633176a986655416ca7e1cfc09a4aa69ee69e6d13b7e9209  B5$Sub.class
9d266853634774b9feff3c774dc38337fbbf3717997d67ea0c3007cee56be07f  B6.class
```

`results/fixture-sha256.txt` states the same three digests.

## The baseline: the statement position refuses whole methods

`class-source --policy single-class --input B6.class --class B6 --format json --evidence all`, the
`news` records of every member (the rule-details plane; the text alone names only the builder's own
quote):

```
===== B6 (baseline HEAD 9faae256) =====
  argless quality=fallback
    head=0 class=B6 presented=False code=jre_new_shape
      message: the construction at BCI 0 was not presented: the instance the allocation at BCI 0
      builds is read only by instructions this build quotes (BCIs 7), so the construction has no
      place in the body
  withArg quality=fallback
    head=0 class=B6 presented=False code=jre_new_shape
      message: … read only by instructions this build quotes (BCIs 9), …
  consumed quality=structured
    head=0 class=B6 presented=True
  chained quality=fallback
    head=0 class=B6 presented=False code=jre_new_shape
      message: the construction at BCI 4 completes inside the construction at BCI 0, and its value
      is not one of the arguments of the constructor call at BCI 15: the `new` expression has no
      single place to write it
    head=4 class=B6 presented=True
  main quality=structured
    head=15 class=B6 presented=True
===== B5 (baseline HEAD 9faae256) =====
  main quality=fallback
    head=0 class=B5 presented=False code=jre_new_shape  (BCIs 7)
    head=8 class=B5 presented=False code=jre_new_shape  (BCIs 17)
    head=18 class=B5$Sub presented=False code=jre_new_shape  (BCIs 25)
```

The bytecode the readers name is the statement position itself (`javap -p -c`):

```
  static void argless();     0: new B6; 3: dup; 4: invokespecial B6.<init>()V; 7: pop; 8: return
  static void withArg();     0: new B6; 3: dup; 4: bipush 7; 6: invokespecial B6.<init>(I)V; 9: pop
  B5.main:                   0: new B5; 3: dup; 4: invokespecial B5.<init>()V; 7: pop
                             8: new B5; 11: dup; 12: bipush 7; 14: invokespecial B5.<init>(I)V; 17: pop
                            18: new B5$Sub; 21: dup; 22: invokespecial B5$Sub.<init>()V; 25: pop
```

## The refusal point: `written.is_empty()`, not the reader set alone

`crates/jarde-java/src/init.rs`, `verify` (the consumer side of the new@1 proof):

```rust
    let readers = outside_readers(ssa, &produced_by);
    let written: Vec<u32> = readers
        .iter()
        .copied()
        .filter(|bci| renders_its_reads(operations, fields, *bci))
        .collect();
    if written.is_empty() {
        return Err(shape(if readers.is_empty() {
            format!("nothing in this method reads the instance the allocation at BCI {head} builds, …")
        } else {
            format!("the instance the allocation at BCI {head} builds is read only by instructions this build quotes (BCIs {}), …")
        }));
    }
```

`renders_its_reads` (`init.rs`, the function that lists the places a value is written) admits
`Store`, `Invoke`, `InvokeDynamic`, `Return`, `Throw`, `Comparison`, `Switch`, `Arithmetic`,
`Negate` and a *claimed* `Field` — and nothing else. The statement position's reader is the
category-1 `pop`, which the decode states as `Operation::Other` (`decode.rs` models no operation for
`0x57`), so `renders_its_reads` answers false, `written` is empty, and the site refuses with
`jre_new_shape` — the message the baseline transcripts carry above.

The second half the previous dispatch already measured is confirmed here: the refusal is **not** the
reader set's own doing. A probe that only widened `renders_its_reads` (below) flips the record to
`presented=true` while the builder still writes nothing, so both sides must move together.

## The argument-classification data face

Inside `verify` the facts a statement-position criterion needs are already in hand, and no new IR,
SSA or pass is required:

* `operands` — `stack_operands(constructor)`: the physical arguments the constructor call reads, in
  order (receiver first);
* `operations.get(bci)` — `Push` (constant), `Load { slot }` (a local read), `Field`, `Invoke`;
* `ssa.value(read).def()` — `Definition::Entry` for a parameter, `this` or a slot nothing wrote, and
  `Definition::Instruction` for a value some instruction produced;
* `nested_sites` + `is_the_instance` — the constructions this same proof stepped over
  (the `nested-ctor-argument-sites` proof), which are already matched against the call's arguments;
* `argument_dependencies` (`value_dependency_bcis`) — the set the interleaved-effect scan reads.

## The gating experiment: the reader acceptance alone

A probe patch (`init.rs` only, reverted before the implementation) added one arm to the `written`
filter: a `pop` that immediately follows the constructor call, reads the value the call wrote and is
the only reader of it counts as a place the instance is written.

```rust
        .filter(|bci| {
            renders_its_reads(operations, fields, *bci)
                || (member.is_none() && discards_the_instance(ssa, block, at, *bci))
        })
```

With that probe, the same evidence request answers:

```
===== B6 (probe) =====
  argless quality=structured   head=0 presented=True
  withArg quality=structured   head=0 presented=True
  consumed quality=structured  head=0 presented=True
  chained quality=fallback     head=0 presented=False jre_new_shape (unchanged)
  main quality=structured      head=15 presented=True
===== B5 (probe) =====
  main quality=structured      head=0/8/18 presented=True
===== VoidBetween (probe) =====
  make quality=fallback        head=0 presented=False
    code=jre_new_interleaved_effect
    message: the construction at BCI 0 was not presented: the invocation at BCI 4 is not a value
    dependency of the constructor's physical arguments at BCI 8, so presenting the construction
    would move that call effect
```

So the reader acceptance alone flips exactly the statement-position shapes, and the frozen
counterexample does **not** flip: its refusal is the `Invoke ∉ argument_dependencies` branch's, which
runs earlier in `verify` than the reader check and names BCI 4 — the ordering task 2.1 pins in a
test.

The probe also shows why the acceptance alone is not the slice: with the site accepted but no
statement written, the builder's own text drops the construction silently —

```
    static void argless() {
        // @method argless()V
        // @declaration a static method of `B6`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return;                     ← the construction and its side effects are gone
    }
```

— which is exactly the "compilable but behaviour-worse" family the patrol filed. The implementation
therefore writes the statement where the bytecode wrote the constructor call
(`results/02-implementation.md`).

## What the gating experiment pins for the implementation

1. The criterion lives on the **consumer side** of the existing new@1 proof: `written.is_empty()` is
   the refusal that must learn the statement position, and the reader set must learn the `pop`.
2. The `pop` must be *claimed*, not merely accepted: the builder's discard plan
   (`DiscardedEvaluations`, P3 2c.31) already accounts for a call whose result the next `pop`
   discards, so the `pop` writes nothing a second time — verified in the implementation.
3. The CST ordering is not negotiable: the `Invoke ∉ argument_dependencies` branch stays where it
   is, and a test pins the counterexample's code and BCI (task 2.1).
4. The two sides must land together, and the corpus sweep is what proves the pair is complete.

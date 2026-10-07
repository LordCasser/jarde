# 2.1/2.2 — the read-back, and what it deliberately does not touch

## The criterion, in the two halves the proposal states

An `int` operand is read back as the `boolean` it is when **its whole production chain is boolean
bitwise context** and **every consumption is a position a `boolean` is read in**. The change states
each half once, at the place the codebase already reads that kind of fact:

| half | rule | where |
| --- | --- | --- |
| production | a value is a `0`/`1` value: the literal, a `boolean` parameter/`Z` call/`Z` field/`instanceof`/`[Z` element load, a read of a variable the plan decided `boolean`, or a bitwise operator of two such — one of them seeded | `boolean_proof` (existing) |
| production, for a **counter** | every write stores the `0`/`1` literal or a bitwise operator's value, every write's value is boolean evidence (the candidate itself assumed boolean while its own update is judged), and at least one write is the accumulation | `accumulates_boolean` (new, `build.rs:2850`) |
| consumption | every use is: a store into a `boolean` variable, a `Z` method's `ireturn`, a claimed field write, or a bitwise operator whose sibling is boolean evidence — whose own value the walk then judges in turn | `BooleanConsumption` (new, `build.rs:29368`) |

`BooleanConsumption` is entered from exactly two places, and both are the read-back:

* **`Builder::bitwise_boolean_operand`** (`build.rs:15737`): the materialised `0`/`1` of a proved
  conditional value is folded to the test (or its negation) only when the bitwise operator that
  consumes it has a boolean-proven sibling **and** the operator's own value passes the consumption
  walk. That is `andNot`: `iand`'s value is the method's own `ireturn`.
* **`accumulates_boolean`**: each read of the candidate counter's slot must be a load whose loaded
  value passes the same walk. That is `mix`: `iload_1` is consumed by the `ixor` whose sibling is
  the `[Z` element's boolean, and that `ixor`'s value is stored back into the counter; the exit's
  `iload_1` is consumed by the method's `ireturn`.

The candidate counter is read as `boolean` **only while its own writes are judged** (the value it
carries is what its own update combines); every other variable is read from the plan's decision so
far, and the new rule runs inside `decide_types`' own fixpoint, so a counter the rule admits can be
the sibling another counter's update reads, and a copy chain can spread the decision further. That
is what makes the rule a data-flow sufficiency test rather than a guess: the plan states a type
once, before any statement exists, and every consumer reads that one decision.

## The gating (task 1.1 → the first measurement)

* the refusal has one emitter: `build.rs:23943`, inside `render_value`'s `Operation::Bitwise` arm,
  fired by `Expr::presented` being `None` — and `presented` is computed by `ast::binary_type`'s
  bitwise arm, which accepts only two `boolean`s or two integrals (`results/01-gating-refusal-point.md`);
* on the patrol's frozen `BW.class` the two refusals flip and nothing else moves:
  `results/02-gating-experiment.txt` — `baseline: 2` → `patched: 0`, `andNot` → `return arg0 &
  !arg1;`, `mix` → `boolean local1; … local1 = local1 ^ local5; … return local1;`;
* the genuinely mixed shapes do **not** flip: the same transcript's `BWN` diff is empty — the
  `0`/`1` consumed by arithmetic (`plusOne`), the bitwise value consumed by a branch test
  (`andNotInt`), the counter read by a comparison (`compareRead`) or by a call argument (`passed`),
  and the `int` sibling (`intSibling`) all keep the text they had.

## The `BW` class-level resolution — the decision this slice owes

The previous slice measured that a **wider** rule (any bitwise consumer with a boolean sibling)
makes `BW.andNot` recover and thereby exposes `mix`'s pre-existing partial quote as a reachable
compilable-wrong face (`true/5/false/…` where the original answers `true/5/true/…`), and narrowed
itself to field writes to keep `BW` byte-identical.

**Decision: `mix` is in-domain, and it recovers *with* `andNot`; no guard is added.** The evidence
is the patrol's own filing and this change's own design:

* the patrol's README names both shapes as one finding — "两形受影响（布尔累积 xor、`a & !b`）" —
  and its mechanism is one: javac int-ifies the boolean (`!b` → `xor 1`; `r ^= x` → an `int`
  counter with the `% 2 != 0` exit);
* this change's design decision 2 names the accumulate counter's presentation explicitly
  (`boolean local = false; … local ^= x;`, the exit `return local;`), and the spec's second scenario
  is `r ^= x`'s accumulation;
* `mix` is not the int/boolean **mixed arithmetic** form the Non-Goals exclude: its every write
  stores a `0`/`1` value and every read is consumed where a `boolean` is read, which is exactly the
  sufficiency criterion the proposal states.

So the class's stripped text compiles and answers what the original class answers — the third value
included:

```text
$ sh openspec/changes/recover-boolean-int-bitwise-operands/results/04-class-level.sh
SELF-TEST OK: the frozen BW answers true/5/true/2/-2147483648/false/3
--- frozen
original      : true/5/true/2/-2147483648/false/3
javac 23 --release 8: true/5/true/2/-2147483648/false/3
corretto 1.8.0_432   : true/5/true/2/-2147483648/false/3
--- v8 / --- v8-javac8      (the recompiled legs: identical, both compilers)
--- bwr
original      : false/false/false/false
javac 23 --release 8: false/false/false/false
corretto 1.8.0_432   : false/false/false/false
negative BWN (v8): does not compile, as the refusals require (2 missing-return errors)
negative BWN (v8-javac8): does not compile, as the refusals require (2 missing-return errors)
CLASS-LEVEL PROBE OK: every recovering class answers what its own class file answers, on both legs
```

The same replay is the ignored test
(`cargo test --test recover_boolean_int_bitwise_operands --all-features --locked -- --ignored`),
which reads the frozen class out of the committed fixture and compares against it directly.

## The presentation (task 2.2)

* the counter's declaration becomes `boolean r;` and its initialising write `r = false;` — the
  `0`/`1` literal spelled as the boolean the decision states (`boolean_spelling`);
* the update is written the way this layer writes every local update: the plain assignment
  `r = r ^ x;`, exactly as the same class's Kernighan loop writes `arg0 = arg0 & arg0 - 1;` (the
  `^=` form the design's sketch shows is not this layer's spelling for locals);
* the exit's `% 2 != 0` outlet **dissolves**: `iload_1` now presents `boolean`, so the `Z` return
  writes `return r;` instead of `return r % 2 != 0;` — the design's "出口直接 `return local;`";
* `andNot` writes `return arg0 & !arg1;` — the `Not` node the layer already builds for `x ^ 1`
  beside a boolean operand, now reachable for the materialised `!b` too.

## What is untouched, verbatim

* every refusal text: the emitter's own diagnostic, the comparison pair's, the argument's — all
  byte-identical (the sweep's `BWN` self-test and the whole-corpus diff);
* the **same-type** bitwise shapes: `boolean ^ boolean`, `int ^ int`, `arg0 << 33`, the folded
  `-2147483648` and the Kernighan loop render byte for byte as before (pinned by the new test, and
  by the sweep's `BW` diff showing only the two anchors' lines);
* the previous slice's admission: `bitwise_boolean_operand` keeps the claimed-field-write position,
  so `recover-conditional-rhs-field-compound`'s `BI`/`RC`/`RCN` presentations are byte-identical
  (the sweep's own self-test), and its suite is green;
* `boolean_position_values`' other two positions (`Z` return, `Z` argument) and the fold's own
  criteria (`integer_constant`, `replaced_by`) are unchanged;
* no budget charge is added to the render pass's admission check: it runs unbilled exactly as the
  `boolean_evidence` read beside it does, while the declarations plan's own walk charges
  `AnalysisSteps` per judged consumer and the bitwise proof nodes as it already did.

## The boundaries this slice states and does not cross

| shape | why it keeps the refusal |
| --- | --- |
| `boolean r = a & !b; return r;` | the materialisation's consumer is a **store into a local the plan did not decide `boolean`** (its first write stores the bitwise value, whose evidence the plan cannot state for a stack-Phi operand) — so the consumption walk refuses the store position and the operand pair keeps its refusal. Measured byte-identical to the baseline on both binaries |
| `return (a & !b) ? 1 : 0;` | the bitwise value is consumed by a **branch test**; a branch test is not one of the four admitted positions, so the materialisation stays the `int` it is |
| `… r ^= x; System.out.println(r);` | the counter is read as a **call argument**; the argument position is not admitted |
| `… r ^= x; return r == other;` | the counter is read by a **comparison** |
| `int r = 0; r ^= x;` (int sibling) | the write's own value has no boolean evidence at all: the sibling is an `int`, so `boolean_proof` refuses the operator |

Every row is pinned by `tests/recover_boolean_int_bitwise_operands.rs` (the `BWN` fixtures) and
classified in `results/05-corpus-delta.md`.

## The one place the walk is conservative by construction

A use with no BCI is a phi operand — a merge, not a consumer. The walk follows it only when the
merge is a **local variable's** own phi that the caller's decision states `boolean` (the value
becomes that variable's value, and every read of the variable is judged by the decision for it); a
**stack** phi is refused, because no decision governs what its later readers do with it. A use chain
that revisits a value, or nests deeper than `MAX_VALUE_DEPTH`, is refused the same way.

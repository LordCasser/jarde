# Task 2.1 — what the instance form is, and what it changed

## The admission (`crates/jarde-java/src/build.rs`)

`FieldCopies::prove`'s `OPCODE_DUP_X1` arm gains one fallback: after the receiver copy and the
conditional receiver copy refuse, `instance_chain_at` reads the run the instance chain is made of.
The function proves one thing: the shape javac writes for `this.a = this.b = this.c = value`, with
every condition an identity the bytecode states.

```text
lead = the first `dup_x1` of the run
source = the value the lead duplicated        (produced in this block before the lead, read by nothing else,
                                               and reusable — see the boundary below)
loop over the run, one store per copy:
  the copy at `position` reads (top, below) by its own two stack slots;
    top must be the chain's value: the source, or the value copy the copy before left below;
  dup_x1_writes identifies the three values it wrote by the slots its own reads name:
    the copy of `top` that lands in `below`'s slot  (the value that stays under the receiver)
    the `below` value moved into `top`'s slot       (the receiver the store is called on)
    the copy of `top` that lands above both         (the store's value)
  the instruction right after the copy is an **instance** field write whose
    receiver is the moved value and whose value is the copy on top, and both are read by it alone;
  next instruction:
    another `dup_x1` → the value copy left below must be read by it alone; continue
    a field write    → the chain's last store: instance write, value = the surviving copy,
                       receiver = the receiver that never moved, both read by it alone,
                       and the receiver is `this`; the run ends
    anything else    → refuse
every store's moved receiver — the value that stood below the copy's top — is `this` (`reads_this`)
the whole run is one statement group: no instruction of it may enter a handler
```

`reads_this` is the identity the instance form adds: the value is produced by a `load` of **slot
0** whose own read is the entry state's value of slot 0. Three `aload_0`s are three SSA values, but
they read the one value `this` is — which is exactly the criterion the spec states ("the same
`this` SSA value", the identity of the three `aload_0`s), and it is what refuses `o1.a = o2.b = 5`
(two different receivers), `h().a = h().b = 5` (a call result) and `this.f().a = this.f().b = 5`.

`dup_x1_writes` is new and states the stack geometry the change is about, in the slots the
instruction's own reads name:

```rust
fn dup_x1_writes(copy, below, top) -> Option<(ValueId, ValueId, ValueId)>
// reads:  [(Stack(below_depth), below), (Stack(top_depth), top)], top_depth == below_depth + 1
// writes: [(below_depth, copy), (top_depth, moved), (top_depth + 1, value)]
```

Nothing is inferred from the opcode sequence: each of the three writes is tied to the slot one of
the two reads stood in, and the two reads are tied to the values the instruction really carries.

## The rendering

A store's **receiver** is a value the copy wrote (the moved one), so the copy has to say what that
value's text is. `FieldCopy` gains one field:

```rust
/// The receiver one instance chain's copy **moved**, and the value it moved it from …
pub(crate) moved: Option<(ValueId, ValueId)>,
```

and `field_copy_expression` renders it where the value being rendered is that moved value:

```rust
if let Some((moved, below)) = copy.moved
    && moved == value
{
    return Ok(self.render_value(below, at, depth + 1)?.derived_from(copy.duplicate));
}
```

The value's text is the receiver that was read from below (`this`), and the copy's own BCI joins
the origin as a derived member — the same shape the pass-through branch already uses. Every other
copy renders its duplicated value's text (`copy.source`), which is what makes each store write the
once-evaluated expression: `this.c = 5; this.b = 5; this.a = 5;`.

The three existing `FieldCopy` construction sites carry `moved: None`, so the static chain, the
receiver copy and the conditional receiver copy are the code they were.

## The signature change

`FieldCopies::prove` takes one more fact — `has_receiver`, `MethodFacts::has_receiver()`'s own
reading of `ACC_STATIC` ("an instance method and a constructor take a receiver, a `static` method
and a static initializer do not"). `instance_chain_at` refuses a body without a receiver before it
reads anything: a `static` method's slot 0 is a parameter, and a chain of its fields is not this
shape. `report.rs` passes the fact from the same method facts the run already holds.

## Boundaries this slice records instead of improvising

1. **The mixed chain stays refused.** `MX`'s three dances (`this.a = this.b = MX.s = 5`,
   `MX.s = this.a = this.b = 6`, `this.a = MX.s = this.b = 7`) carry a static store inside the same
   dance — javac writes `dup; putstatic s; dup_x1; …`, `… dup_x1; putstatic s` and
   `… dup; putstatic s; …` respectively. The instance form's run is the one whose every store is an
   instance write called on the same `this`; the static store's own copy is a bare `dup` and the
   run that carries it is the static chain's shape, whose proof reads one run of `dup`s and refuses
   the interleaved copies. Both forms therefore keep their refusal, and the three dances are pinned
   verbatim by `tests/recover_instance_field_assignment_chains.rs`.
2. **A source that would have to be saved stays refused.** `this.a = this.b = this.c = f()` is the
   instance form's saved-local case: the instance form writes no saved local of its own, so the
   proof admits only a source whose re-evaluation is not observable
   (`field_copy_source_is_reusable`) — the "pure source shares the same evaluated expression text"
   invariant. The static chain's saved local (`int saved0 = f();`) is untouched, and the instance
   probe's `NEG.saved` pins the refusal.
3. **A receiver that is not `this` stays refused**, including a parameter chain (`o.a = o.b = 5`
   with `o` in a local slot) and a `static` method's first-parameter chain: the criterion is
   stated on the method's own receiver slot, which is what `has_receiver` proves.
4. **A mid-dance start is refused.** The proof can be entered from a later copy of the run; such a
   run's source is itself a copy, which `field_copy_source_is_reusable` refuses, so the only
   admission possible is the one that states the whole run from its first copy.

## The test target

`tests/recover_instance_field_assignment_chains.rs` (root `tests/`, the facade `Engine` +
`ClassSourceRequest` + a jar built from the committed class files + the self-header assertion
before anything is counted) pins:

* `CP.inst`/`CP.pair` on both legs, and that `CP` keeps no `// @bytecode` line at all;
* the four recovering methods' quote-free bodies;
* `MX`'s three refusals and `NEG`'s four refusals verbatim, with no field assignment presented in
  a refused body;
* the ignored replay: `CP`'s stripped text compiles with `javac --release 8` **and** with real
  javac 8 (Corretto 1.8.0_432), runs under `-Xverify:all`, and answers identically to the committed
  class file; `MX` and `NEG`'s stripped texts must not compile.

The fixtures are `tests/fixtures/recover-instance-field-assignment-chains/{v8,v8-javac8}/` with
their `README.md`; the same bytes are in `tests/fixtures/corpus-fingerprint.json` (blake3), which
`tests/p5_corpus_fingerprint.rs` verifies.

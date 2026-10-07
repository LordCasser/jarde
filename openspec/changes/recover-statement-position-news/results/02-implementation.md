# 2.1/2.2 — the implementation, the zero-regression checks, and the regression the sweep caught

## What changed, and where

The statement position extends the **existing** new@1 construction proof's consumer side; it is not
a parallel mechanism, and no new IR, SSA, pass or public API is introduced.

### `crates/jarde-java/src/init.rs` (the proof)

* `Site` gains one field — the `pop` that discards the finished instance:

  ```rust
  /// The `pop` that discards the finished instance, when this site is a **statement-position**
  /// construction: `new X(args);`. … `None` for every construction a store, a call, a `return` or
  /// a claimed field access consumes.
  pub(crate) discarded: Option<u32>,
  ```

* `verify`'s consumer side learns it. The reader set and the `written` filter are unchanged; what
  is new is the **discarded** verdict, read only where the old code refused:

  ```rust
  let discarded = if member.is_none()
      && written.is_empty()
      && let [pop] = readers.as_slice()
      && discards_the_instance(ssa, block, at, *pop)
      && arguments_without_invocations(ssa, operations, &operands, &nested_sites)
  {
      Some(*pop)
  } else {
      None
  };
  if written.is_empty() && discarded.is_none() {
      return Err(shape(…));          // the refusal text is byte-identical to the baseline's
  }
  …
  let written: Vec<u32> = match discarded {
      Some(pop) => vec![pop],
      None => written,
  };
  ```

  The three facts the criterion reads are the bytecode's own, and the refusal text for every shape
  it does not admit is the one the baseline stated (so the negatives are verbatim).

* `discards_the_instance` — the `pop` is the block instruction **immediately after** the constructor
  call, its opcode is `0x57` (the decode states no operation for it, so the opcode is read from the
  instruction), it reads exactly one value, that value is the one the constructor call wrote, and
  nothing else reads it. Those are the same identities `Builder::discarded_evaluations` reads
  (P3 2c.31), so the two layers cannot disagree about which `pop` a construction's discard is.

* `arguments_without_invocations` — every physical argument is one of the three admitted producers
  (design decision 1): `Operation::Push` (a constant), `Operation::Load` whose read is
  `Definition::Entry` (a direct local/parameter read — a load of a slot some store filled reads that
  store's value and is not admitted), or the completed instance of a construction this same proof
  stepped over (`nested_sites` + `is_the_instance`). Anything else — an invocation, a field read, an
  arithmetic or conversion chain, a merge — keeps the construction's refusal.

### `crates/jarde-java/src/build.rs` (the presentation)

* `instruction()`'s site arm writes the statement, and only for the constructor of a discarded site:

  ```rust
  if let Some(site) = self.sites.site_of(at)
      && at == site.constructor
      && site.discarded.is_some()
  {
      return self.discarded_construction(site, at);
  }
  ```

* `discarded_construction` writes `StmtKind::Expr(self.new_expr(site, at, 0))` anchored at the `pop`
  that discards it — the same expression the consumed position writes, in the position the bytecode
  wrote the constructor call. The `pop` itself writes nothing: the discard plan already accounts for
  it (`DiscardedEvaluations`, whose first arm covers `invoke…; pop`), so no text is written twice. A
  text this layer cannot write falls back through `binding_quote_bcis`, which names the constructor,
  the site's own instructions and the `pop`, so nothing the bytecode evaluated leaves the artifact
  unaccounted for.

### The regression the corpus sweep caught (and the fix)

The first draft of the `instruction()` rewrite replaced the single early return
(`self.array_initializers.owns(at) || self.chains.owns(at) || self.sites.owns(at) || …`) with a
`self.sites.site_of(at)` branch. `Sites::owns` is the **owned set**, which also holds the three
instructions of every *bound-receiver tail* (`dup; <discarded null check>; pop` over a lambda's
captured receiver, `ReceiverTail`); those instructions have no `Site`, so `site_of` missed them and
the tail's `dup` fell through to the ordinary arms and rendered as a copy:

```
MOVED (single-class): tests/fixtures/recover-proved-nonnull-bound-receivers/v8/OP.class
    > // jarde: not recovered: the recovery run for `sideEffect(…)` produced no statement …
    > // the copy at BCI 10 has no proved local assignment
MOVED (single-class): openspec/evidence/java-syntax-2026-10-04/method-reference-patrol/fixture/M1.class
    < // jarde: omitted physical lambda helper "lambda$main$0" after proving its single class-wide use
```

The fix keeps `Sites::owns` exactly where it was and only adds the statement arm **before** it:

```rust
if self.array_initializers.owns(at) || self.chains.owns(at) {
    return Ok(());
}
if let Some(site) = self.sites.site_of(at)
    && at == site.constructor
    && site.discarded.is_some()
{
    return self.discarded_construction(site, at);
}
if self.sites.owns(at) || self.clause_parameters.contains(&at) || … {
    return Ok(());
}
```

Both classes are byte-identical to the baseline again, and the final sweep moved nothing outside the
delta list of `results/05-corpus-delta.md`.

## Zero-regression: the consumed positions and the existing channel

* `cargo test --test p3_ordinary_new_invokes` — the frozen counterexample's two tests pass unchanged
  (`jre_new_interleaved_effect` at BCI 4, the origin set, the real-call-argument control).
* `tests/recover_statement_position_news.rs` pins the consumed positions in-text (`B6.consumed`,
  `B6.main`'s `new B6(9).n`, `SP.consumed`, `SPN.<clinit>` and `SPN.main`).
* The corpus sweep (pass A: 2838 loose classes; pass C: 739 jar entries) moved 14 classes, every one
  of them classified in `results/05-corpus-delta.md`; **no class became more refused or more
  quoted** (a count over every rendered pair: `jarde_refused_body` lines and `@bytecode` lines never
  grow).
* The three precedent families' own controls (`BI`/`RC`/`RCN`, `CF`/`NEG`, `ICM`/`ICN`) and the
  bitwise change's `BW`/`BWN` are byte-identical on both binaries — the sweep's self-test.

## Budget and cancellation

The criterion reads facts the run already holds (the SSA table, the decoded operations, the nested
sites and the physical operands) and charges nothing new: `init::sites` bills no per-instruction
dimension today, and the new work is bounded by the constructor's own argument count plus one
block scan. No budget dimension, no cancellation point and no public request shape changed.
`cargo test --workspace --all-targets --all-features --locked` covers the budget and cancellation
suites unchanged (`results/06-gates.md`).

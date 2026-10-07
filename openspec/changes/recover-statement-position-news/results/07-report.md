# `recover-statement-position-news` — the implementer's report

## Status

Implemented, self-tested and gated in this worktree. **Not pushed**; the five commits below are the
complete change. Root reviews, merges and accepts on `main`.

## 1.1 — the gating, in one paragraph

The patrol's `B5.class`/`B5$Sub.class`/`B6.class` are SHA-identical to `results/fixture-sha256.txt`.
At the parent commit the statement position refuses whole methods: `B6.argless`/`withArg` and all
three of `B5.main`'s sites answer `jre_new_shape` — "the instance … is read only by instructions
this build quotes (BCIs 7/9/17/25)" — because the only reader of the instance is the category-1
`pop`, which `renders_its_reads` does not list, so `verify`'s `written.is_empty()` branch fires. The
gating experiment (a temporary probe widening only that reader set) flips exactly those sites to
`presented=true`, leaves the frozen counterexample `VoidBetween.make` refused with
`jre_new_interleaved_effect` naming BCI 4 (the `Invoke ∉ argument_dependencies` branch runs first),
leaves `chained` refused, and — critically — shows the accepted site's statement is then **silently
dropped** from the text (`argless() { return; }`): both sides must move together. Full transcript:
`results/01-gating.md`.

## What changed

`crates/jarde-java/src/init.rs` — `Site` gains the `pop` that discards the finished instance, and
`verify`'s consumer side accepts the statement position when the sole reader is that `pop` (the block
instruction immediately after the constructor call, reading the value the call wrote, single use) and
every argument is a value the `new` expression writes in place: a constant, a direct local/parameter
read, or a construction this same proof completed. Every other shape keeps the refusal text it had.

`crates/jarde-java/src/build.rs` — the builder writes `new X(args);` at the constructor's own
instruction, anchored at the `pop`; the `pop` writes nothing a second time because
`DiscardedEvaluations` already accounts for it (P3 2c.31). `Sites::owns` keeps its exact place, so
the bound-receiver tails a verified site owns stay silent — the regression the first draft caused
and the sweep caught (`OP`, `M1`; `results/02-implementation.md`).

No new IR, SSA, pass, budget dimension, cancellation point or public API. `results/02-implementation.md`
is the mechanism and the zero-regression notes.

## Per-task evidence

| task | evidence |
| --- | --- |
| 1.1 replay, refusal point, data face, gating | `results/01-gating.md`, `results/01-baseline-news.txt` |
| 1.2 variants/negatives frozen and replayed | `tests/fixtures/recover-statement-position-news/` (README + both legs + the assembled `SPC`), `tests/recover_statement_position_news.rs`, `results/04-class-level.out` |
| 2.1 the criterion; B5/B6 recover; CST unchanged; `chained` refused | the acceptance transcript below; `results/01-gating.md`; the CST ordering pinned in two tests (`SPC` statement-position twin + the frozen `VoidBetween`) |
| 2.2 zero regression, budget/cancel | `results/02-implementation.md`, the sweep, the workspace run |
| 3.1 gates | `results/06-gates.md` (verbatim) |
| 3.2 three-way comparison and behaviour | `results/04-class-level.sh/.out`, the ignored replay test |
| 3.3 root's independent review | **not mine** — the change is handed over for it |

### The acceptance anchors, on the final binary

```
===== B5 =====
  main         quality=structured news=[(0, True, None), (8, True, None), (18, True, None)]
      3 construction candidate(s) read under new@1: 3 presented as `new`, 0 refused
===== B6 =====
  argless      quality=structured news=[(0, True, None)]
      1 construction candidate(s) read under new@1: 1 presented as `new`, 0 refused
  withArg      quality=structured news=[(0, True, None)]
      1 construction candidate(s) read under new@1: 1 presented as `new`, 0 refused
  consumed     quality=structured news=[(0, True, None)]
  chained      quality=fallback   news=[(0, False, 'jre_new_shape'), (4, True, None)]
      2 construction candidate(s) read under new@1: 1 presented as `new`, 1 refused
  main         quality=structured news=[(15, True, None)]
```

`B5` renders whole (0 quoted BCIs) with `new B5(); new B5(7); new Sub();`; `B6.argless`/`withArg`
render whole with `new B6();`/`new B6(7);`; `chained` keeps its refusal; the consumed positions do
not move. The stripped `B5` text compiles with `javac --release 8` **and** real javac 8, runs under
`-Xverify:all` and prints the committed `orig.out` byte for byte — the three constructor `println`s
included. `B6`'s text stays incomplete by design (the registered `chained` boundary) and is asserted
**not** to compile, so nothing compiles silently wrong.

## Verbatim gate tails (authoritative, final tree)

```
$ cargo fmt --all -- --check
FMT-OK
$ sh /tmp/ci-clippy.sh          # ci.yml 46–76, and the same with -D warnings appended
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 32.90s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 32.07s
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
EXIT=0
331
0
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.58s
$ git diff --check
DIFF-CHECK-OK
$ openspec validate --all --strict
Totals: 307 passed, 0 failed (307 items)
```

(`331` = `grep -c 'test result: ok'`; `0` = `grep -c 'test result: FAILED'`; the exit code is the
run's own. No flakes were seen in the two full runs on this tree; the first run's four failures were
stale corpus expectations, each updated or regenerated — `results/06-gates.md` lists them.)

## Corpus delta and the oracle leg

```
SELF-TEST OK: B5 refusals 3 -> 0 with all three statements; SPC/SPN byte-identical; BW/BWN byte-identical; BI/RC/RCN/CF/NEG/ICM/ICN byte-identical
pass A loose candidate classes: 2838      pass A: moved=13 unrendered=1
archive candidate classes: 739            pass C: moved=1 unrendered=1
moved classes: single-class=13 jar=1 total=14
unrendered candidates: A=1 C=1            (both the same package-info entries every precedent printed)
```

Every one of the 14 deltas is classified in `results/05-corpus-delta.md`: 8 recover into texts that
compile and answer exactly what their original classes answer under `-Xverify:all` (including the
patrol's own swallowed constructor side effects), and the other 3 classes' texts were **already** not
compilable units on the baseline for reasons this slice does not own (the self-nested pool spelling
in `DN`, the member-class spelling in `D3` — proved pre-existing by a consumed-position control that
is byte-identical on the baseline — and `Multiseg`'s refused declaration). No class became more
refused or more quoted. The oracle leg (`p3_execution_comparison --ignored`) passed 3/3 with **no
stale expectation**. The census moved to `(844, 3702, 290, 2273, 8)` and the fingerprint gained
exactly this change's seventeen fixture files.

## Commits (no push)

| commit | contents |
| --- | --- |
| `2ca98098` | `feat(java): present a statement-position construction as the expression statement it is` — `crates/jarde-java/src/{init.rs,build.rs}` |
| `d2e7bff2` | `test(statement-position-news): freeze the anchors, the variants and the boundaries on both legs` — the fixture family and `tests/recover_statement_position_news.rs` |
| `33a651e2` | `test(corpus): update the two expectations the statement position moves, and re-measure the census` — `tests/class_source.rs`, `tests/recover_javac8_allocation_qualifier_null_check.rs`, the reader's census ledger |
| `6cf1207d` | `test(census): re-render the corpus fingerprint for this change's fixtures` |
| `1c384ace` | `docs(change): the gating transcript, the implementation, the corpus differential and the gates` |

## Remaining boundaries (registered, not recovered by this slice)

* **`chained`** — `new B6(new B6(1).n)`: the argument is a `getfield` over a construction that
  completes inside this one; the field read triggers the declaring class's `<clinit>`, so it keeps
  its `jre_new_shape` refusal (the design's registered boundary).
* **Any invocation at an argument position** — `new SPN(probe());` keeps the reader-check refusal
  verbatim (the frozen CST family's discipline); the invocation that is *not* an argument dependency
  is refused earlier still, by the `Invoke ∉ argument_dependencies` branch, and that ordering is
  pinned by two tests.
* **Arithmetic/merge arguments** — `new SPN(x + 1);` is neither a constant, a direct read nor a
  proved chain, so the strict enumeration keeps its refusal (`SPN.arithmeticArgument`).
* **Member (inner) constructions in the statement position** — `member.is_none()` is part of the
  criterion: a site the member projection proves keeps its own proof and is not admitted here. A
  site whose class is a member class of the declaring class but whose projection is *not* proved
  (`D3`) recovers in the channel's own expression spelling (`new D3$In(new D3())`) — the same text
  the consumed position already writes for it — which is not valid Java; that spelling belongs to
  the member projection's route, not to this slice, and is the one delta root may want to rule on.
* **The output of a `sites.owns`-only early return** — the first draft's rewrite dropped the
  bound-receiver tails; the final code keeps `Sites::owns` where it was, and the sweep's self-test
  families plus `OP`/`M1` are the guard.

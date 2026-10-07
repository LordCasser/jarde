# `recover-boolean-int-bitwise-operands` — the implementer's report

Status: **implemented, self-verified, awaiting root's independent review (task 3.4)**. Nothing is
pushed; four commits sit on this worktree's branch.

## 1.1 gating, and the class-level resolution decision

* the refusal has one emitter (`build.rs:23943`, `render_value`'s `Operation::Bitwise` arm, fired by
  `Expr::presented` being `None`) and one carrier of the operands' types
  (`ast::binary_type`'s bitwise arm) — `results/01-gating-refusal-point.md`;
* on the patrol's frozen `BW.class` the two refusals flip and nothing else moves
  (`results/02-gating-experiment.txt`): `andNot` → `return arg0 & !arg1;`, `mix` →
  `boolean local1; … local1 = local1 ^ local5; … return local1;`; the `BWN` negatives' texts are
  byte-identical on both binaries;
* **the `mix` decision: it is in-domain, and it recovers *with* `andNot`; no guard was added.** The
  patrol's README files both shapes as one finding, this change's design decision 2 names the
  accumulate counter's presentation and its spec's second scenario is `r ^= x`, and `mix` is not the
  mixed-arithmetic form the Non-Goals exclude (every write stores a `0`/`1` value; every read is
  consumed where a `boolean` is read). The class-level probe
  (`results/04-class-level.txt`) is the acceptance: the frozen class answers
  `true/5/true/2/-2147483648/false/3` and the recovered text answers the same on both compilers under
  `-Xverify:all` — the third value included, so no compilable-and-different state exists.

## What changed

| file | change |
| --- | --- |
| `crates/jarde-java/src/build.rs` | `BooleanConsumption` (the consumption walk), `bitwise_boolean_operand` widened from the field-write position to that walk, `accumulates_boolean` (the counter's seed) and the `decide_types` fixpoint that runs it |
| `tests/fixtures/recover-boolean-int-bitwise-operands/` | `BW.java` + `patrol-BW.class` (the patrol anchor, byte for byte), `BWR.java`, `BWN.java`, both compiler legs, the README with commands and sha256 |
| `tests/recover_boolean_int_bitwise_operands.rs` | the anchors, the same-type pins, the negatives verbatim, the dual-leg replay |
| `crates/jarde-reader/src/classfile.rs`, `tests/fixtures/corpus-fingerprint.json` | the census ledger entry and the regenerated fingerprint (+10 files) |

## Verification actually run

| check | result |
| --- | --- |
| the change's suite | 5 passed / 0 failed / 1 ignored, and the ignored replay passes |
| the three precedent suites | green, and all three replays pass |
| class-level probe (frozen + both legs, both compilers, `-Xverify:all`) | identical answers; `BWN` does not compile |
| corpus sweep (2822 loose + 739 archive candidates, both postures) | moved = 6, all this change's own anchors/fixtures; jar moved = 0; the two `package-info` candidates unrendered on both binaries, as every precedent sweep recorded |
| `p3_execution_comparison --ignored` (the oracle) | 3 passed; **no stale expectation to update** |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | run 3: `EXIT=0`, 330 `test result: ok`, 0 `test result: FAILED` (runs 1 and 2 carried two load-sensitive flakes, each rerun green ×2; `results/06-gates.md`) |
| fmt, CI-exact clippy (with `-D warnings`), `openspec validate --all --strict`, `git diff --check`, census, fingerprint | all clean; transcripts in `results/06-gates.md` |

## Remaining boundaries

* `boolean r = a & !b; return r;` (a materialisation stored into a **local**) keeps its refusal
  verbatim: the local's type is the plan's decision and the plan cannot state `boolean` for a
  variable whose first write stores a bitwise value over a stack-Phi operand. Measured byte-identical
  on both binaries;
* the branch-test, comparison, call-argument and `int`-store consumptions keep their refusals (pinned
  by `BWN`), and the int-sibling counter keeps its `int` presentation;
* the walk is conservative by construction on a stack-Phi merge, a revisited value and a chain deeper
  than `MAX_VALUE_DEPTH` (`results/03-implementation.md`);
* the render pass's admission check runs unbilled (exactly as the boolean-evidence read beside it
  does); the declarations plan charges `AnalysisSteps` per judged consumer and the bitwise proof
  nodes as before.

## Re-running the evidence

* this worktree's `target/` is cleaned before this report: `cargo build -p jarde-cli --locked` first,
  then `sh openspec/changes/recover-boolean-int-bitwise-operands/results/03-corpus-sweep.sh` (the
  script now refuses to start when either binary is missing);
* the baseline binary is the parent commit's own build in `/tmp/jarde-bwslice-baseline` with
  `CARGO_TARGET_DIR=/tmp/jarde-baseline-target` (kept in place);
* `sh .../results/04-class-level.sh` and `sh .../results/02-gating-experiment.sh` are self-contained
  and re-run the two acceptance transcripts.

## Commits (no push)

| commit | contents |
| --- | --- |
| `0c3cca97` | `crates/jarde-java/src/build.rs` |
| `417580e3` | the fixtures, the README and `tests/recover_boolean_int_bitwise_operands.rs` |
| `5fddd02b` | the census ledger entry and the regenerated `corpus-fingerprint.json` |
| `b442c69f` | `openspec/changes/recover-boolean-int-bitwise-operands/` (the evidence) |
| the head commit (`git log -1`) | this report and the sweep's binary preflight |

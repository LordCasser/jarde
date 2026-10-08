# Task 1.2/2.2 — the frozen anchors and negatives

Every text and refusal below is pinned in `tests/recover_loop_test_copy_store.rs` (this slice's own
fixtures), `tests/recover_io_resource_finally.rs` (the io anchor, updated) and
`tests/recover_dup_store_conditional.rs` (the dup-store family's loop-test negative, updated) — and
read from the committed bytes of both compiler legs.

## The anchors

| anchor | where | what it pins |
| --- | --- | --- |
| `IO.readAll` (patrol jar + `v8` + `v8-javac8`) | `tests/recover_io_resource_finally.rs` | the whole class text, `readAll`'s own text and the loop test's in-place form; the ignored replay compiles the stripped whole-class text with both javac legs and runs the class's own driver against a real `data.txt` (read to EOF) beside the fixture's class — both answer `2/hello|world|` |
| `Probe.readAll` (both legs + a jar) | `tests/recover_loop_test_copy_store.rs` | the whole class text, byte-identical on both legs and from a jar built in-test; the ignored replay compiles the stripped text with both javac legs and runs the probe's driver beside the fixture's class — both answer `hello|world|/1105` |
| `Probe.guardPlain` | same | the guard body's own control: the same guard, loop and `if`, with no dance in the `if`, presents — so the negative beside it is the dance's |
| `NEG.liveLine` (dup-store fixture, both legs) | `tests/recover_dup_store_conditional.rs` | the same-form loop at top level presents as `while ((line = read()) != null) { … }`, written once |
| `cf06.NegativeAssignments.loopCondition` | `crates/jarde-java/tests/p3_inner_assignment.rs` | the CF-06 control presents in place (`while ((local1 = arg0.length()) > 5)`), its four siblings stay quoted |

## The negatives (refused verbatim)

| negative | the link it breaks | the refusal |
| --- | --- | --- |
| `ProbeControls.parameterTarget` | the loop test's target is a parameter | `// local 2 crosses a quoted fallback region …` at `// @bytecode 0 11 21 28` |
| `ProbeControls.guardIfFirst` | an `if`-position dance with an observable target inside the protected range | `// the local assignment condition was not completely proved` |
| `MultiCopy` (the patched `Probe`) | the copy's second consumer | `// local 1 crosses a quoted fallback region …` at `// @bytecode 0 17 27 37` |
| `NEG.shortChain` | the short-circuit chain's own proof | unchanged, verbatim |
| `CF-06`'s `ExtraCopy`/`WrongType`/`interleaved`/`exceptional` | the copy family's own controls | unchanged, verbatim |
| `IONegatives.twoNested`/`closeReturns`, `NestedDepth.fourLayer`, `LockGuardNegatives`, `LockGuardProbe`, `LoopTestValues.storeTest` | the guard/postfix families' controls | unchanged, byte-identical to HEAD (the matrix) |

## The fixture contract

`tests/fixtures/recover-loop-test-copy-store/README.md` records the sources, both compiler
commands, the byte-patched control's window and verifier evidence, the SHA-256 of every committed
class and the measured behavior of every driver. `patch_controls.py` regenerates `MultiCopy.class`
from each leg's own `Probe.class` (one byte: the loop test's `iconst_m1` → `dup`).

## What moved, and why it is the shape this slice exists for

* `IO.readAll` — the io slice's registered boundary, whose refusal that change pinned verbatim
  ("its loop test's copy-and-store dance has an observable target, which that family's purity
  criterion refuses"). It presents now, and the io test's ignored replay flipped from "the
  whole-class text must stay uncompilable" to "it compiles and answers what the class answers".
* `NEG.liveLine` and `cf06.loopCondition` — the same shape at top level. The dup-store change's own
  document registered the position as the copy family's ("the split cannot be written in front of a
  re-evaluated condition, and the assignment expression is that rule's position, not this one's"),
  so this slice is what closes it; both tests are updated (the refusal replaced by the presented
  text, the controls that stay refused kept).

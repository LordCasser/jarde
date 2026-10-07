# recover-capture-ctor-super-order — implementation evidence (2026-10-07)

Commits (this worktree, not pushed — root reviews and accepts on main):

| commit | contents |
| --- | --- |
| `6ab514ce` | `fix(ctor-order)`: the second arm in `present_prologue_first` + the argument walk, `build.rs`'s `class_methods` argument, and the `p3_patterns` generator pin moved with the criterion |
| `bfb0694b` | `test(double-brace-capture)`: both fixture families + `tests/double_brace_capture.rs` + the reader's re-measured fixture population + the corpus fingerprint's 17 new entries |
| the tip | `docs(change)`: this evidence, the recorded safety condition in the change's own spec/design, and the task ledger (the commit that carries this file — a commit cannot state its own hash) |

## What changed, in one paragraph

`crates/jarde-java/src/ctor_order.rs::present_prologue_first` — the one place the val$/`super()`
order is decided (task 1.1's answer: **not** `member_inner.rs`/`facade.rs`) — now moves the
certified synthetic-store group past a constructor call the group's fields cannot be observed
through. The pre-existing `java/lang/Object.<init>()V` arm is kept verbatim, and a second arm was
added that uses **one class's own declaration view**: a class whose method table declares nothing
but its constructors and its class initializer has no code the call can dispatch into, so the
synthetic captures (minted into this class alone — no superclass was compiled against them)
cannot be read during it. That arm also requires the call's arguments to be free of the moved
fields and of any invocation, walked as an SSA dependency closure (dataflow, not spelling). The
presentation order itself is unchanged (`super();` first, the group in its original relative
order, then the instance-block statements — decision 2's shape, the pre-existing rotation).

## The safety reading, and why it is the existing guard's

`recover-ctor-reorder-dispatch-guard`'s discipline is "move only past a call that cannot run the
code that reads the captures", proved from facts the run holds (its own class header, the
prologue's own invoke) — never by reading another class's body. The new arm keeps that discipline:
it reads the same declaration view (`Inputs::class_methods`, the sibling of the `class_fields`
headers `is_synthetic_field` already reads), and it *keeps* the `Object.<init>` arm. What it adds
is the proof for the case the guard could not name: a class that declares no code of its own
beyond the constructors cannot be observed during the call, whatever the superclass constructor
does. The dispatch fixture and its three siblings declare a method that reads the moved field
(`observe()` / `render()`), so they keep the byte order — and the two order-sensitive control
suites stayed green throughout (2/2 and 7+6, [02](02-gating-and-baseline.md), [06](06-gates.md)).

Two facts the implementation pinned down by measurement, both recorded in
[design.md](../design.md) ("实现补记") and the fixtures:

* a `getfield` of the moved field before the call (`ReadArg$1`) is **not verifiable** — JVMS
  4.10.1.9 lets `uninitializedThis` be used only for a `putfield` of the current class and for the
  `invokespecial` that initializes it; both JDKs raise `VerifyError` and this run's frame pass
  refuses the method (`ir_frame_deferred`). The literal "super argument reads the captured field"
  shape therefore has no loadable form, and its current state is the whole-constructor refusal —
  pinned as current fact, with the day it changes named;
* the argument walk's live arm is the *call* case (`CallArg$1`, verifiable, prints `c`): a
  body this run does not hold is not a proof, so the group stays where the bytes put it. javac
  never writes that shape (it hoists the call out of the constructor), so the conservative refusal
  costs nothing in real input.

## Evidence index

| file | what it records |
| --- | --- |
| [01-locate-ordering-logic.md](01-locate-ordering-logic.md) | task 1.1: the ordering logic's location, with the transcripts |
| [02-gating-and-baseline.md](02-gating-and-baseline.md) | task 1.2 + the gating experiment (baseline exit 1, the flip, the byte-identical zero-regression renders, both javacs, the control suites) |
| [03-workspace-tests-1.out](03-workspace-tests-1.out) / [03-workspace-tests-2.out](03-workspace-tests-2.out) | the full workspace suite before and after the final edits |
| [04-corpus-scan.md](04-corpus-scan.md) | task 3.2: the two two-leg corpus scans and every delta classified |
| [05-oracle-leg.out](05-oracle-leg.out) | `p3_execution_comparison -- --ignored` (the corpus-moving discipline's oracle leg) |
| [06-gates.md](06-gates.md) | tasks 3.1–3.3: the gate commands and their tails |
| [06-clippy.out](06-clippy.out) | the CI-exact clippy run |
| [scan-corpus.sh](scan-corpus.sh) / [scan-evidence.sh](scan-evidence.sh) | the two self-tested scan scripts |

The fixtures live with the code they guard:
`tests/fixtures/proved-java-structure/double-brace-capture/` (both compilation legs + `freeze.py`
+ README + `SHA256SUMS` + `run.sh`), `tests/fixtures/proved-java-structure/capture-super-arg-probes/`
(the two hand-made probes + `freeze.py` + README + `SHA256SUMS`), and the root test
`tests/double_brace_capture.rs` (four tests: the capture-form order, the zero-regression forms,
the recompiled family's compile+run, and the two probes).

## Boundaries left open

* `TH$1` (the Thread anonymous form the 2026-10-05 patrol registered) declares `run()`, so its
  companion keeps the byte order and stays uncompilable. The patrol's reading of the discriminator
  ("trailing field initialization") is not what the code decides — `TH$1` and `TH$2` differ in
  their super target (`Thread` against `Object`, through the `Runnable` interface) — and proving
  `Thread.<init>` harmless would need its body, which is not in any artifact this run holds. The
  registered anchor stays open.
* The `super(compute())`-style argument (a call, class with only a constructor) keeps the byte
  order by the conservative arm. It is not javac-shaped, so this is a cost in principle only.
* Path B (jadx's allocation-site double-brace form) remains the change's Non-Goal.

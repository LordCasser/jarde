# Postfix lvalue negative boundaries

This source-only suite accompanies the main audit one directory up. It does
not modify Rust/Cargo sources or OpenSpec tasks. Replay with:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/boundaries/run_boundaries.py
```

The script compiles the hand-written support sources with `javac --release 8
-g:none`, applies bytecode or constant-pool operand changes to a fresh copy of
`BoundaryProbe.class`, then first executes each result with
`java -Xverify:all`. It keeps a case only after that JVM verification and run
pass. For every accepted class it saves the patched bytes, their SHA-256, full
`javap -c -p`, direct runtime output, complete JADX and Jarde source, and each
generated whole-class compile/run result. It never edits decompiler output.

The CLI paths and hashes are pinned in `manifest.json`; Jarde is
`/tmp/jarde-cli-bitwise-root-after` with SHA-256
`88f2e7aa9b5b8172da02f5af5d4d2c029e72a52d9b3298b46b82774e0bd5fdfd`, and JADX
is `/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx` with SHA-256
`64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7`.
`manifest.json` records the source hashes, base class size/SHA/Code count,
constant-pool indices, each exact patch, each mutated class size/SHA/Code
count, and the complete execution summaries. No final variant was excluded;
all ten passed the JVM verifier. There are no retained invalid-class
counterexamples.

| Case | Mutation or control | Direct patched-class result | JADX whole class | Jarde whole class |
| --- | --- | --- | --- | --- |
| `field-different-member` | Read `BoundarySubBox.value`, write `BoundarySubBox.other` | old `10`; value `10`; other `11` | compile/run 0; matches | javac 1; no run |
| `field-different-owner` | Read hidden `BoundarySubBox.value`, write `BoundaryBaseBox.value` on the same subclass instance | old `10`; sub value `10`; base value `11` | compile/run 0; matches | javac 1; no run |
| `array-different-index` | Spill array/index/value, write incremented value to index + 1 | old `30`; values `30,31` | compile/run 0; matches | javac 1; no run |
| `array-different-array` | Spill array/index/value, write to `otherValues` | old `30`; values `30,40`; other values `31,60` | compile/run 0; matches | javac 1; no run |
| `field-extra-consumer` | Duplicate old result and pass one copy to `observe(int)` before returning the other | old `10`; field `11`; observed `10` | compile/run 0; matches | javac 1; no run |
| `array-extra-consumer` | Same extra old-result consumer for the array element | old `30`; values `31,40`; observed `30` | compile/run 0; matches | javac 1; no run |
| `field-gap-effect` | Call observable `tick()` after `receiver()` and before `dup` | trace `RT` | compile/run 0; matches | javac 1; no run |
| `array-gap-effect` | Call `tick()` after `array()` and `index()` but before `dup2` | trace `AIT` | compile/run 0; matches | javac 1; no run |
| `control-prefix` | Unpatched `return ++receiver().value` control | return and stored value `11` | compile/run 0; matches | javac 1; no run |
| `control-assignment` | Unpatched ordinary field assignment returning the assigned value | return and stored value `99` | compile/run 0; matches | javac 1; no run |

The Jarde runs intentionally compile each complete generated class, so the
controls are blocked by other postfix methods in the same class. Their source
and javac refusal are saved as emitted; no method was repaired to isolate a
passing result. These cases therefore make no Jarde runtime claim.

The two gap cases establish an evaluation-order boundary independently of
whether a copied old value has one later consumer. The receiver has already
run before `tick()` (`RT`), and the array expression and index have both run
before `tick()` (`AIT`); moving either producer into a later `return x++`
position would change the observed order. A recovery rule must prove the
existing final-evaluation location and order, rather than treating unique
value consumption alone as permission to move a producer.

`src/` contains the self-written boundary classes and runner. `build/` keeps
the base-source javac status and output. Each case folder under `cases/`
contains the exact class and its original/JADX/Jarde evidence;
`run_boundaries.py` contains the verifier-safe Code and constant-pool patch
logic.

# Interface initializer rejection boundaries

These self-written fixtures exercise the all-or-nothing proof boundary for
Java 8 ordinary-interface field initialization. `run_boundaries.py` compiles
each source fixture with `javac --release 8 -g:none`, then (where specified)
applies a deterministic classfile-only mutation. It saves the resulting
`BoundaryProbe.class`, complete original source set, full JADX and Jarde class
source text, compiler output, `javap -v -p -c`, SHA-256, and the actual
`java -Xverify:all` runtime output. The patched classes are local fixtures;
the script neither downloads nor runs external code.

Replay from the repository root with the accepted, frozen CLI:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/boundaries/run_boundaries.py \
  --cli /tmp/jarde-cli-member-root-accepted
```

The CLI SHA-256 is `19ede7a6fe88b95530637e9765c7ae02b9f233520dc6a2e2e815f0dca58d0552`.
The captured toolchain used `javac 23.0.1` targeting release 8 and JADX 1.5.6.
`summary.json` contains the exact post-patch `BoundaryProbe.class` hashes and
statuses. A `*-javac.log` with errors means the decompiler output was honestly
attempted as a whole class and rejected; no source was edited to make it pass.
Root corrected the replay helper to place the source-only support classes in JADX's
generated `defpackage`; the compiler errors still come from the generated interface
declaration or body. Root then replayed into
`/tmp/jarde-interface-boundaries-root-GPopdI` and compared both `summary.json`
files byte-for-byte.

## Cases

| Case | Mutation or source boundary | Verified original class observation | Source-level consequence |
| --- | --- | --- | --- |
| `extra-effect` | Add `invokestatic BoundaryEffects.independent()V` at BCI 16 after all field writes and before return. The helper records `BoundaryProbe.SECOND`. | `-Xverify:all` passes; output is `ABXB\|A\|B`. | The independent effect runs after both writes and observes `SECOND == "B"`. Folding it into either initializer runs it before that initializer's `putstatic`; it observes the default `null`, changing the trace to include `Xnull`. A Java interface has no standalone static initializer statement to spell this effect. Keep the whole group unprojected. |
| `duplicate-write` | Retarget the `FIRST` `putstatic` at BCI 5 to `SECOND`; `FIRST` is unwritten and `SECOND` is written twice. | `-Xverify:all` passes; output is `AB\|null\|B`. | A complete declaration group cannot assign one source initializer to each field and preserve this path: adding an initializer for `FIRST` invents a write, while retaining both `SECOND` writes violates the single initializer-per-field projection. Reject the group. |
| `branch` | No patch: the legal Java source uses a runtime conditional expression directly in `FIRST`'s initializer, producing a branch and merge in `<clinit>`. | `-Xverify:all` passes; output is `CAB\|A\|B`. `javap.txt` shows the branch offsets. | A branch is not inherently impossible to express as a field initializer: this source already is one. But a projector that only proves a unique straight-line write cannot claim that proof here. This is a boundary for that proof rule, not a claim that all branching source must be rejected forever. |

For the two mutated classes, `original-source/BoundaryProbe.java` is the exact
legal source that `javac` compiled before the controlled classfile mutation;
it necessarily cannot spell the mutated bytecode behavior. The recorded
`original-runtime.txt` is from the mutated class, whose bytes and checksum are
saved alongside it. JADX and Jarde both emitted full class source for the
mutated input, and both complete-source Java 8 compilation attempts failed.
Those failures are recorded outcomes, not evidence that the patched class
itself is invalid: `javap` parses it and `java -Xverify:all` loads and runs it.

The third case is unpatched source and class. Both decompilers' generated
sources and Java 8 compiler diagnostics are preserved as-is. The original
class is the validation oracle; no runtime result is reported for a generated
source whose compilation failed.

## Verification limits

The first two are deliberate classfile states not expressible by one Java
interface field initializer per declaration. They prove that class validity
alone is insufficient: projection must account for extra effects and exact
write multiplicity. They do not test malformed or unverifiable input. The
branch fixture shows why a control-flow edge needs an explicit accepted
equivalence rule or a precise refusal; it does not establish a general
source-equivalence rule for arbitrary branches. Exception-handler edges and
`ConstantValue` field-table placement remain untested by this small set.

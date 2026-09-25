# Enum helper, constructor, and constant-prefix boundaries

This is a separate, reproducible audit under the enum declaration evidence directory. It leaves the
existing `Measure` and `refusal-boundaries` results untouched. `run_audit.py` uses only the frozen
Jarde CLI; it does not invoke Cargo.

Run it from the repository root or from another working directory:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/enum-declaration/helper-constructor-prefix-boundaries/run_audit.py
```

Set `JARDE_CLI` to another copy only when replaying on a separate host. The checked CLI is
`/tmp/jarde-cli-static-root-after`, SHA-256
`8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44`. The replay records full
`javap -v -c -p` output for every input class, whole JADX and Jarde source for `Measure`,
`BoundarySupport`, and `BoundaryRunner`, compiler/runtime logs, class hashes, and a machine-readable
status summary beneath `generated/`.

## Inputs and observations

`Measure.java`, `BoundarySupport.java`, and `BoundaryRunner.java` compile with javac 23.0.1 using
`--release 8 -g:none`. The baseline runs with `java -Xverify:all` and prints:

```text
values=LOW:0:2,HIGH:1:5
lookup=LOW:LOW,HIGH:HIGH,MISSING:IllegalArgumentException
sum=7:7
helper-calls=0
```

Two verifier-safe patches start from the baseline `Measure.class`. The patcher asserts that class
major version is 52, field and method table entries (flags, names, descriptors, and order) are
identical, and every changed byte is inside the target method's existing Code array. The shared
member-table digest is
`eacfcf2b849335d8d08be730d84d701ba8e3ebd15f2bbe3d8d51dfd8ca81b002`; individual input hashes are
in `class-sha256.txt` and the patch proof is in `patch-manifest.json`.

| Mode | Bytecode evidence | Verified behavior |
| --- | --- | --- |
| `values-order` | `$values()[LMeasure;` changes only the `getstatic` operands at BCIs 6 and 12, swapping `LOW` and `HIGH`. | `values=HIGH:1:5,LOW:0:2`; names and ordinals remain attached to their original constants. |
| `valueof-null-argument` | `valueOf(String)` changes only BCI 2 from `aload_0` to `aconst_null`. | Every `valueOf` call throws `NullPointerException`, including known names. |
| `constructor-shape` | Source-compiled `<init>(String,int,int)` calls `BoundarySupport.constructorEffect()` at BCI 11 after storing `units`. | Constants retain their units and order; the helper count is 2. The constructor includes an observable user effect beyond the simple field-store shape. |
| `prefix-effect` | Source-compiled `<clinit>` calls `BoundarySupport.prefixValue(int)` at BCIs 8 and 25 while evaluating each constant argument, before the corresponding enum constructor call. | Constants retain their units and order; the helper count is 2. The constant-initialization sequence contains user effects before `$values()` is called. |

The first two rows are one-method class-file mutations made after their baseline input compiles with
`javac --release 8`; the exact patched bytes then pass `java -Xverify:all`. A Java compiler cannot
produce those deliberately patched states from the baseline source, so the script records source
compilation and exact patched-class verification as separate steps. The latter two rows are
independent javac-produced Java 8 classes. The baseline and both source variants compile from source,
and all five exact inputs run with `java -Xverify:all`. The source variants and class inputs are
hashed in `source-sha256.txt` and `class-sha256.txt`.

## Decompiler results and boundary meaning

JADX 1.5.6 decompiles all five complete class sets; each full JADX source set compiles under
`--release 8` and its runner passes `-Xverify:all`. Its source matches the baseline, constructor, and
prefix-effect runs. The two patched helper cases compile but do not preserve the input behavior:

| Mode | Patched JVM output | JADX-generated output |
| --- | --- | --- |
| `values-order` | `values=HIGH:1:5,LOW:0:2` | `values=HIGH:0:5,LOW:1:2` |
| `valueof-null-argument` | `lookup=LOW:NullPointerException,HIGH:NullPointerException,MISSING:NullPointerException` | `lookup=LOW:LOW,HIGH:HIGH,MISSING:IllegalArgumentException` |

For the first patch, JADX spells constants in `$values()` array order, and Java then assigns new
ordinals from that declaration order. For the second, it presents the bytecode method as the
compiler-generated enum `valueOf` instead of retaining the patched null argument. Both are concrete
semantic gaps even though the emitted Java compiles and runs.

Jarde produces full class-source text and JSON reports for all three classes in each mode. Each
Jarde source set fails javac at the enum's ordinary field presentation
(`public static final Measure LOW;`), and runtime is therefore recorded as not attempted. These
results are evidence about the current source boundary, not a claim that a compilable enum
declaration was recovered.

The helper mutations test facts an enum declaration would need to prove: the values helper's array
order can disagree with field-table order, and `valueOf(String)` can stop forwarding its argument.
In the value-order case, Jarde still leaves `$values()` without a recovered statement, so the output
does not state the changed sequence. In the value-of case, method recovery does preserve the patched
null argument as `(java.lang.String) null`, although the complete enum text remains uncompilable.

The constructor and prefix variants delimit how much code a future enum-constant recognizer may
absorb. `Measure.<init>` contains a user-visible helper call; the prefix variant's `<clinit>` contains
two user calls before the synthetic `$values()` setup. Their recovered Jarde text currently carries
the constructor call and renders the prefix calls as constant arguments, while still declaring enum
constants as ordinary fields. Do not interpret compilable JADX output or preserved Jarde method
text as proof that the enum source layer recognizes or safely rejects these forms.

`generated/summary.json` gives the exact javac, JVM-verifier, JADX, and Jarde exit status for each
mode. In particular, `125` means execution was not attempted because the Jarde sources did not
compile; it is not a JVM failure. The boundary classes all ran successfully under full verification,
and no patch changed a field or method table entry.

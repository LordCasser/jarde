# Compound lvalue update fixture

`CompoundProbe.java`, `CompoundBox.java`, and `CompoundRunner.java` are byte-for-byte copies of
the source-only `write-only/` audit. They form the permanent whole-class pre-implementation
fixture. The separate `CompoundBoundaryProbe` covers verifier-safe rejection boundaries and the
ordering cases the proposed `+=` recovery must preserve. Generated Jarde methods and execution
results are stored unedited in the linked OpenSpec evidence directory.

From the repository root, rebuild and replay the fixture with:

```sh
javac -Xlint:-options --release 8 -g:none -d tests/fixtures/p3-compound-lvalue-updates/v8 \
  tests/fixtures/p3-compound-lvalue-updates/CompoundProbe.java \
  tests/fixtures/p3-compound-lvalue-updates/CompoundBox.java \
  tests/fixtures/p3-compound-lvalue-updates/CompoundRunner.java
javac -Xlint:-options --release 8 -g:none -d tests/fixtures/p3-compound-lvalue-updates/boundaries/v8 \
  tests/fixtures/p3-compound-lvalue-updates/BoundaryBox.java \
  tests/fixtures/p3-compound-lvalue-updates/CompoundBoundaryProbe.java \
  tests/fixtures/p3-compound-lvalue-updates/CompoundBoundaryRunner.java
python3 openspec/evidence/java-syntax-2026-09-22/compound-assignments/permanent-fixture-replay/replay_fixtures.py
```

The replay compiles sources in a temporary directory and checks them against the committed class
bytes. It verifies every original and patched boundary input with `java -Xverify:all`, then runs
JADX and the frozen Jarde CLI (`/tmp/jarde-cli-null-root-final`, SHA-256
`30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c`) on complete classes. It
records compiler/decompiler status, stdout/stderr, whole generated source, `javap -c -p`, and each
execution under
`openspec/evidence/java-syntax-2026-09-22/compound-assignments/permanent-fixture-replay/evidence/`;
`summary.json` contains the comparison results. The script checks that the frozen CLI hash is
unchanged before and after the replay.

## Positive whole-class case

The copied source SHA-256 values are:

| File | SHA-256 |
| --- | --- |
| `CompoundProbe.java` | `18d257a39d8c749183cdcc8916ff7049569b00a17a41e5f3eefa4f3f4dd660f1` |
| `CompoundBox.java` | `053b77534a272c75c8db2f28c7ac9f4c85a76772bb4bce778022c4d56310d1f4` |
| `CompoundRunner.java` | `da91ad8f75e97729a0280aae0d3eb1543e5feb46ec56e6d1a6003537fd139cdb` |

The input `v8/CompoundProbe.class` is 1113 bytes, has 12 `Code` methods, and has SHA-256
`889f36d3a5c06951d32762c8914829c059d8b1d6692001c08df73404c92f673f`. The helper and runner
class hashes are recorded here too: `CompoundBox.class` is 168 bytes with SHA-256
`48871fae8c673361b4201c3484a08e8acedc894b4cf680bd93804f00ca3b11da`; `CompoundRunner.class` is
1446 bytes with SHA-256 `4dd4320b8e36f27482ae4161f713c92b4a627b3dee1dd6bae147f3a599366508`.

Original and JADX both compile and pass `-Xverify:all`, producing these seven lines:

```text
local=8:rhs=1
field=9:select=1:rhs=1
array=14:select=1:rhs=1
field-snapshot=9:select=1:rhs=1
array-snapshot=14:select=1:rhs=1
null=NPE:select=1:rhs=0
bounds=AIOOBE:select=1:rhs=0
```

Jarde also compiles and verifies the complete generated class, but its output differs on six of
the seven lines: the local `+=` is correct; field/array updates leave 7/10 and skip the RHS; the
RHS-mutation snapshots also leave 7/10 and skip the mutation; null receiver and out-of-bounds
array cases return without throwing. The unedited Jarde and JADX source, reports, compiler logs,
and executions are in the evidence directory linked above.

## Boundary inputs and dataflow

The boundary Java sources compile to `boundaries/v8/CompoundBoundaryProbe.class` (2055 bytes,
24 `Code` methods, SHA-256
`d5acfc8babad92a18357a4fbf0e38657e374d384b15c4ffe365f8cdedc7422ed`). Source hashes, helper
class hashes, generated variant sizes, and each patched class SHA-256 are in
`boundaries/manifest.json`. The manifest holds the five original identity/consumer patches plus
six later verifier-safe effect-gap patches. `BoundaryBox.class` is 219 bytes with SHA-256
`3b7d1f963f27b4307c368ceb8b572ef805c1692b4796157993f6564581a07cd9`; `CompoundBoundaryRunner.class`
is 3156 bytes with SHA-256
`71da681a56aa34bd02a9e47b93d61fa0097a108a4a7eeff0c8bb5c1b454d015e`. The patcher edits only the
named method's `Code` bytes or one `putfield` constant-pool operand; it leaves the remaining class
bytes intact. All eleven patched classes load and execute under `-Xverify:all`. The frozen
pre-change replay covers the original five patched identity/consumer cases and eight source
boundaries. The separate
[`post-fix replay`](../../../openspec/evidence/java-syntax-2026-09-22/compound-assignments/post-fix-fixture-replay/README.md)
also covers the six added effect gaps; it records original/JADX/Jarde compilation and execution
without changing the frozen evidence.

The boundary source SHA-256 values are `BoundaryBox.java`
`3438080f500c0bc6763d6d180b8c9fabec3fe443d3ceb3546594f073d2c6a234`,
`CompoundBoundaryProbe.java`
`3410f50b95426ca1ea69dd0391fec01272b1a92b6a97fb1e420f8c9f0e0867e4`, and
`CompoundBoundaryRunner.java`
`5f2347b9871ca18d1aabe60736617f95001671d15178333ea63cf9a933cf4495`.

The CLI does not expose internal SSA value IDs. The following stack-derived identities and BCIs
therefore document the proof boundary without pretending they are Jarde's internal SSA names:

| Case | Bytecode evidence | Original result under `-Xverify:all` | Jarde pre-change result |
| --- | --- | --- | --- |
| Different field member | `receiver@0 → R0; dup@3`; `getfield BoundaryBox.value:I@4` consumes one `R0`; `putfield BoundaryBox.other:I@12` consumes the retained `R0` | `field=7:other=9:select=1:rhs=1` | Compiles/runs but `field=7:other=0:select=1:rhs=0` |
| Different index copy | `data@0 → A0; index@3 → I0; dup2@6`; upper copy becomes `I0+1` at `iadd@8`; `iaload@9` reads that pair; `iastore@15` writes lower `(A0,I0)` | `data0=24:data1=20:select=1:rhs=1` | Compiles/runs but leaves data unchanged and skips RHS |
| Different array copy | `dup2@6` retains `(data,I0)` for store; `pop2@7; getstatic otherData@[I@8; iconst_1@11; iaload@12`; `iastore@18` still writes retained `data,I0` | `data0=44:other0=30:select=1:rhs=1` | Compiles/runs but leaves data unchanged and skips RHS |
| Extra field-copy consumer | `receiver@0; dup@3; dup@4; observeReceiver@5; getfield@8; rhs@12; iadd@15; putfield@16` | `value=9:extra=1` | Compiles/runs but leaves `value=7` and skips observer/RHS |
| Extra array-copy consumer | `data@0; index@3; dup2@6; dup2@7; observeElement@8; iaload@11; rhs@13; iadd@16; iastore@17` | `data0=14:extra=1` | Compiles/runs but leaves `data0=10` and skips observer/RHS |
| Ordinary field assignment | `receiver@0; rhs@4; putfield value:I@7`; no old-value read/copy | `value=2:select=1:rhs=1` | Same as original |
| Ordinary array assignment | `data@0; index@3; rhs@7; iastore@10`; no `dup2`/`iaload` | `data0=4:select=1:rhs=1` | Same as original |
| `long` field boundary | `dup@3; getfield wide:J@4; longRhs@7; ladd@10; putfield@11` | `wide=72:select=1:rhs=1` | Compiles/runs but leaves 70 and skips RHS |
| `long[]` boundary | `dup2@6; laload@7; longRhs@8; ladd@11; lastore@12` | `long0=52:select=1:rhs=1` | Compiles/runs but leaves 50 and skips RHS |
| Array null check | `data@0; index@3; dup2@6; iaload@7` throws before RHS at `rhs@9` | `null-array=NPE:select=1:rhs=0` | Returns without throwing |
| Array bounds check | Same `iaload@7` precedes `rhs@9`; run with length 1 and index 1 | `bounds=AIOOBE:select=1:rhs=0` | Returns without throwing |
| Field old-value snapshot | `getfield value:I@4` precedes `rhsFieldMutation@7`; RHS writes 100; `iadd@10; putfield@11` uses saved 7 | `value=9:select=1:rhs=1` | Leaves 7 and skips mutation/RHS |
| Array old-value snapshot | `iaload@7` precedes `rhsArrayMutation@8`; RHS writes 100; `iadd@11; iastore@12` uses saved 10 | `data0=14:select=1:rhs=1` | Leaves 10 and skips mutation/RHS |

The array-null and array-bounds tests are distinct. Both evaluate the index once and prove that
the array read/check precedes the RHS. The snapshot cases prove the old value is captured before
the RHS mutates the same field or element.

The five same-shaped, mismatched-identity and extra-consumer cases are legal class files made by
`openspec/evidence/java-syntax-2026-09-22/compound-assignments/permanent-fixture-replay/patch_boundaries.py`:
it changes the field reference, inserts stack-valid index/array code, or
adds an observer consumer. The complete original source fixture remains javac-produced. Ordinary
`=` is source-generated and remains a positive check for the existing assignment path. This audit
tested only the rows above; it does not claim coverage of every alias topology, other primitive
widths, static fields, other compound operators, cross-block updates, or every invalid chain.

The six added gap classes reuse `fieldSnapshot()` and `arraySnapshot()` and insert the observable
sequence `rhs(0); pop` while an lvalue or update value remains on the operand stack: before field
`dup`, between `dup` and `getfield`, between `iadd` and `putfield`, between array and index
evaluation, between `dup2` and `iaload`, or between `iadd` and `iastore`. Their exact patched
class hashes are in the manifest. The original output for each reports `rhs=2`; JADX matches it.
Current Jarde must not promote these variants to `+=`, because that would move or drop an effect.
They supplement the earlier 13 replay modes and do not extend the claim to other alias topologies.

For future implementation acceptance, a boundary may remain conservatively represented by its
physical-source references. For the six effect-gap variants, that is a partial representation, not
a full refusal or evidence of runtime equivalence: current Jarde output can compile and run while
changing the original result. The evidence here verifies absence of an unsupported `+=` and retention
of source anchors only. The in-scope `int` snapshot, single evaluation, null/bounds order, and effect
tests must match the original class; `long` and the other out-of-scope cases remain refusal
boundaries.

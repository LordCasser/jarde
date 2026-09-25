# Static qualifier boundaries

This is a source-only audit. It adds no Rust changes and no permanent test fixture. Run it from
the repository root with `python3 openspec/evidence/java-syntax-2026-09-22/static-field-qualifier/boundary/run_boundary.py`.
The script requires `javac`, `java`, `javap`, `jadx`, and the frozen CLI at
`/tmp/jarde-cli-null-root-final`; it checks the CLI SHA-256 before and after the replay. It writes
the compiled class files, generated source, `summary.json`, and each command's `.stdout`, `.stderr`,
and `.status` beside this file. Compilation and execution use temporary directories and Java 8
source/target rules.

The main probe source hash is `11c6266dfd4fc7ddf659bbd43928d21f56e20a1e40a9ec3ed803282dd1f67835`.
Its class is 1,388 bytes, SHA-256
`6fb9f6a79bd78d3d1491ed7b3fea157741f8246d24667e5f81c0e4283b7c9de8`, with 16 `Code` attributes.
The frozen CLI SHA-256 is
`30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c`.

## Same bytecode, three Java spellings

The following source methods have identical instruction sequences in the compiled class:

| Source spellings | Instruction sequence |
| --- | --- |
| `return receiver().rhs();` and `receiver(); return rhs();` | `invokestatic receiver @0; pop @3; invokestatic rhs @4; ireturn @7` |
| `receiver().mark();` and `receiver(); mark();` | `invokestatic receiver @0; pop @3; invokestatic mark @4; return @7` |
| `receiver().value = rhs();`, `value = receiver().rhs();`, and `receiver(); value = rhs();` | `invokestatic receiver @0; pop @3; invokestatic rhs @4; putstatic value @7; getstatic value @10; ireturn @13` |

For this class, each spelling calls `receiver()` once, then calls the same static target once. The
three static-write spellings also store the same RHS result after the same evaluation order. A
throwing RHS still follows the one receiver call and prevents `putstatic`; a throwing receiver
prevents the RHS call. The class file does not say which spelling was used, so exact syntax cannot
be reconstructed from these instructions. The behavior can be preserved by writing a discarded
invocation as a statement and writing the static target by its owner.

The static-field read control is `invokestatic receiver @0; pop @3; getstatic value @4; ireturn @7`.
Jarde emits `receiver(); return QualifierBoundaryProbe.value;` for it, so its side effect runs once.

## Whole-class comparison

The original source, JADX output, and Jarde output all compile and run under `-Xverify:all` for the
main probe. Original and JADX produce the same 16 output lines. Jarde's class also compiles, but
eight state lines show `selects=2` where the original and JADX show `selects=1`: both qualified and
plain spellings that compile to the shared `invoke; pop; invokestatic` shape are duplicated. The
RHS call still runs once. The receiver-throws case keeps `rhs=0`, because the first receiver call
throws before the duplicated expression can execute. `summary.json` contains the complete outputs
and equality flags.

The current shape accounts for the `pop` as the following static call's qualifier in
`Builder::discarded_evaluations` (`crates/jarde-java/src/build.rs:3598`) and writes it in
`Builder::call_expr` (`:4731`). For an invocation-produced value, the producer call is also emitted
as a statement: a `pop` is not a value-rendering reader in `produced_value_reaches_a_reader`
(`:5673`). Thus the same call runs once as a statement and again as the next call's qualifier. The
static-field-write case shares those exact instructions, so the current output also invents a
qualifier on the RHS call.

The smaller repair boundary is an immediate `Invoke; pop; invokestatic` where
`call_result_is_discarded` (`:3669`) proves the invocation's result reaches only that `pop` and no
local. Reuse the existing discarded-call statement for the producer and write the next static call
using its owner (`receiver(); QualifierBoundaryProbe.rhs();`, or the existing same-class bare call).
That preserves all three source spellings' observable behavior and does not introduce a new AST
mechanism. Keep the existing qualified-call path for a popped value that cannot be a Java expression
statement, such as the `aload_0; pop; invokestatic` regression in
`tests/p3_meeting.rs::viaRef`; its source explicitly uses an instance expression as the static-call
qualifier. The `pop` must remain accounted for in either path.

## Static interface method boundary

`InterfaceBoundaryProbe.class` is 344 bytes, SHA-256
`ec0c9caa6d67c9f367549f5a7d306c2e57cb7804cb891c1ecf298d39d5904151`, with three `Code` attributes.
The legal Java 8 source writes `receiver(); return InterfaceStaticOwner.value();`; its bytecode is:

```text
0: invokestatic receiver:()LInterfaceStaticOwner;
3: pop
4: invokestatic InterfaceMethod InterfaceStaticOwner.value:()I
7: ireturn
```

Original and JADX compile and run with `value=17:selects=1`. Jarde emits
`receiver(); return receiver().value();`, and javac rejects the generated class: Java 8 interface
static methods cannot be selected through an instance expression. The separate
`IllegalInterfaceQualifier.java` check confirms that `receiver().value()` itself is rejected by
javac. For this call kind, the renderer must use the interface owner, and an invocation-produced
discard should be written separately. If the popped producer is not independently writable, the
existing refusal path should retain it rather than manufacture an illegal qualifier. The parsed
`CallTarget` already states whether its constant-pool reference is an interface reference.

## Methodref owner boundary

The owner probe is compiled from Java 8 source where `Child extends Base`, `Base.ping()` returns 31,
and `Child receiver()` is popped before `invokestatic Child.ping:()I`. javac uses `Child` as the
Methodref owner for this inherited static method; the unpatched class verifies and runs as 31.

The replay makes two equal-length constant-pool-only edits, leaving the instruction bytes and BCIs
unchanged. The first changes the Methodref owner `Child` to unrelated `Other`, which has its own
`ping()` returning 41. It changes only class-file byte offsets 130–134. The patched class verifies
and runs as 41. Jarde writes `return receiver().ping();`; this compiles, but javac resolves that
expression back to inherited `Base.ping()` and it runs as 31. JADX instead writes
`receiver(); return Other.ping();` and reproduces 41.

The second patch changes the owner from `Child` to unrelated `EvilX` and the method name from
`ping` to `pong` (both replacements preserve constant-pool UTF-8 lengths). The verifier accepts the
class and it runs `EvilX.pong()` as 51. Jarde writes `return receiver().pong();`, which javac rejects
because `Child` has no `pong()` member. JADX writes `receiver(); return EvilX.pong();` and runs as
51. The first patch proves owner mismatch can silently rebind a compilable expression; the second
shows an unresolvable member can make the expression invalid. This is a valid JVM bytecode boundary,
not a source spelling javac could produce. When a producer call can be emitted separately, using
the Methodref owner avoids both outcomes. For a non-statement producer, only retain expression
qualification when the owner/member is representable through its stated reference type; otherwise
quote the shape.

The patched class files, source inputs, complete `javap -c -p` outputs, Jarde/JADX text, compiler
diagnostics, and hashes are in this directory. `summary.json` records each patch's exact changed
offsets, Methodref, `-Xverify:all` result, and recompiled output.

The root agent independently copied this directory to a temporary location and reran
`run_boundary.py` against the frozen CLI. The replay exited 0, reproduced the class hash and all
four identical-Code pair checks, found original/JADX equal and Jarde unequal, verified the
interface result `value=17:selects=1`, and reproduced patched JVM outputs 41 and 51. The frozen
CLI hash was unchanged before and after this replay.

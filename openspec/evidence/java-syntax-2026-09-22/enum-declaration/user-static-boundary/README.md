# Enum boundary with user-defined static initialization

`Measure.java` is a small Java 8 enum with two constructor-argument constants, the instance field that stores each argument, a user static field, an explicit user static initializer block, and one user static method. `MeasureRunner.java` checks enum order and `valueOf`, constructor-backed fields, the static total, and the custom method. The original source compiles with javac 23.0.1 under `--release 8 -g:none`, and the runner passes `java -Xverify:all` (`original-run.txt`).

JADX 1.5.6 decompiles the enum and runner as a whole. Its complete source set compiles as Java 8 and passes the same verified runner (`jadx-javac.log`, `jadx-run.txt`). Jarde's complete source set and JSON report are preserved. Its javac attempt reports one error: `LOW` is written as an ordinary `public static final Measure LOW;` field where an enum constant must appear. The Jarde runtime attempt is preserved and cannot load the runner because the generated classes did not compile.

The source emits two explicit enum methods with code: the constructor and `sumUnits`; the initializer block is compiled into `<clinit>`. `javap-Measure.txt` shows how user and compiler work share that initializer. `<clinit>` constructs `LOW` then `HIGH` with JVM name, ordinal, and source argument; builds the synthetic `$VALUES` array by calling `$values()`; then calls the user method `sumUnits()` and stores the result in user field `totalUnits`. `sumUnits()` itself reads both constants' `units` fields. The runner observes the intended order and the value `7`, so preserving only the class file's final constants while dropping the final user assignment would lose source behavior.

Jarde already preserves the user `totalUnits` field, the body of `sumUnits()`, and the user assignment `Measure.totalUnits = sumUnits()` at the end of its emitted static block. It also writes the compiler-generated constant assignments into that same block. At present it does not separate those responsibilities: it declares `LOW` and `HIGH` as ordinary fields, emits `$VALUES`, emits `values()` and `valueOf(String)`, exposes the JVM-only `String`/ordinal constructor parameters, and emits `$values()` with an unrecovered body. Those forms are useful evidence of what the class file contains, but they are not a legal Java enum source declaration.

## Architecture observation

This probe supports an enum-aware source structure, with no new reader production indicated. The reader already supplies the class `ACC_ENUM` flag, the constant fields' `ACC_ENUM` flags and field-table order, descriptors, method headers, and `<clinit>` Code. The existing source layer recognizes the enum class header, and ordinary recovery already handles the user method and final user initialization action in this sample.

The source structure needs to identify enum constants, extract their source constructor arguments from the compiler-generated prefix of `<clinit>`, and separate that prefix plus `$VALUES` setup from the user-defined initialization that follows. It should then suppress compiler-owned `$VALUES`, `$values()`, `values()`, and `valueOf(String)` machinery; remove the synthetic name and ordinal from the source constructor signature; and retain user declarations and logic such as `totalUnits` and `sumUnits()`. The enum constant ordering in this sample is stated by both field order and the ordered construction/assignment sequence, while `<clinit>` is the evidence that connects each constant to the source argument. More complex initializer control flow or constants that reference earlier constants are outside this minimal probe.

`class-sha256.txt` freezes both Java 8 input classes, and `javap-Measure.txt` / `javap-MeasureRunner.txt` preserve `javap -v -c -p`, including private constructor and helper. `jarde-cli-sha256.txt` records `/tmp/jarde-cli-static-root-after` at SHA-256 `8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44`. Full Jarde source, JSON, compiler logs, and runtime output remain unchanged for review.

## Replay

Run `python3 run_audit.py` from this directory. The script uses its own directory for source and evidence files, and honors `JARDE_CLI` if the CLI is stored elsewhere. It was replayed successfully from a copied directory. It recreates the class files, hashes, `javap`, JADX output, Jarde text/JSON, and compile/runtime logs. Required tools: `javac`, `java`, `javap`, and JADX 1.5.6.

The root agent independently replayed a copy in `/tmp/jarde-enum-user-root-tIsfII` on 2026-09-23. Both class hashes, original and JADX runner output, and Jarde enum text matched byte-for-byte. The capture was then expanded from `javap -v -c` to `-v -c -p` and regenerated without changing the class hashes or decompiled output.

## Known limitation: the proved shape is name-bound (recorded 2026-10-04, root audit)

The recovery this directory records is implemented by `prove_static_assignment_suffix`
(`src/enum_constants.rs`), whose doc comment states it proves "the one static assignment suffix
frozen by the Measure/Counted fixtures" and that "the fixed source constants and BCI shape
deliberately keep this first slice narrow".

Concretely, the production code requires the **specific user names** from this fixture:

| Site | Requirement |
| --- | --- |
| `enum_constants.rs:711`, `:735` | a field named exactly `totalUnits` (scan and uniqueness check) |
| `enum_constants.rs:785`, `:806` | a method named exactly `sumUnits` with descriptor `()I` |
| `enum_constants.rs:832–845` | the **expected bytecode suffix is built from those names** — a `MethodRef` `sumUnits:()I` and a `Fieldref` `totalUnits:I` are constructed as literals, then the instruction suffix `[call, store, ret]` is matched against them. So the names are baked into the BCI-shape match itself, not merely filtered |
| `src/class_source.rs:8309` | the same `totalUnits` name in the projection gate |

Any enum whose user static field or static method carries a different name is not matched by
this proof. Per the same doc comment, such a suffix "remains in the original `<clinit>`
presentation, along with every physical enum member; it is never trimmed from emitted text" —
i.e. the fallback keeps the physical presentation rather than emitting a partly-projected
`<clinit>`. **Root has not independently measured a differently-named fixture** (this note was
written during a zero-build audit while another slice held the build lock), so the loudness of
that fallback is sourced from the code's own documented contract, not from a replay. It is
recorded here as an accepted narrow-first-slice boundary rather than a correctness defect; if a
future slice touches this path, measuring a non-`totalUnits` fixture should be part of its
forensics.

This limitation was **not** recorded here or in the inventory ledger when the slice landed
(commit `2ad29cee`); it lived only in the code comment. Root's hardcoded-identifier audit
(2026-10-04) surfaced it while investigating a genuinely defective case of the same pattern —
`recover-proved-string-arg-enum-constant-bodies`, whose spec claimed a general capability while
its implementation hardcoded the field name `op` (exactly its own acceptance anchor's name).
See [hardcoded-identifier-audit](../../../java-syntax-2026-10-04/hardcoded-identifier-audit/README.md)
for the full audit and the distinction between the two cases, and
[recover-enum-string-field-name-generalization](../../../../changes/recover-enum-string-field-name-generalization/proposal.md)
for the filed fix.

If this slice is ever generalized, the proved names must be threaded from the bytecode to the
emission site (as the `op` fix does) rather than hardcoded, and a frozen positive fixture using
a **different** field/method name must be added — otherwise the acceptance anchor alone cannot
distinguish a general capability from a name-bound one.

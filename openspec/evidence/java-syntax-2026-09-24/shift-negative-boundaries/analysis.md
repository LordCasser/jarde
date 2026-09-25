# Shift negative boundaries

This evidence completes task 1.2 for `recover-shift-expressions`. The baseline is the frozen Java 8
`ShiftSlice.class` from `shift-root-core-slice`; its SHA-256 is
`1036af7d36484406373dc63259fb2bf3232cd5fcf289b22b488815eac90a438a`.

## JVM-verifiable in-memory variants

`VerifyShiftVariants.java` parses the constant pool and changes one existing, equal-length UTF-8
descriptor in a cloned byte array. It defines each clone through a fresh class loader, resolves its
methods, and invokes the changed signature. Nothing writes the changed class bytes to disk. All four
classes below loaded with `java -Xverify:all`:

Root independently recompiled the harness and reran it with `java -Xverify:all`; the four class
hashes and `4`/`6` results matched the table. The boolean-consumer invocation returned `false` on
this JVM; it remains a valid verifier case but has no legal Java source `return intShift;` in a
`boolean` method, so that observed boolean value is not used to justify an invented conversion.

| Class bytes | Edit | SHA-256 | Result |
| --- | --- | --- | --- |
| Original | None | `1036af7d36484406373dc63259fb2bf3232cd5fcf289b22b488815eac90a438a` | Verified |
| Boolean left | `(SI)I` → `(ZI)I` | `119e9964b54061fcfbf573182c553ee316df28e167d47e92087b7284ff9ada86` | `shortLeft(true, 2)` returned `4` |
| Boolean distance | `(II)I` → `(IZ)I` | `91f3ad82934a66ca18ccf27c96b98c077b80bc4203240521500f99f184522d45` | `intLeft(3, true)` returned `6` |
| Boolean consumer | `(II)I` → `(II)Z` | `f709ec0c587a9d38a643ad6a97e47ea6bdfb4513add95a99715287de145fc177` | Verified; its `ireturn` is an int-like stack type consumed by a `Z` descriptor |

These are valid negative Java-recovery examples, not invalid class files: the JVM verifier groups
`boolean` and `int` in the same integer verification type. Jarde's descriptor-boundary test refuses
the boolean left and distance variants instead of writing an ill-typed Java shift. The boolean
consumer variant is refused at the return consumer: `intLeft(II)Z` quotes `@bytecode 3`, and its
all-evidence source map retains BCI 3, the original `ireturn` position. No verifier-invalid bytes
were used as evidence or presented as a valid negative example.

## Old local and its source boundary

`ShiftLocalBoundary.java` is source-only. The targeted Rust test compiles it with
`javac --release 8 -g:none` into a temporary directory, passes those bytes through Jarde, and removes
the directory at test exit. Its generated class SHA-256 on this run was
`670e1a4cfd6953508f043cb5f3a7fcd225205a0a07201d855855b5a1d6931d59`. `javap -c -p` shows
`istore_2` at BCI 1, the overwrite of `arg0` ending at BCI 5, and `ishl` at BCI 8. Jarde presents
`int local2 = arg0; arg0 = arg0 + 1; return local2 << arg1;`; it does not substitute the rewritten
argument. The all-evidence test confirms BCI 8 remains mapped. A separate `-Xverify:all` runner
executes `savedBeforeOverwrite(7, 2)` and gets `28`, confirming the saved value is what the shift
uses.

Root independently rebuilt the source-only class to the same SHA-256
`670e1a4cfd6953508f043cb5f3a7fcd225205a0a07201d855855b5a1d6931d59`, confirmed
`ishl` at BCI 8, then used the final CLI SHA-256
`9467c73083d721a51abae84455175980bb7a2f2b38ed29c12c7f073d3672f6a9` to recover its
complete class without a quote. The unmodified Jarde Java class compiled with `javac --release 8`
and `-Xverify:all` printed the same `savedBeforeOverwrite=28` as the original class. The three
descriptor variants were also replayed through this CLI: `shortLeft(ZI)I` and `intLeft(IZ)I`
retained a refusal, while `intLeft(II)Z` kept `@bytecode 3` at the return consumer.

## Reproduction

From the repository root:

```sh
work_dir=$(mktemp -d /tmp/shift-negative-final.XXXXXX)
javac --release 8 -g:none -d "$work_dir" \
  openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/source/ShiftSlice.java \
  openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/source/ShiftSliceHelper.java \
  openspec/evidence/java-syntax-2026-09-24/shift-negative-boundaries/VerifyShiftVariants.java \
  openspec/evidence/java-syntax-2026-09-24/shift-negative-boundaries/ShiftLocalBoundary.java \
  openspec/evidence/java-syntax-2026-09-24/shift-negative-boundaries/ShiftLocalBoundaryRunner.java
java -Xverify:all -cp "$work_dir" VerifyShiftVariants \
  openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/original/ShiftSlice.class
java -Xverify:all -cp "$work_dir" ShiftLocalBoundaryRunner
javap -classpath "$work_dir" -c -p ShiftLocalBoundary
shasum -a 256 \
  openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/original/ShiftSlice.class \
  "$work_dir/ShiftLocalBoundary.class"
CARGO_TARGET_DIR=/private/tmp/jarde-shift-negative-target \
  cargo test --test p3_shift_expressions --test p3_shift_negative_boundaries
cargo clean --target-dir /private/tmp/jarde-shift-negative-target
```

The two integration test binaries passed (3 shift-expression tests and 2 negative-boundary tests).
The dedicated Cargo target was cleaned after the run; no other target directory was touched.

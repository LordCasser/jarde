# Verification

## Frozen input and RED/green replay

`tests/fixtures/p3-null-resource/` keeps the Java sources and target class files for the 670-byte
null-resource core, ordinary null-plus-catch case, same-length broken-suppression mutation,
inherited-only type boundary, and body-throws case. Fixture BCI notes, source/class SHA-256 values,
and the `useNullResource()V` Code hashes are in its [README](../../../tests/fixtures/p3-null-resource/README.md).
The broken-suppression script verifies the BCI 36 call before replacing the BCI 36–38 instruction
with `pop2; nop; nop`; the class stays 670 bytes, verifies with `-Xverify:all`, and is rejected at
the suppression proof with a BCI-bearing `jre_guard_suppressed` refusal.

The pre-change root replay at
`openspec/evidence/java-syntax-2026-09-22/try-with-resources/root-replay/null-resource/audit.json`
is the RED result: original and JADX class compilation/runtime pass, while jarde's class compilation
fails. Root rebuilt after formatting and the iterator-style cleanup. The final frozen CLI SHA-256
is `30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c`, identical before
and after the replay at
`openspec/evidence/java-syntax-2026-09-22/try-with-resources/root-after-null/`.

## Guard and type boundary

The change reuses the existing TWR proof. The null candidate must be a unique direct
`aconst_null; astore` initializer and first pass the existing normal-close and exceptional-close
contour checks; then the existing full TWR proof checks ranges, suppression, and rethrow. A
same-type ordinary `RuntimeException` catch remains a user catch. The verifier-valid mutation
refuses at the broken suppression rather than emitting a resource or synthetic catch. The
inherited-only child has no direct interfaces, so it is not assigned its own class as the resource
type. These method-level outcomes are asserted in `tests/p3_null_resource.rs` and independently
summarized in `root-after-null/negatives/summary.json`.
The final negative replay uses the same frozen CLI: broken suppression receives
`jre_guard_suppressed` without a fake catch, inherited-only receives a direct-interface refusal,
and the ordinary user method retains its `RuntimeException` catch without a resource header.
The positive fixtures use default-package top-level classes. `current_class_simple_name` also
accepts spellable package components and writes the simple current-class name, but package-name
resolution is not exercised here; binary names containing `$` are conservatively refused.

The independent same-type whole-class replay found an existing adjacent limitation: original and
JADX source compile and run, but jarde whole-class `javac` fails in the handwritten
`closeWithoutSuppression()` method because its ordinary `null` local is emitted as `Object` and
then used with `.close()`. The `ordinaryNullThenCatch()` method itself is structured and retains
its real `RuntimeException` catch. The handwritten source also takes a `goto` around its normal
close and therefore is not used as the damaged-suppression guard input; that assertion uses the
same-length binary mutation instead. This is the existing general null-local type-inference limit,
outside the null-resource header change.

The ordinary-catch behavior also has an isolated 441-byte source-only replay at
`root-after-null/ordinary-only/`: original, JADX, and jarde all compile and pass `-Xverify:all`,
with the identical `ordinary-catch=1` output. The jarde result has only the real
`RuntimeException` catch and no resource header.

For the null body-throws path, the frozen CLI replay records original/jarde `javac` and
`-Xverify:all` success and identical output:
`null-resource-exception:identity=true,suppressed=0,bodyCalls=1,closes=0`. JADX does not compile
this class: its reconstructed method emits `throw th` without declaring/catching `Exception`
(`root-after-null/exceptional/jadx-javac.stderr`). For the primary full-class comparisons, the
core, null-resource, and multi-resource original/JADX/jarde outputs all compile and run under
`-Xverify:all`; their line-by-line runtime comparisons are equal in the corresponding
`root-after-null/{core,null-resource,multi-resource}/audit.json` files.

## Implementation tests

`cargo check -p jarde-java --tests` passed. The final adjacent regression command was:

```text
cargo test --test p3_null_resource --test p3_guard --test p3_typed_catch --test p3_twr_catch
```

Result: 24 passed, 1 ignored (the existing TWR catch test requiring a separate `returns`
increment). The five null-resource cases cover the typed header and source map, a thrown body
exception, ordinary catch classification plus broken suppression refusal, inherited-only type
refusal, and evidence/budget/cancellation contracts. Root independently reran those suites plus
`p3_nested_try` after the final source edit: 25 passed, one existing ignored. The wider
`jarde-java` unit and integration suites passed 109 + 32 + 50 tests.

## Root-owned acceptance

The final rebuilt-CLI replay compiled and ran original/jarde whole classes for nullable factory,
three resources, the direct null header, the null body-throws case, and the isolated ordinary
catch. Their outputs match line by line (3, 1, 1, 1, and 1 lines respectively); the three baseline
TWR cases and ordinary catch also have compiling, equal JADX outputs. JADX's exceptional-body
source does not compile, as recorded above. The mixed same-type class remains an explicit
whole-class RED only for the separate ordinary null-local type limitation. No recovered text was
edited to make a comparison compile. The null header contains no bytecode quote or synthetic
`Throwable` catch.

The reader fixture census is `(111, 726, 94, 261, 8)` after six new class inputs, and the corpus
fingerprint now pins 293 files, including 17 new null-resource source/class entries. The reader
census test and fingerprint suite pass (5 passed, 1 regeneration test intentionally ignored).
Final `cargo fmt --all -- --check`, `git diff --check`, and
`openspec validate --all --strict --no-interactive` pass (54/54). Targeted Clippy with the
pre-existing `region.rs:1736` type-complexity warning allowed passes; strict Clippy is still
blocked only by that pre-existing region warning. Root corrected a new iterator-style Clippy
warning in this change before rebuilding and replaying the final CLI.

The suppression refusal category is correct for the patched class, but the existing generic
message says its receiver differs from the stored primary even when the `addSuppressed` call is
absent. Diagnostic precision is a separate wording debt, not part of this resource header change.

# Verification

This implementation covers tasks 1.2, 2.1–2.3 and 3.1. Root-owned tasks 3.2 and 3.3 remain open for independent review, adjacent regressions, corpus fingerprinting and strict validation.

## Fixture and boundaries

- The frozen primary `ClassLiteralProbe.class` is 851 bytes with SHA-256 `11b1724cd33490629a115dfdd8104fed394f3569b03aa6ecfe9007ca1bedfc4b`, matching the evidence input. Its source and runner are byte-for-byte copies of the frozen evidence files.
- The fixture README records Java 8 compiler provenance and hashes for the primary class, local-name assignment control, current-class `String` control and current-class `java` refusal input.
- The permanent decoder tests exercise `ldc` and `ldc_w` Class entries, refuse `ldc2_w` Class entries and synthetic MethodType/MethodHandle pool facts, and refuse malformed MUTF-8, invalid ASCII type names and `$` binary names. A valid U+10400 surrogate pair decodes to its scalar, then is refused by the current ASCII source-name boundary; full Java 8 Unicode 6.2 identifier support remains separate work.
- The local `java` fixture recovers and stores `java.lang.String.class`; a class named `String` also keeps that path. The class named `java` refuses the same emitted root path and quotes both producer BCI 0 and consumer BCI 2. Its fallback source-map segment anchors BCI 2 directly and BCI 0 as a derived producer.
- A single-class run cannot prove that an unrelated same-package sibling named `java` is absent. This change does not add cross-class resolution; the limit is recorded in the design and evidence analysis.

## Recovery, evidence and runtime

- The primary fixture recovers all nine Code methods. `reference`, `array`, `primitiveArray`, `self` and `argument` each emit one class literal; `primitive` and `voidType` still use the original `Integer.TYPE` and `Void.TYPE` path.
- The literal segments preserve exact origins: `reference` BCI 0/CP 13; `array` BCI 0/CP 15; `primitiveArray` BCI 0/CP 17; `self` BCI 0/CP 8; and the argument literal BCI 0/CP 13. The consuming call at BCI 2 remains mapped to the call text. Default and all-evidence requests produce identical source. Low-budget and pre-cancelled class-source requests return incomplete outcomes.
- The ignored whole-class test compiles the original source, full JADX source and full jarde class source with Corretto 8, then executes each with `-Xverify:all`. All three produce exactly:

  ```text
  reference:java.lang.String
  array:[[Ljava.lang.String;
  primitiveArray:[[I
  self:true
  primitive:int
  void:void
  argument:java.lang.String:calls=1
  ```

  JADX is compiled before it is used as a runtime comparison. The jarde class has zero `@bytecode` markers.

## Commands

- `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test -p jarde-java` — passed (109 unit tests, plus the crate's existing 32- and 48-test integration suites).
- `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --test p3_class_literals` — passed (4 tests, 1 ignored JDK comparison).
- `PATH="/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin:$PATH" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --test p3_class_literals original_jadx_and_jarde_whole_classes_compile_and_have_equal_runtime_results -- --ignored --exact` — passed under Corretto 8.
- `rustfmt --edition 2024 --check crates/jarde-java/src/ast.rs crates/jarde-java/src/build.rs crates/jarde-java/src/decode.rs crates/jarde-java/src/emit.rs tests/p3_class_literals.rs` — passed.

## Root independent acceptance

Root built and froze `/tmp/jarde-cli-class-literals-root` (SHA-256 `25bf181ccf0d879351a16710818e27efba0df6b3851931a29eb98d9f41adb1e0`), then replayed the unedited source, JADX and jarde whole-class pipeline in `class-literals/root-after-25bf/`. The original 851-byte input retained SHA-256 `11b1724cd33490629a115dfdd8104fed394f3569b03aa6ecfe9007ca1bedfc4b`. All three compiled and ran with `java -Xverify:all`; both comparison diffs are empty over seven lines and jarde has zero `@bytecode` markers. Root also reran the ignored full-class test under actual Corretto 8. The direct literal segments carry BCI 0 and CP 13/15/17/8/13 respectively; the consuming call keeps BCI 2. The known `java` type-path collision quotes consumer BCI 2 and producer BCI 0; local `java` and current class `String` remain positive controls. Decoder unit tests cover `ldc_w`, malformed MUTF-8, other CP tags, non-ASCII and `$` refusal. Emission treats `T.class` as a primary expression with presented type `java.lang.Class`.

Root reran `cargo test -p jarde-java --locked` and the class-literal, MUTF-8, array access, invocation-argument, deferred-order and required-conversion integration targets; all applicable tests passed. The adjacent `p3_type_qualifier::static_owners_survive_generated_parameter_and_suffix_names` remains RED: a parameter is named `arg0` while a static owner is printed `arg0.pick(1)`. The pre-change frozen CLI `908c472560d6146354e1fd679c595e884afadf5ec7051727b51e79a1f95c6570` produces the same signature and call text on the same fixture, so this is the already planned `preserve-type-qualifier-bindings` gap, not a Class-literal regression. Its other two active tests pass.

The four frozen Class-literal/type-name class inputs add 15 bodies and no handlers or targets: reader census now `(104, 683, 81, 236, 8)` and passes. The corpus fingerprint initially reported exactly nine unlisted files under `p3-class-literals`, with no changed or deleted earlier file; root regenerated it and its five active checks pass (one intentional generator ignored). `cargo fmt --all -- --check`, `git diff --check` and OpenSpec strict all pass; strict reports 53/53. Disk space after the serialized Rust runs remained about 11 GiB free, with this repository's `target` about 1.6 GiB.

`assert-syntax/root-after-class-25bf/` is an adjacent limitation: this Class literal change reduces the real assertion fixture from four quotes to one and recovers `AssertProbe.class.desiredAssertionStatus()`, but `<clinit>` still cannot consume a branch Phi at `putstatic Z` BCI 13. The complete class still fails javac due its uninitialized synthetic final field; the residual is assigned to `recover-conditional-values` rather than this constant-pool change. Full Java 8 Unicode class names and ambiguous `$` binary names remain separate type-spelling work, as documented above.

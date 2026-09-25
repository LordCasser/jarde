# Array reference conversion audit

This is a source-only audit of Java 8 array reference widening at static calls. It uses the complete `ArrayReferenceConversions` sample and a `ArrayReferenceCore` sample that removes both array-write methods. The core sample keeps the three covariant calls, the `Object` and `Object[]` widened controls, exact typed methods, runtime class/length reads, empty arrays, and null cases. No partial-dimension or unsupported array-shape case is mixed into the samples.

The original class was compiled with `javac --release 8 -g:none`, then executed in a separate JVM with `java -Xverify:all`. The frozen full class is 1,765 bytes with SHA-256 `5298588750f47fdd68525c1d570b2226f5d1c00230101e572b390c782073baf2` and has 18 `Code` attributes. The core class is 1,632 bytes with SHA-256 `549e371ac98362c3ca0d0c992e7a7cb2292a90a5f9b46ac58ee468338dd9ae0d` and has 16 `Code` attributes. Both original JVM runs completed with `-Xverify:all`; the full runner emitted 37 lines and the core runner emitted 28 lines.

`original-javap.txt` shows the resolved descriptors directly:

* `typedIntMatrix(int[][])` invokes `overload(Object[])`.
* `typedStringArray(String[])` invokes `overload(Object[])`.
* `typedStringMatrix(String[][])` invokes `overload(Object[][])`.
* The `Object` and `Object[]` widened controls resolve to `overload(Object)` and `overload(Object[])` respectively.

The runtime output confirms those targets. It also records zero-length arrays, the runtime classes `[[I`, `[Ljava.lang.String;`, and `[[Ljava.lang.String;`, and the expected failures: `ArrayStoreException` when an `Object` or `Object[]` is stored through an incompatible covariant view, `ArrayIndexOutOfBoundsException` for the empty write cases, and `NullPointerException` only for dereferencing null. Passing null through the overloads that do not dereference it still selects the `Object` or `Object[]` overload as reported.

The original and JADX paths both compile and run successfully with `javac --release 8` and `java -Xverify:all`. Their complete outputs are byte-for-byte equal: full output SHA-256 `bd7a4dd0bca442a2005b4f2d447cef6b2c486a3661c4819936e490372cc35558`, core output SHA-256 `80169ee0cc3b5555eeb26df2f102c3a04a92d72d192ad364fc27bfb96cd00f27`.

`ArrayReferenceOverloadTarget` is a separate target-selection control. It has both `overload(Object[])` and the more specific `overload(String[])`. `explicitObjectTarget(String[])` uses the source expression `overload((Object[]) value)` and its `javap` output keeps the `overload:([Ljava/lang/Object;)` `Methodref`; the current JDK also emits a redundant three-byte `checkcast "[Ljava/lang/Object;"` for that explicit cast. `localObjectTarget(String[])` first assigns to an `Object[]` local and then calls `overload(widened)`, and its bytecode has `astore_1; aload_1; invokestatic overload(Object[])` with no `checkcast`. `implicitStringTarget(String[])` selects `overload(String[])`. The original and JADX target classes compile and run with `-Xverify:all`, and their nine output lines match byte-for-byte (`target-original-runtime.txt` and `target-jadx-runtime.txt` both have SHA-256 `cffd64d728ad957af2aad459361c67a8f6d4f4b3e791a7ce40117ff3525e103c`). Jarde emits a target-argument marker for the explicit call and fails its complete generated source at `javac`; its skipped runtime is recorded separately. This supplies a direct control for preserving the invocation's target static type when recovering a covariant argument.

The frozen Jarde CLI is `/tmp/jarde-cli-deferred-final-ecab`, SHA-256 `ecab8244d1709765fa0f2b0effcebe49b9f066eea911843d11d34abec5837330`. It generated six `@bytecode` markers in the full source and three in the core source. Both generated sources fail `javac --release 8` at the same three typed methods (`typedIntMatrix`, `typedStringArray`, and `typedStringMatrix`), each with a missing return statement. The Jarde comments explain that the `int[][]` to `Object[]`, `String[]` to `Object[]`, and `String[][]` to `Object[][]` invocation arguments had no safe reference-conversion evidence. The full sample has the same three failures as the write-free core, so the separate core result isolates the covariance issue from the full sample's `new Object()`/`aastore` write path. Since compilation fails, no Jarde runtime result is claimed; the runtime status files explicitly say `skipped`.

For the target-selection control, Jarde emits one `@bytecode` marker for the no-cast local path and fails `javac` there; the explicit cast and implicit `String[]` calls are represented as source. The Jarde run is recorded separately from the successful original/JADX controls. The preserved `target-patch.json`/`target-patched-*` records replace only the three-byte `checkcast` with three `nop`s. That class passes `java -Xverify:all` and produces the same nine lines as the original target class; Jarde then emits five markers and fails `javac` at both the direct cast and no-cast local `Object[]` invocations, so no patched Jarde runtime is claimed.

Run the complete audit with:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/array-reference-conversion/run_audit.py
```

The script compiles both source samples afresh, checks the full class against the frozen class bytes, runs both original JVMs, invokes the fixed CLI, compiles and runs the generated Jarde and JADX sources when their compilation succeeds, records per-command stdout/stderr/status files, and verifies that the CLI hash is unchanged.

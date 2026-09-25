# Annotation default boundary audit: nested annotations and floating values

This is a source-only Java 8 audit. `Defaults.java` is legal Java source, compiled by javac 23.0.1 as:

```sh
javac --release 8 -g:none -d classes Defaults.java
```

The fixture was not patched after compilation. The four input class files are Java 8 class files; their SHA-256 values are frozen in `class-sha256.txt`. `javap-*.txt` preserves `javap -v -c` output, including each `AnnotationDefault` payload and the runner's code. The CLI under test is `/tmp/jarde-cli-static-root-after`, SHA-256 `8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44` (also recorded in `jarde-cli-sha256.txt`). JADX is 1.5.6.

The source covers a scalar nested annotation, an array of nested annotations, class and enum values, empty and populated arrays, negative zero for both floating widths, positive float infinity, and double NaN. The runner inspects reflective defaults, including raw sign bits for both negative zeros. The original fixture passes (`original-reflection.txt`); JADX's four complete generated classes compile with `--release 8`, and its runner passes (`jadx-javac.log`, `jadx-reflection.txt`). JADX preserves the source-level forms for nested annotations, arrays, class/enum values, negative zero, infinity, and NaN.

Jarde's outputs and machine-readable report are preserved as `jarde-*.java`, `jarde-*.log`, and `jarde-Defaults.json`. Jarde retains the declarations, class literals, enum defaults, and primitive arrays. It omits both the scalar nested default and the entire nested-annotation array default. It also omits all four float/double defaults, including negative zero. This agrees with the current `resolve_default` implementation: `ElementValueFacts::Annotation` returns `None`; an array is resolved only if every child resolves; and `F`/`D` are refused. The reader did consume these values: `javap-Defaults.txt` shows `@...` and `[...]` payloads for the nested values and `F#`/`D#` payloads for the floating values.

Jarde source compilation fails, so there is no successful Jarde reflection run. The harness still attempts the run and preserves the `ClassNotFoundException` in `jarde-reflection.txt`. The full compilation transcript is `jarde-javac.log`; an annotation-only attempt is in `jarde-annotation-only-javac.log`. The latter independently shows Jarde emits `extends java.lang.annotation.Annotation` on `@interface`, which javac rejects, and the nested defaults are absent from the source. The full attempt first reports the unrelated enum-member presentation in `jarde-Tone.java`. No generated output was edited to work around these failures.

## Architecture finding

The reader already has the structure needed for nested annotation values: `ElementValueFacts::Annotation` carries the annotation descriptor and ordered name/value pairs, and `Array(Vec<ElementValueFacts>)` recursively carries every element. No new class-file reader mechanism is indicated by this evidence. The source layer is the gap: `src/class_source.rs::MemberDefault` and `resolve_default` have no annotation variant, so a nested value cannot become a declaration literal and nested arrays fail atomically.

A narrow source-layer extension can resolve a nested annotation's descriptor and ordered pairs recursively, then spell it as an annotation use (`@Type(name = value, ...)`), including recursively rendered arrays. This follows the existing all-or-nothing array behavior and avoids inventing values. The float/double boundary is separate: the reader keeps their constant-pool indexes, while source resolution currently refuses both tags. If later addressed, spelling must preserve at least signed zero and compile-time-constant legality for infinities/NaN; ordinary method calls such as `Float.intBitsToFloat` cannot occur in annotation element values. The evidence here does not establish how arbitrary NaN payload bits should be represented in Java source, so that choice needs its own source-semantics review. This audit does not change production code or OpenSpec tasks.

## Replay

From the repository root, run:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/annotation-default-boundaries/run_audit.py
```

The script regenerates the classes from the checked-in source, records hashes and `javap`, decompiles all four classes with JADX and Jarde, and preserves compiler/runtime logs and exit statuses. It expects the existing CLI at the path and hash stated above, plus `javac`, `java`, `javap`, and `jadx` on `PATH`.

Root copied this entire evidence directory to `/tmp/jarde-annotation-root-UG3q4k` and reran the portable script. All four regenerated class hashes were byte-for-byte identical to `class-sha256.txt`; original and JADX reflection both again printed `reflection defaults: PASS`. The unedited Jarde full-class compile failed first on the enum presentation, while the annotation-only compile exposed the separate illegal `@interface ... extends java.lang.annotation.Annotation` header. Root checked the generated `Defaults` source directly: `nested`, `nestedArray`, both negative zeros, infinity and NaN have no default; the class/enum/int-array controls retain theirs. The frozen CLI SHA was checked before and after the replay. No Jarde runtime equivalence is claimed.

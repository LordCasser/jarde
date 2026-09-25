# Bitwise type and consumption boundaries

This audit uses one class compiled with `javac --release 8 -g:none`. It patches only same-width
method descriptors and one straight-line `Code` body; it adds no permanent fixture class and does
not edit decompiler output. The script records each input hash and command status in `summary.json`.

The source class is 541 bytes, major version 52, SHA-256
`8ef5bf00b1aa19456d5fb405bd64cf64b7734583ab2c8cd7f428534f0d5c581d`. The patched class is 539
bytes, SHA-256 `6c55df799d7a643beec915b968596ab3a1edd4243a1b06d1dfe3d3142cf1accc`. Both source and
patched classes load and run under `java -Xverify:all`. The frozen jarde CLI hash is
`8cd1f767ba8cde9928bacedc29263d44be8e5e61b46d3b6f00456fac1c7c77cd`, unchanged after the run.

The mixed descriptors are verifier-legal because the `Z` arguments are integral computational
values to these `iand`/`ior` instructions. `mixedLeft(ZIJ)I` applies `iand` at BCI 2; `mixedRight(IZB)I`
applies `ior` at BCI 2. The JVM runner exercises both parameter orders. The corresponding direct
Java expressions `boolean & int` and `int | boolean` both fail `javac --release 8`; JADX instead
rewrites the left case with a boolean-to-int conditional and emits an invalid comparison in the
right case. jarde quotes both operation/return pairs at BCIs 2/3. This matches the spec boundary:
do not infer a Java expression from verifier `Int` values or insert a conversion.

`pureIntegerLiterals(II)I` is `iload_0; iconst_1; iand; iload_1; iconst_0; ixor; ior; ireturn`
(BCIs 0–7). Its result is 3 for inputs 5 and 2. The literals do not make this an implicitly
boolean expression. The overwritten-local case saves argument 0 to local 2 at BCI 1, overwrites
local 0 at BCI 4, then reads local 2/local 1 at BCIs 5/6 for `iand` at BCI 7 and `ireturn` at BCI 8.
The correct result for `(6, 3)` is 2. jarde's existing partial text keeps `local2 = arg0; arg0 = 31;`
and quotes BCIs 7/8, so the old value remains identifiable.

`observeOnce(II)I` starts as javac's source-equivalent local form:
`iand; istore_2; iload_2; invokestatic observe:(I)I; pop; iload_2; ireturn`. The patch replaces that
with `iand` at BCI 2, `dup` at 3, `observe` at 4, `pop` at 7, and `ireturn` at 8. The old and patched
classes both return 2 and leave `trace == 2`, proving the shared value and side-effecting call each
execute once. The full source class and JADX class both compile and run with identical six-line
output. Whole-class jarde output is intentionally reported separately: its current frozen baseline
leaves five value-returning methods without return statements, so `javac` fails and no jarde runtime
is claimed. On the patched class jarde quotes the unproved duplicate and consumer chain rather than
writing the call twice; the `observe` member itself remains independently rendered.

No OpenSpec change is needed. The existing mixed-operand and integer-literal boundaries are already
explicit, while the rejected-consumer requirement says to retain source references when a chain is
not proved. The duplicate form exposes an existing boundary: general multi-use expression
materialization through `dup` is unsupported. Supporting arbitrary duplicated values would need a
separate bounded use-count/evaluation-order design; it is not necessary to recover the bitwise
operator family and should remain outside this change.

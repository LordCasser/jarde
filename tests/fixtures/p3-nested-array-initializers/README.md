# Java 8 nested array initializer controls

`v8/NestedArrayInitializer.class` and `v8/JadxOrderingControl.class` are the frozen
byte-for-byte fixtures in `openspec/evidence/java-syntax-2026-09-25/nested-array-initializer/`.
The first proves ordered nested `int[][]` and `String[][]` recovery; the second pins the
effectful reverse-index trace at `12`.

`NestedArrayExtraUse.class` is built with `javac --release 8 -g:none` from the adjacent
source. Its child array is read through `arraylength` before its parent store. The original
class verifies and prints `[[4]]:4`; recovery must not publish a partial nested literal.

# EM-18 byte-array return fixture (prepared only)

This is a minimal full-class input for the active JADX positive assertion in `TestArrayFill3.test`: an ordinary instance method returns `new byte[] { 0, 1, 2 }`. The method body is copied as a Java fixture shape, without the upstream JADX test harness or product code. `Runner` is shared unchanged across original, JADX, and Jarde source legs.

The runner prints the first returned value, whether two calls return the same array, then the value from a fresh call after mutating the first array. This checks the literal contents, allocation freshness, and that one call's returned array is independent of later calls.

This directory only prepares source inputs; no compiler, JADX, Jarde CLI, or runtime was invoked. A later Java 8-targeted `javac` replay can exercise the source shape on JDK 8/23, but it does not establish the upstream `ECJ_J8` or `ECJ_DX_J8` profile result. In particular, this is not evidence for DEX `fill-array-data`.

The likely existing Jarde path is the typed array proof in `crates/jarde-java/src/build.rs:12729` (`ArrayInitializers::prove_with_composition`), called from `crates/jarde-java/src/report.rs:11096`, with `ExprKind::NewArray` emitted by `crates/jarde-java/src/emit.rs:1383-1413`. Those are reuse candidates only; this prepared fixture has not tested them.

Input hashes:

- `ByteArrayReturn.java`: 105 bytes, SHA-256 `11f6b209d34b05454cbd98e8c59f69e51f479235c1731f7ff56e9b795b53b4a2`
- `Runner.java`: 413 bytes, SHA-256 `8109c62371babca1caca0ab1c1ae9230fbb1e398b1aaff0b11d6806156a3a772`

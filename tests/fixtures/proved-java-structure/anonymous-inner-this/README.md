# Anonymous `Inner.this`

This Java 8 fixture isolates a lexical enclosing-instance read inside one anonymous `Runnable`. Its anonymous body stores `Inner.this` as an object identity and updates `Inner.this.f`; `main` observes both as `true` and `38`.

Compile with `javac --release 8 -g:none` and run with `java -Xverify:all`. `SHA256SUMS` freezes every class emitted by that compilation. The companion replay under `openspec/evidence/java-syntax-2026-09-27/anonymous-inner-this/` compares full JADX and Jarde class-source source sets without writing fixture artifacts into the repository.

The shape does generate an anonymous-class `this$0` field. This fixture is not a DT-06a no-capture superclass-argument case: its allocation targets `Runnable`, and its only constructor argument is the lexical `Inner` instance captured by the anonymous class.

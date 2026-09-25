# Optional throws suffix in a method Signature

`javac 23.0.1 --release 8 -g:none` compiles `GenericThrowsProbe.java` to a 312-byte Java 8 class with SHA-256 `f1f1b4cff298113db985e13cc0fe3cc1df740ca4e388f1876712ae21dd2b2b27`. The physical method descriptor is `(Ljava/lang/Number;)Ljava/lang/Number;`. Its `Signature` is `<T:Ljava/lang/Number;>(TT;)TT;` with **no** `^` throws suffix, while the same method's `Exceptions` attribute lists `java/io/IOException`. See `generic-throws-javap.txt` and the frozen `.class`.

The original class and JADX 1.5.6 source both compile under Java 8 and produce `3|1` and `java.io.IOException` through the source-only reflection runner under `java -Xverify:all`. `GenericThrowsProbe.jadx.java.txt` is the raw JADX output; its synthetic `package defpackage;` line was removed only for the default-package recompilation. A Jarde class-source comparison awaits the next CLI build.

This is a positive compatibility boundary for `recover-generic-method-signatures`: an empty Signature throws list cannot be equated with an empty `Exceptions` attribute. The method header must retain the ordinary checked exception from `Exceptions` while recovering its method-local `T`.

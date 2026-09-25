# Mixed short-circuit value stored in a local

Compile `MixedBooleanLocal.java` with `javac --release 8 -g:none`. Its `one(Z)Z` method stores `(a && b()) || c()` once in a boolean local, then reads that local once for a static field write and once for the method return. `Runner.java` covers all eight `a/bValue/cValue` combinations under `java -Xverify:all`, recording the returned value, field value, and `b()`/`c()` call counts. The frozen class is the subject; `Runner.class` is intentionally not retained.

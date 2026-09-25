# Left-false short-circuit field write

`ShortCircuitFalse.assign(boolean)` assigns `left && rhs()` to a static `boolean` field. `rhs()` increments a visible counter and returns `true`, so the `false` input proves that short-circuiting skips the right-hand side while the `true` input proves it runs exactly once. The checked-in `.class` files are the frozen Java 8 input used by the OpenSpec replay.

From the repository root, regenerate the frozen classes with:

```sh
javac --release 8 -g -d tests/fixtures/proved-java-structure/short-circuit-left-false tests/fixtures/proved-java-structure/short-circuit-left-false/ShortCircuitFalse.java
```

The three-way replay and behavior comparison are in `openspec/evidence/java-syntax-2026-09-25/short-circuit-left-false/reproduce.sh`.

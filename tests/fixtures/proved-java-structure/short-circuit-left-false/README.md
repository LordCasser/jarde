# Left-false short-circuit field write

`ShortCircuitFalse.assign(boolean)` assigns `left && rhs()` to a static `boolean` field. `rhs()` increments a visible counter and returns `true`, so the `false` input proves that short-circuiting skips the right-hand side while the `true` input proves it runs exactly once. The checked-in `.class` files are the frozen Java 8 input used by the OpenSpec replay.

From the repository root, regenerate the frozen classes with:

```sh
javac --release 8 -g -d tests/fixtures/proved-java-structure/short-circuit-left-false tests/fixtures/proved-java-structure/short-circuit-left-false/ShortCircuitFalse.java
```

The three-way replay and behavior comparison are in `openspec/evidence/java-syntax-2026-09-25/short-circuit-left-false/reproduce.sh`.

## Behavior baseline (CI guard, measured 2026-10-07)

The frozen classes' `java -Xverify:all` run (JDK 23.0.1) prints:

```text
false-result=false,calls=0
true-result=true,calls=1
```

The CI guard is [`tests/fixture_behavior_guards.rs`](../../../fixture_behavior_guards.rs): the presentation
leg stays in the default suite (the single `result = left && rhs();` write), and the behavior leg is
`#[ignore]`d like `p3_execution_comparison` (`cargo test --test fixture_behavior_guards --locked -- --ignored`)
— it runs the frozen class and recompiles the recovered text, comparing the two runs. The measured golden,
the classification and the javap facts are recorded in
[`openspec/changes/recover-fixture-behavior-guard-coverage/results/`](../../../../openspec/changes/recover-fixture-behavior-guard-coverage/results/README.md).

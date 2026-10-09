# Luna implementation record v1

## Written, not executed

- `crates/jarde-java/src/build.rs`: added only the release-8
  `java.math.BigDecimal -> java.lang.Number` row to the existing `NUMBER_FAMILY`. The existing
  release gate and six boxed rows remain. The old BigDecimal negative helper assertion is now a
  positive; release 7/9, BigDecimal to unlisted targets, reverse assignment, BigInteger, atomic
  types, and the existing other negatives remain refusal cases.
- `tests/p3_bigdecimal_number_widening.rs` adds
  `main_only_bigdecimal_array_is_recovered_from_both_frozen_javac_legs` as a structural test that
  does not need Java. It opens one original `Main.class` at a time in the production
  Engine/class-source API, requires structured Java recovery without a bytecode fallback in `main`,
  checks one construction, one `Number[]`, one three-append/toString print chain, and requires
  source-map coverage at BCIs 1/6/9/12/15/16/17/28/29/34/40/43/46. The separately ignored
  `main_only_bigdecimal_generated_source_matches_frozen_original_runtime` compiles and verifies the
  complete generated source and compares the frozen raw output streams. Java subprocesses clear
  `JAVA_TOOL_OPTIONS`, `_JAVA_OPTIONS`, `JDK_JAVA_OPTIONS`, and `CLASSPATH`.
- The `javac8/Main.class` and `javac23/Main.class` fixtures are direct copies from the frozen
  baseline jars, with SHA-256 `7c302ba965d6f94528b3be6d634477bb62eb8078408c12a460d1075730571997`
  and `e4bf655f328f116957186fd9a244ad0e11cc27f8fa3c6684c3b63c11ab19dc87`, respectively. Their
  `Main.java` files are the unchanged historical source. The stored stdout hash is
  `2cff3d4e1d402f123ac6042cd50af20e8bbe97d94384b341056fbbe4e80d7981`; stderr is empty.
- Added `tests/fixtures/bigdecimal-number-widening/NumberArgument.java` as the source-only
  invocation control. Root compiled and froze both original toolchain outputs, then
  `bigdecimal_argument_is_recovered_at_the_actual_number_descriptor_call` was added as a
  Java-independent structural production test. It asserts the actual identity descriptor is
  `(Ljava/lang/Number;)Ljava/lang/Number;`, the main call is mapped at BCI 12, and the BigDecimal
  construction, identity invocation and body are fully structured Java. Its separately ignored
  `bigdecimal_number_argument_generated_source_matches_frozen_runtime` compiles and verifies the
  generated whole class against the frozen streams.
- Frozen NumberArgument class SHA-256 values are `6c424bfdf36362d95bbc8af07b2c2513758434a5458e417a498f9069955bf79f`
  (Corretto 8) and `4f066c9b1a592c47c272926db767cf7f8ad9daf11e884c13c430237ce6169f82`
  (OpenJDK 23). The root command and original-run record are in
  `results/number-argument-original-v1/manifest.json`; raw stdout is `2.50\n`, stderr is empty.

Root compiled that control independently with each frozen JDK, using an empty classpath and
sourcepath and an isolated output directory:

```sh
"/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac" -source 8 -target 8 -g:none -Xlint:-options -classpath "$EMPTY" -sourcepath "$EMPTY" -d "$OUT8" tests/fixtures/bigdecimal-number-widening/NumberArgument.java
"/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javac" -source 8 -target 8 -g:none -Xlint:-options -classpath "$EMPTY" -sourcepath "$EMPTY" -d "$OUT23" tests/fixtures/bigdecimal-number-widening/NumberArgument.java
```

Those original compilations completed with exit 0. The expected actual consumer is the
`invokestatic NumberArgument.identity:(Ljava/lang/Number;)Ljava/lang/Number;` at BCI 12 in
`main`; the BigDecimal construction begins at BCI 3 and invokes its constructor at BCI 9. Root
should record both resulting `NumberArgument.class` hashes before adding them as frozen test inputs.

Luna did not run Cargo, Java, CLI, rustfmt, or OpenSpec task commands. Both structural tests and
the ignored host-JDK generated-source comparisons remain unexecuted; root owns all verification and
task-checkbox updates. No architecture blocker was found in this scoped code review.

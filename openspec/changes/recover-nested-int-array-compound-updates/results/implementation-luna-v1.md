# Luna implementation preparation v1

## Prepared, not applied or executed

- `implementation-luna-v1.patch` removes only the redundant array-store-copy type gate in
  `prove_array_update`. It retains the original-array `int[]` proof and the existing exact four
  `dup2` output identities, read/store operands, single-use checks, instruction order, dependency
  closure, and budget/stop propagation. The added comment records why the copied aaload value is
  not independently typed and how the existing source walk lowers `[[I` to `[I`.
- The patch updates the old P02 snapshot test. The historical baseline/fixed evidence is still
  asserted equal; constructor, `sum`, and `main` regions are compared against the fixed snapshot.
  Only `lambda$sum$0` is upgraded to structured Java with no quote, explicit
  `p0[0][0] += p1.intValue();`, and source-map coverage for BCIs 0/1/2/3/4/5/6/7/10/11/12.
- `tests/p3_nested_int_array_compound_updates.rs` is a new production integration test against the
  two frozen `NestedIntUpdates.class` fixtures that the root control script will create. Its normal
  test checks complete member recovery, no bytecode fallback, preserved compound assignments in
  2D/3D/scalar/traced/replace-row methods, one evaluation each for trace calls, and ordinary
  assignment for different read/write rows. Its explicitly ignored execution test compiles the
  complete generated class plus the fixed `Runner.java` with empty class/source paths, verifies
  only fresh classes, and compares stdout/stderr to the frozen runner oracle. Java subprocesses
  clear the four injected JVM/classpath environment variables and use the existing JARDE_JAVAC23 /
  JARDE_JAVA23 selection pattern.
- The fixture source, expected scenarios, toolchain identity, commands, and output paths are fixed
  by `results/prepare-controls-v1.py`; Luna did not run it. Its generated fixture paths are
  `tests/fixtures/nested-int-array-compound-updates/{v8,v23}/NestedIntUpdates.class` and
  `tests/fixtures/nested-int-array-compound-updates/Runner.java`.
- No tasks were checked. No Cargo, Java, CLI, Git, or rustfmt command was run. The patch and new
  test are unexecuted and await root review/application after the independent CI correction and
  freeze. No new framework or AST/type mechanism was introduced.

## Separate stale-test correction for BigDecimal CI

`ci-stale-boundary-fix-luna-v1.patch` targets only the two existing tests named by root. It updates
the constructed Number initializer to require Structured/Java, the actual Number[] with one
Integer and one BigDecimal, one `mark("1")` and one `mark("2")`, while retaining both construction
records and all prior source-map BCIs. It updates the BNX negative to the exact fresh two-leg
render: only AtomicInteger at BCI 46 carries a refusal; BigDecimal at BCI 21 no longer does, while
the whole main correctly remains a fallback due to the atomic call. Root reported that this patch
passes `git apply --check`; it has not been applied by Luna.

# FT2 whole-class source-origin test candidate

This private candidate adds one permanent real-class test to
`crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs`. It does not modify the
workspace, `region.rs`, the CLI, or any fixture/census files.

## Pins

- Product baseline required by the implementation task: `crates/jarde-java/src/region.rs`
  SHA-256 `b34f8549ed8cd598a1b0f45b3ba5157df387882eb8955eaad1dc2ca351575f4c`.
- Test-file base SHA-256:
  `fc809a7921d1366c12b5472a3f211c3d4526c4c7bbf7f4eaf52e8e68c32aeb99`.
- Frozen input is the already-tracked FT2 class at
  `openspec/evidence/java-syntax-2026-10-11/cf12-upstream-remaining-root-v1/full-replay-root-v1/TestSwitchWithFallThroughCase2.test/original/classes/jadx/tests/integration/switches/TestSwitchWithFallThroughCase2$TestCls.class`.
  Its evidence SHA-256 is `1b1e78da99c371f3292bdf672e599fb07737b2b9f4eca5233bad5fd724f66a3b`; the test additionally pins its embedded BLAKE3 identity and byte length.

## Test intent

The new test reuses the existing real-class recovery setup and physical-bytecode coverage helper, parameterized to accept the frozen FT2 class. It exercises the complete `test(IZZ)Ljava/lang/String;` method twice (default/source-map evidence and all evidence) and requires identical text and source maps. It checks that the outer `if` region owns dispatch block 8 and join block 175 exactly once, the complete switch source span starts at physical switch BCI 11, the tail at BCI 195 appears once after the switch, and the physical outer condition at BCI 5 maps to its complete `if (a > 0)` span. Every physical instruction must retain an origin from this exact method.

The frozen whole-class API output was read from
`/private/tmp/jarde-cf12-ten-fixture-candidate-replay-root-v1/raw/0173.TestSwitchWithFallThroughCase2-jdk23_tools-jarde-all-TestSwitchWithFallThroughCase2$TestCls-class-source.stdout.raw`.
It establishes canonical blocks/edges and source-map semantics used by the assertions: dispatch terminal BCI 11, switch targets 36/121/150/default 153, shared join 175, and outer boundary 197. The source map gives the nested conditional exits at goto BCIs 93 and 118 one direct `break;` span each. Goto 147 is derived on the case-2 `if` span and goto 150 is derived on the switch span in that output; the test checks they are physical `goto` instructions and covered by the exact-method physical-origin check, without asserting a nonexistent direct break span or guessed derived-span cardinality.

The region report is top-level, so the test does not invent a nested switch-region record or assert that join 175 is absent from the outer-if region. Instead, it verifies that the join/tail physical origins are all after the complete switch statement span, and that the join has a single region owner. This is the available public report/source-map evidence for “join outside cases, tail once.”

## Validation status

Not compiled or run here, by task instruction. No Cargo, JDK, rustfmt, Git, or product CLI commands were run. Root owns applying this patch to the frozen base, compiling it, and deciding acceptance against the focused test and whole-class evidence.

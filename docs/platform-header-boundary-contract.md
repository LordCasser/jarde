# Current platform-header boundary and AW test correction

This note records a test-contract correction, not a newly supported conversion. Production code,
platform tables, frozen fixtures and archived historical evidence are unchanged.

## Current authority

Under Java 8, PlainJar, SnapshotAll, Generic layout, disabled multi-release handling and the single
clean parent-first `app` domain, a selected application header can prove an edge it names directly.
`AW$Work implements java/lang/Runnable` therefore proves the required Runnable target without
requiring a physical Runnable header.

The same rule does not supply a missing intermediate ancestor. Both frozen AW compiler legs have
`AW$MyErr extends java/lang/Exception`, with no physical Exception or Throwable definition. H2 has
the same exception hierarchy. At the missing Exception node, depth is one, the initiating loader
is still the caller's app loader, and no platform-table edge has been traversed. The existing
`consult_platform` gate is false. Exception has a nonempty platform parent row, so the terminal
negative closure cannot apply. The relation stays `Missing`, not a proved conversion and not
`Incompatible`. Supplying an actual physical chain would be a different input and would not repair
or validate the original absent-header assertion.

Evidence-request categories observe the same proof; they do not grant authority. Constructor
arguments, nested versus top-level spelling, and class names likewise cannot confer authority.
The test matrix exercises these counterexamples through the public class-source Engine entry
in the same environment. Its generated inputs are same-context derivatives, not independent
holdout samples. The frozen fixture class bytes and H2's two refusal checks, including the missing
Helper chain, remain unchanged.

## Historical identity mapping

- `9dbfd22a` (2026-10-02) pinned H2's refusal.
- `2fdb30b1` (2026-10-06) pinned AW's refusal under
  `the_named_platform_target_is_proved_and_the_absent_one_still_refuses`.
- `52c6bc31ede43ce093531b065aaffcad033856db` (2026-11-10) changed AW to
  `the_named_platform_target_and_selected_user_header_chain_are_proved`, a positive expected
  invocation and Structured outcome. That same revision retained the restrictive production gate
  and H2 refusal. It is an inconsistent expectation migration, not evidence of admitted behavior.
- The corrected identity is
  `the_named_platform_target_is_proved_and_the_missing_intermediate_refuses`. It deliberately
  replaces the unsupported positive assertion. A passing corrected test must never be reported as
  execution or success of the old positive test.

The archived narrow proposal required actual selected headers up to the named target and preserved
hierarchy fixtures; its October 15 clarification excludes missing intermediate ancestry. Broader
qualified finite-platform-table architecture does not supersede this narrower boundary. Existing
fixture README descriptions agree with the refusal. Historical records are not rewritten here.

## What the evidence does and does not show

The corrected frozen test pins the entire refusal text, real invocation BCI, retained app-loader
physical-header dependency, complete execution, explanation-only content and non-Structured
quality. Separate controls derive the invocation BCI from actual input instructions and retain the
full Missing diagnostic. Both frozen compiler legs continue to require the named Runnable call.
Existing H2 runtime checks execute the original valid input with Helper on the runtime classpath;
that runtime classpath is not the analyzed snapshot.

Generated controls compile Java 8 inputs and execute those original inputs under `-Xverify:all`.
They do not recompile the refused empty void body or call that semantic equivalence. They show the
original call is valid at runtime while the narrower analysis input cannot prove its conversion.
Full target, proof-boundary, two-seed, lint and specification receipts are validation evidence
separate from this source-level contract; this note alone claims no test execution.

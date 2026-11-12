# Current shared-catch ownership contract

This note supersedes only the old SHARED expectation in
`recover-loop-body-try-catch/design.md` (2026-10-13, paragraph starting
“完整工作区发现”). That historical record and its evidence remain unchanged.

Both frozen `ScopeRefusalsEscape` compiler inputs currently recover a method-visible
`java.lang.Object local1`, initialized from the Runnable argument, and a distinct
clause-local multi-catch `local2`, copied into `local1` before returning it. The
handler's store/load at BCIs 11/12 belong to slot 2; BCI 13 assigns slot 1 and
BCIs 14/15 read/return it. The exception table has the two typed rows over [2,8),
both targeting 11. This is not the historical ill-typed Runnable declaration.
The public Engine source, original class execution and untouched whole-source
recompilation are separate checks. No production admission changes are required.

The escaped derivative changes only the handler store/load to slot 1. Its catch
header still cannot own the method-visible local. The refusal must retain its
full sentence, quoted blocks 0/11/14, region records and source-map closure;
no partial executable body is allowed. Executing that original derivative does
not establish recovered semantics: its refused whole-source output cannot compile.

`sharedHandler` has no cleanup. `savedAcrossFinally` only assigns its saved local
back to another local; that non-observable assignment does not prove exception
cleanup semantics. SAVED/COMPUTED remain refused, and complete `ScopeRefusals`
output still fails javac. Structural nestedHandler checks are not whole-class
runtime equivalence.

A separate complete `LooplessFinally` gate compares both frozen compiler inputs
with unmodified whole Engine text using an independent throwing PrintStream.
It checks each path cleans up once, the return value is saved before mutation,
cleanup overrides pending return or body failure, and plain finally adds no
suppressed exception. Existing cause/suppressed identities are preserved.
This is distinct from try-with-resources: its body failure remains primary and
close failures are suppressed. The whole WTyping gate proves single-resource
behavior; the multi-resource geometry gate proves close/suppression order in an
assembled-method harness only, not in complete untouched class output. No new
cleanup shape or resource-finally admission is introduced here.

These current gates use the installed javac with `--release 8` and installed JVM
with `-Xverify:all`, on both frozen compiler-input legs. They do not claim a fresh
real-javac8 compiler or Java8 runtime execution when those tools are absent.

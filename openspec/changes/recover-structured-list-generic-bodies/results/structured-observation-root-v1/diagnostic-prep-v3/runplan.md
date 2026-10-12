# CF11 structured-void/generic diagnostic prep v3

Prepared only; this runner is for root to consider after full4 and the product's own CI are complete. It is not an instruction to start a build now. No test, build, acceptance checker, or Git command was run by this prep.

The diagnostic patch is unchanged from v1 and stays bound to the same fixed `cf11/UpstreamNestedLoops.test(Ljava/util/List;Ljava/util/List;)V` path. It only adds a fixed-method `structured_facts` event in the existing `generic_return_candidate` diagnostic. It leaves candidate selection, refusals, declarations, and output untouched. The event dumps the recursive Program/origins and final root statement, full decoded MethodCodeFacts, each physical BCI's operation/SSA/effects matches, extra SSA/effect BCIs, descriptor parameter-slot reads/writes, and full SSA values/uses/phis. The same diagnostic attempt still emits its prior candidate/refusal events.

V3 fixes the runner's closure-key check: `closure.source_paths()` returns absolute `Path` values, while `closure.pin_sources()` returns a dictionary keyed by relative path strings. The runner derives the expected relative keys from the actual `Path` set, compares that dynamic set before and during instrumentation, permits exactly the two instrumented source files to differ during the test, and requires every source pin after restoration to equal the before snapshot. It records the actual dynamic count and does not hardcode a closure count. It has no CLI or historical build-receipt binding. It also disables Python bytecode writes before loading helper modules, and routes the final fixed `cargo clean` through the guard's normal one-second resource sampler with only that cleanup's space limits disabled.

Still fixed and fail-closed: the exact base SHA for `src/class_source.rs` (`bd797db1b47a0258c5031260f867fcd766ae9d5036eb4e8373f7be6fb827a71f`) and `crates/jarde-java/src/report.rs` (`9dde045eb410c797f840a85df5b9dccf711bea140ccce30651fd34243bcd2497`); candidate source hashes; the existing TestNestedLoops fixture path/SHA; one existing public API test; default/all/no-rule-details repetition; complete event groups and byte-identical observations across requests; default Rust stack; 5 GiB free-space / 1 GiB target guards; only the fixed `cargo clean` finally exception; raw stdout/stderr capture; exact byte restoration and target absence. If root's full4 checkout changes either base source hash or this fixture, the runner refuses before applying the patch rather than updating pins implicitly.

The runner output is `/private/tmp/jarde-cf11-structured-void-generic-diagnostic-run-root-v3/execution.json`.

Root invocation after choosing the safe serialized window:

```sh
python3 /private/tmp/jarde-cf11-structured-void-generic-diag-prep-luna-v3/run_diagnostic.py
```

Afterward, inspect the captured `structured_facts` event in every request group. The runner checks only presence and request-to-request equality; it does not certify that the BCI correspondence is complete or that any generic candidate is sound. Specifically verify raw Code vs operations vs all SSA blocks/effects at each physical BCI, missing/extra/duplicate rows, final void-return identity, and the two parameter slots' actual reads/writes through uses, calls, casts, and Phi inputs. No hand-built `VoidBody` is inserted, so the downstream parameterized declaration gate is not claimed to have run.

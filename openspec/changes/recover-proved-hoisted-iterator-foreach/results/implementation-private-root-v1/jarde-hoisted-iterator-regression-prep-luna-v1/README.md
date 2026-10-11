# Hoisted iterator regression preparation (private, unrun)

This directory is a review candidate for task 2.2 only. It does not change the shared repository and does not claim any new toolchain or implementation result.

`p3_hoisted_iterator_foreach.rs.candidate` is a complete public-API test candidate using the accepted frozen `cf11/UpstreamNestedLoops.class` already in the repository evidence tree. It checks the intended inner-only foreach projection while retaining the outer iterator assignment/while; compares default, all evidence, and evidence without RuleDetails; verifies exact target method identity, every physical instruction BCI and UTF-8-safe map spans; and derives a late AnalysisSteps budget from a complete run so the target method itself must stop with a location and no body/map/news. Pre-cancel is separate and zeros every counted usage field after zeroing elapsed time. No threshold is guessed.

`HoistedIteratorBoundaryControls.java.candidate` and `gate-test-matrix.md` are source-only probes and an adversarial matrix. They are deliberately not permanent assertions yet: without compiled bytes and observation of the actual first gate, a refusal could arise earlier for an unrelated reason. The matrix separates concrete accepted SSA facts from proposed counterexamples. It does not invent an SSA test API or require every unrelated phi to be trivial.

Root should adopt the public candidate only after the implementation's actual output agrees with its source-shape expectations. Then compile each source-only probe if useful, inspect emitted debug-local identities/IR first, and promote only controls that reach the intended condition. Unit tests may target a naturally extracted pure production helper if implementation review finds that seam worthwhile; no test-only helper or SSA mutator is proposed here.

No Java, JDK, JADX, CLI, Cargo, Git, or test command was run for this preparation. Candidate Rust was not compiled. Expected behavior for all new probes remains a design expectation, not an observed result. Pins record the frozen class and the read-only source/artifact context used for review.

# TestNestedLoops2 same-IR ForHeader diagnostic (private prep v2)

This candidate adds one environment-gated logger and one private library test to `region.rs`. The test opens the pinned original `javac23` class and analyzes the exact `cf11.UpstreamNestedLoops2.test(Ljava/util/List;)Z` method through Reader, then prints that invocation’s canonical CFG edges, SSA table and instruction reads/writes, normal-flow nodes, natural loop headers/blocks/latches, and recovery result. During recovery the existing `prove_for_header` calls emit their actual arguments and results. The candidate trace also labels the candidate’s explicit `None` gates by their source line in the pinned base and records stage boundaries around fallible lookup steps. If candidate analysis reaches the header test instructions, it prints each operation and reports the unsupported-operation gate with the exact BCI; this makes it possible to see whether `Invoke@6` is reached before the refusal.

All trace calls are test-only/environment-gated. They do not change a production predicate, its order, budget charging, or stop handling. No whitelist or API is changed. This is a diagnostic candidate and has not been compiled or run.

Apply only after confirming `base_sha256` in `pins.json`; run the command recorded there. Restore `region.rs` byte-for-byte to the pinned base after the guarded run.

The paired `javap.txt` and class file paths and hashes in `pins.json` bind this test to the actual `javac23-original` artifact, not a reconstructed fixture.

V2 corrects the init-stage diagnostic lookup to pass `*preheader` to `NormalFlowView::id_of`; the candidate algorithm and test body are otherwise byte-identical to v1.

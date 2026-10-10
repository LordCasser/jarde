# Conditional switch physical boundary test drafts

This private bundle contains two source drafts and complete javac class inputs. Nothing was written into the repository. No Git, Cargo, Rust compiler, rustfmt, JDK, Jarde CLI, or tests were run.

`tests/conditional_switch_proof_tests.rs` is a unit-test module fragment for the private `region.rs` test module after the certificate helper composition. It opens the actual javac 8 A/B `ConditionalSwitchBoundaries` class through `analyze_method_ir`, checks the full six-block/eight-edge canonical graph and empty clone paths, then calls the same `prove_switch_fallthroughs` API with the untouched complete canonical edge iterator. A expects refusal because case 36 exits to two case entries (57 and 67). B expects `36 -> 67`; it also checks the caller's adjacent-label invariant rejects that map because entry 57 intervenes. No edge rows or fake CFG are synthesized.

`tests/p3_conditional_switch_fallthrough.rs` is a reader-backed public recovery integration-test draft for B. It pins complete class bytes, physical method identity and ordinal, all canonical blocks/edges, and decoded physical branch instructions 37 and 47. It exercises default/all reports through public `recover`, requires equal report text and source map, checks no structured switch or `SwitchBreak` is published and no loop is claimed, and checks the public pre-cancelled recovery contract publishes no partial artifact.

The four `.class` files under `tests/fixtures/p3-conditional-switch-boundaries/` are full original reader inputs copied byte-for-byte from `/private/tmp/jarde-conditional-physical-root-v1/cases/`. The tests currently consume javac 8 A/B; javac 23 A/B copies are included for future root selection and are not asserted by these source drafts.

## Root-owned checks still needed

- Compile and run both drafts after composing the matching implementation. These sources are uncompiled drafts.
- Confirm with the real B public report that the structural predicates identify the actual refusal as intended. Add an exact fallback code/reason assertion only after observing that report; this draft deliberately does not invent that classification.
- Capture actual proof budget/cancellation traces before asserting any exact cost, BCI, or stop dimension. The pre-cancelled assertion only covers atomic public cancellation.
- Full-class source compilation/runtime equivalence and javac 23 behavior remain outside this fixture-boundary slice.

## Source observations

The accepted reader dump is `/private/tmp/jarde-conditional-physical-ir-root-v1/0.stdout.raw`. Both javac 8 variants report complete canonical graphs, no unreachable blocks, six blocks at 0/36/40/57/67/74, eight Normal edges, and empty paths. A has `36 -> 40`, `36 -> 57`, `40 -> 67`, `57 -> 74`, `67 -> 74`; B instead has `36 -> 40`, `36 -> 67`, `40 -> 67`, `57 -> 74`, `67 -> 74`. Both decode the physical branches at 37 and 47. The byte SHA-256 values are listed in `SHA256SUMS`.

# Static review

- Scope: one Java source file plus two internal tests and small test-only helpers. No production helper or SSA model changes.
- The tests consume actual analyzed SSA. They do not fabricate or mutate `SsaTable`, `SsaValue`, `SsaUse`, or phi inputs.
- Iterator sources are found by decoded `Operation::Invoke` facts and their first following local `Operation::Store`; the source `ValueId` comes from that store's actual SSA writes.
- Allowed physical BCIs are collected at runtime from source-value physical uses and the immediately preceding local loads of the named iterator receiver calls. The test requires exactly two distinct BCIs for the selected source shape; root must confirm this against both actual class outputs before adopting it.
- The expected refusal is at the current `iterator_phi_use_ownership` live/unreplaced-phi check. Structural assertions require a mixed same-local phi in `mixedIterator` and an exact source operand in a different-local phi in `extraAliasConsumer`; they do not claim the helper reaches arm resolution beyond that earlier rejection.
- Compile/runtime facts are pending. The source's intended shape is not evidence that javac produced it.

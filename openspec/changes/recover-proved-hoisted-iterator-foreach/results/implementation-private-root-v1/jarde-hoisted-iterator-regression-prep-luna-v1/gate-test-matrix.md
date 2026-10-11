# Hoisted iterator regression candidate: gate matrix

This is a review plan, not an assertion of current behavior. No candidate source below has been
compiled, no gate has been observed on it, and no new production helper/API is presumed.

## Public API positive based on frozen input

`p3_hoisted_iterator_foreach.rs.candidate` targets the already accepted
`cf11/UpstreamNestedLoops.test(Ljava/util/List;Ljava/util/List;)V` class bytes. The outer iterator
remains an assignment plus `while`; the intended projection is the inner iterator only. Existing
same-IR evidence records iterator declaration, test load, and `next` load with the same
`LocalVariable { slot: 5, index: 0 }`, elevated owner `[1]`, emitted declaration, and the physical
header phi `V12 = [V37, Itself]`. The later block's `V22 = [V37, V37, V37]` is the concrete
occurrence-multiplicity case. `Local(6)`'s `V23` phi is unrelated and must not constrain this
proof. The candidate checks three public evidence modes, same text/maps/method identity, all 45
physical instruction BCIs mapped to the exact target method, UTF-8-safe ranges, and measured
late-stop plus pre-cancel atomicity. It does not assert a budget constant.

The exact class is already present under `openspec/evidence/...`; the proposed test avoids adding a
new fixture entity. Whether the new production gate changes this class's source is still for root to
verify against its compiled candidate before adopting the test.

## Source-only boundary probes

`HoistedIteratorBoundaryControls.java.candidate` is only source. It is not pinned as Java 8 class
bytes and does not prove compiler local identities or which gate rejects a case.

| Method | Intended evidence question | Safe expectation to assert only after compilation and gate observation |
| --- | --- | --- |
| `iteratorReadAfterLoop` | Does an additional post-loop read of the same iterator local prevent projection? | Keep the original loop if that read is an extra consumer; verify the exact SSA/local identity first. |
| `mixedInitialIterator` | Does a loop whose iterator local has two reaching initializers avoid claiming either iterable? | Keep the original loop if the merged value is mixed; determine whether it reaches the candidate gate or is rejected earlier. |
| `splitBranchCursors` | Are distinct branch-local declarations treated as distinct locals even when source names match? | Do not assert source spelling or conversion count until javac's debug local table and the gate are inspected. |
| `initializerHasSeparateCatchOwner` | Does an initializer in a different protected region retain its exception boundary? | Preserve the loop when moving iterator creation would cross the initializer's catch owner; verify region ownership from actual IR. |
| `iteratorReassignedInLoop` | Is a live/reassigned iterator rejected rather than treated as a stable foreach cursor? | Preserve the loop if reassignment creates a mixed/live value; identify the actual first rejection condition. |

These cases are probes, not permanent public assertions yet. Root should only promote a compiled
case whose actual bytecode reaches the intended identity/ownership boundary. Avoid turning an
earlier unrelated parse/region failure into proof of a specific SSA rule.

## Exact SSA adversarial matrix for implementation-aligned tests

The matrix below states proof obligations, without requiring Java tests to construct inaccessible
`jarde-jvm::ssa::SsaTable` values. Existing test helpers in `crates/jarde-java/src/build.rs` can
analyze real method IR; the actual implementation should decide whether any pure helper is a
natural boundary worth unit-testing.

| Iterator-value use graph | Expected gate result | Why |
| --- | --- | --- |
| Actual target: header phi has one iterator-value operand plus its self backedge; later matching phi has three occurrences and three corresponding `None` uses at that block | Accept | Exact same-IR case; count occurrences, do not deduplicate operands. |
| Matching phi has fewer or more matching operand occurrences than corresponding use records | Reject | Every `None` use must be paired to exactly one matching operand occurrence. |
| A matching phi arm resolves to another value, or has a live/nonempty use set | Reject | It no longer proves a single stable iterator value. |
| A matching phi is unreplaced, cyclic beyond the accepted self edge, or resolves to another value | Reject | Replacement must prove the selected iterator value. |
| An extra `Some(bci)` reader of the iterator exists outside the allowed test/next loads | Reject | The claimed local has another observable consumer. |
| Init/test/next use different local identities, declaration is absent/unemitted, or owner/visibility is incomplete | Reject | The identity and declaration-placement proof is incomplete. |
| Unrelated local has a live/nontrivial phi but no operand/use references the iterator value | Ignore for this proof | An unrelated local must not veto a proof about the iterator; do not require every phi in the block to be trivial. |
| Same iterator value appears in mixed/nonmatching phi with an additional live path | Reject | A partially matching graph is not sufficient evidence. |

For each rejection, record the observed first gate and original structured fallback after the
implementation exists. Do not claim a test covers a downstream condition when it exits earlier.

## Existing test shape and constraints

- Reuse the `class_source_with_evidence`, physical `inspect_method_bytecode`, exact `PhysicalMethodId`
  origin, and budget/cancellation patterns in `tests/recover_proved_conditional_constructor_values.rs`.
- Do not add mock SSA constructors or mutate `SsaTable`; its data shape belongs to `jarde-jvm` and
  current `jarde-java` tests cannot create the private SSA use/phi graph through a public API.
- If the implementation naturally factors a deterministic pure proof helper, root may add focused
  unit tests at that existing production boundary. No new test-only service or mutator is proposed.
- No runtime/JADX/Jarde comparison is supplied by this candidate; the public positive is source
  structure, evidence, origin, and atomic-budget coverage only.

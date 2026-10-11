# Private constructor-Phi candidate v2 review

Status: private preparation only. This copy is based on the current shared source hashes recorded in `pins.json`. Nothing here is applied to the shared checkout. The v1 draft remains unchanged. No Git, Cargo, Rust compiler/formatter, Java, JADX, or CLI command was run; this is a static review, not a passing test result.

## Scope and behavior

The implementation is still limited to the existing `for_header_candidate`/constructor `Sites` path and its report receipt. The final Program walk counts only a direct-origin, method-local `ExprKind::New` at the candidate constructor BCI; exactly one node is required for a conditional site to count as presented. The Site keeps its current `finished_value` meaning. The same immutable final-Program receipt drives Site refusal, counts, and record materialization. `report.rs` is the only production consumer of `Sites`; no Builder or public API change is included.

`outside_readers` now extracts the constructor-output eligibility fact while doing its existing scan. It visits and charges each existing producer write and its uses once, as before. It records a Phi operand candidate only if the constructor has exactly one stack write, that value is unreplaced and defined by the constructor instruction, and the already charged uses slice contains exactly one use with `bci() == None`. The extra Phi and physical-consumer proof runs only for that candidate. Ordinary constructors, and constructors whose output has a physical BCI use, do not scan the Phi table or join block and do not pay the new proof charges. The original physical reader list and its sort remain unchanged.

The one-hop proof still requires a two-input stack Phi in that exact operand block, two distinct explicit SSA values, the constructor output in exactly one input, an unreplaced Phi definition, exactly one use of that Phi output, and one physical instruction in the Phi's join block reading that value exactly once. A different Phi block, a shared constructor output, a duplicated Phi input, a replaced Phi, a multi-use Phi result, or a consumer outside the join rejects the marker. This marker is only a candidate ownership fact; the existing Builder still checks the complete conditional CFG, exact arms and join, dependencies, and expression eligibility before emitting a `Conditional`.

Constructor argument evaluation, the constructor invocation, exception timing, and arm-local effects stay in the original Builder proof and rendering path. This candidate does not move an invocation or field access, quote an unproved side effect into a new expression, or claim general purity. If the Builder rejects the region, final-AST accounting records zero matching New nodes and the existing Site is reported as not emitted.

## v2 changes relative to v1

`outside_readers` no longer calls the Phi helper unconditionally. The helper accepts the already found constructor `ValueId` and operand block, so it does not repeat constructor writes or constructor-value uses. Existing writes/uses are charged once in the original scan; only Phi enumeration, eligible Phi inputs, Phi result uses, join instructions, and reads needed for the conditional candidate are charged in the conditional path.

`Sites::refusals` now merges the existing refusal list and the conditional-not-emitted Site iterator. Both are sorted by head BCI: refused candidates and accepted Sites are sorted in `sites_with_pending_array_composition`. A stable merge preserves the previous `Vec::extend` plus stable `sort_by_key` ordering, including existing refusals first for equal BCIs, without allocating a second vector or sorting the combined result.

Private tests were added for final New counts of zero, one, duplicate (saturated at two), foreign-method origin, and method-reference shape. A fixture-based one-hop test checks the known constructor/Phi/consumer path at BCIs 96/99 and rejects a deliberately mismatched operand block. There is no verified existing fixture in this change's local set that makes this constructor output feed a shared conditional Phi; this draft does not claim that boundary is exercised by the new unit test. The shared-use and multi-use checks remain explicit in the implementation and need an appropriate fixture or root-directed fixture test before claiming runtime coverage.

## Static checks and remaining validation

The private normalized patch is generated against the exact current `crates/jarde-java/src/{init.rs,field.rs,report.rs}` files. `pins.json` records the source bytes and SHA-256 values read for this draft, the four OpenSpec artifacts, and the bytecode fixture used by the one-hop test. The candidate has not been parsed by rustfmt or compiled, and none of the added tests has run. Root should apply only after reviewing this patch and then run the task's specified tests against the real control baseline. Any concrete Builder rejection should be diagnosed before changing the existing proof.

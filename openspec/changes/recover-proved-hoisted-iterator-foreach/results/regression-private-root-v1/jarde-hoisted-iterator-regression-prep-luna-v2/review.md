# Review notes

- Patch base: shared `crates/jarde-java/src/build.rs`, byte length and SHA-256 recorded in `pins.json`. The production helper implementations remain byte-for-byte those of the base; the unified diff is confined to the `#[cfg(test)] mod tests` helper region.
- The test analyzer is factored from the existing `analyze_own_grid` body. Its only behavioral change is taking the class bytes, method name, descriptor, and Java release as arguments; `analyze_own_grid` delegates with its previous method identity.
- No `SsaTable`, `SsaValue`, `SsaUse`, or `PhiInput` is fabricated or mutated. Assertions read real SSA produced from the frozen class bytes. The split-slot identity comes from the existing v8 debug fixture.
- Statement-list and `Declarations` mutations are local test inputs to the production declaration gate. The new wrong-type probe specifically covers the current shared check that the `at_region` anchor's type is `Iterator`.
- The low analysis-step budgets are intentionally set to one and assert the observed second visit's BCI, not a production cost constant. Root must confirm these Stop locations in test execution.
- The historical same-IR diagnostic and fixture pins establish inputs and expected shape only. They do not establish that this candidate compiles or passes.

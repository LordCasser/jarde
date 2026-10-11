# Private v4 provenance-gate candidate

This candidate is based on the currently formatted shared `crates/jarde-java/src/field.rs` captured at preparation time. The base SHA-256 and byte count are pinned in `pins.json`; this remains an unapplied private copy.

The only production change adds `primary().provenance() == Provenance::Direct` to the existing conditional-New receipt gate, alongside the existing no-method check. Candidate membership is still determined by the existing primary-coordinate scan; no derived-origin enumeration, new helper, new receipt type, Builder change, or CFG/SSA change is introduced.

The new unit covers two negative cases: (1) a `New` whose primary has the candidate BCI but is `Derived`; and (2) a `New` whose primary is a different direct BCI while the candidate BCI appears only in its derived anchors. Both must contribute zero to the final-Program committed-New count. Existing zero/one/duplicate/foreign-method/method-reference receipt cases remain intact in the copied source.

Static API check: `source_map.rs` defines public `Provenance::{Direct, Derived}`, `Origin::provenance()`, and `OriginSet::derived_from(primary, derived)`. The visitor already obtains its candidate coordinate from the primary anchor, so the second test proves derived-only coordinates cannot satisfy membership; the added gate closes the first case.

No compiler, formatter, Git, Java, JADX, or CLI was run. The guarded shared build was not touched. This is not a claim that the candidate compiles or that tests pass. Apply only after the current root-owned run and review are complete.

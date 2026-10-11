# Private compound canonical-edge negative candidate

Base `crates/jarde-java/src/region.rs` SHA-256:
`bab494a21724ed61213240fcc75362a68163e47370fbd60ccb07d3ab15f2c235`.
Candidate SHA-256 and exact patch SHA are recorded in `base.json`.

The candidate extracts the current compound test/exit canonical-edge scan into one module-private
validator. The production proof calls that same validator with `CanonicalCfg::edges()` through a
lazy tuple iterator, the same edge count, normal-flow view, frame, tests, exits, and visited set.
It preserves the existing single full-row scan, per-edge poll, analysis charge locations/counts,
root predecessor logic, and acceptance conditions. It adds no public constructor, CFG API, override
field, edge type, or second graph walk.

The existing real Switch2 compound test then feeds the validator the actual reader-produced canonical
row list and three narrow variants using only identities already in that class: (1) outside block 171
enters payload 131; (2) the actual 98→111 test predecessor row is repeated; (3) an Exception row from
171 enters shared exit 136. The unmodified reader rows must pass, and each variant must refuse. These
exercise the exact helper called by the complete production proof, but not a re-run of graph discovery
with a mutated view. In particular, the test does not claim to cover traversal cycle detection.

The patch also applies the three root-reported redundant field-name cleanups (`pool: pool` → `pool`)
at the corresponding construction sites, without an allow attribute.

## Confirmed coverage and remaining gaps

The existing full compound proof test covers frame-boundary, case-entry, and wrong-join refusal, plus
proof/integration budget, pre-cancellation, and rollback. The two-payload source fixture covers a real
positive with unique ordinary payload arms and one shared join. The old switch helper's edge mutations
remain unrelated and are not counted here.

This candidate adds direct coverage of compound canonical-edge closure for external entry into a
non-join payload, a repeated incoming row to an internal test, and a non-Normal row incident to the
shared join. It still does not cover a real cycle entering the traversal gray-node guard, clone/path
mismatch in a compound test graph, or a real irreducible/non-series-parallel compound graph whose
condition cannot be reduced. `CanonicalCfg` storage is crate-private to `jarde-jvm`; a `jarde-java`
test cannot mutate a published graph in place. The reader has no public graph override seam, and no
existing Java source fixture observed so far supplies these exact shapes. Such cases need separate
reader-produced classfile fixtures or a separately approved test seam; they must not be described as
covered by the row-validator mutations.

No workspace file was changed. No Git, Cargo, JDK, compiler, or formatter was run. This candidate is
uncompiled and root must apply/review and run the focused tests before assigning coverage credit.

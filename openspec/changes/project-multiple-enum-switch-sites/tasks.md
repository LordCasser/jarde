## 1. Group same-method enum projections

- [ ] 1.1 Replace the multi-site blanket refusal with a grouped path that consumes the existing per-site proof candidates without broadening proof discovery or dependency reads.
- [ ] 1.2 Apply every site's labels to one clone of the same recovered method AST, validate every BCI and selector, emit one whole-method body, and stage it only after the entire group succeeds.
- [ ] 1.3 Keep per-site proof evidence, markers and failure reasons; on any proof, AST-shape, output-budget or cancellation failure, leave the whole method on the original integer path.

## 2. Verify the bounded slice

- [ ] 2.1 Add the DT-31 two-site positive fixture: distinct enum types/tables in one method; assert both switches use enum labels and the complete source compiles with `javac --release 8`.
- [ ] 2.2 Run original and reconstructed source with `java -Xverify:all` over every enum value and each null selector; compare return values, exception class and observable side-effect order.
- [ ] 2.3 Add a two-site partial-proof negative fixture; force one site's mapping proof to refuse and assert that neither site is projected and the reason is attached to the affected candidate.
- [ ] 2.4 Keep single-site projection byte-for-byte stable and verify independent method recovery still exposes integer cases; compare essential/all success text and cover cancellation or insufficient output budget without partial publication.
- [ ] 2.5 Run focused Java/class-source tests, formatting and required repository checks; review the proof records and full-source replay before completion.

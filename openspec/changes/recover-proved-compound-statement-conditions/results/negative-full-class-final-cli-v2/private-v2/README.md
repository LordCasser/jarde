# Compound negative whole-class render collector v2

This private v2 has not run the candidate CLI, `javap`, or any Java target method.

`collect.py` requires explicit `--cli`, `--sha256`, `--build`, and `--out`. It verifies the final build record, live product pins, fixture class pins, and the saved BLAKE3 oracle execution/raw/tool pins. It captures each fixture's complete `javap -v -p -c` output and runs whole-class `class-source` once with default evidence and once with `--evidence all`. All command stdout/stderr is retained. No Java target method is invoked.

For each profile, the collector records `physical_source_coverage`: exact equality between the set of target report origin BCIs and that class's javap instruction BCIs, with at least one origin required; duplicate anchors are allowed. A missing/partial map is recorded as `complete: false`; `verify.py` rejects it. Every present class or method source-map segment is checked as a UTF-8 byte range against the corresponding actual `doc.text` or method `report.text`, including bounds, codepoint-boundary and slice-decoding checks. The verifier recomputes these checks independently. It binds all API method identities and target origins to the root-provided BLAKE3 digest, byte length, and `standalone_root` snapshot.

The API contains eight members including the private constructor, hence seven nonconstructor methods. The verifier compares default/all full-class text and source maps, while recording the CLI outcome without requiring `Structured` or `ExplanationOnly`.

`development-dryrun-v2.json` records read-only historical schema/helper checks plus validation of the pinned BLAKE3 oracle record. Historical API and javap examples are described separately; they are not candidate CLI evidence.

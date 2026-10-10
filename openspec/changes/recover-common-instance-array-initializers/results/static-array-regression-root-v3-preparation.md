# Static-array regression replay V3 — preparation

V3 is an independent successor to V2; V1 and V2 remain unchanged, and their evidence directories are not read as output targets. It corrects the too-broad method-document equality: candidate and accepted method facts now compare the complete physical `method.item`, physical `outcome.report.text`, and physical `source_map`, while requiring every physical report execution status to be `complete`. Profile-dependent request/region/evidence containers and usage/elapsed counters are deliberately excluded.

The V2 field comparison remains: physical `item`, annotations, markers, and type annotations are compared while derived `declaration` text may differ; the static initializer proof must still match exactly. The default/all generated full class texts must match. Exact class paths include any package directory. The eight original-runtime candidate legs and raw/oracle capture are unchanged.

Preparation only; no script or toolchain was run. Root should execute V3 into `static-array-regression-root-v3/` after review.

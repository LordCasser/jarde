# Own-CI verifier v3 static review

Status: private candidate for root-run acceptance; verifier not executed. No product workspace, Git state, Cargo/toolchain, or build outputs were changed.

## Narrow correction

v2 failed because `verify_named_manifest` required every Cargo metadata source path to be in the separately reconstructed 4732-path historical local closure. The existing closure deliberately does not inventory the seven `examples/*.rs` files although Cargo metadata includes them as workspace example targets. v3 leaves `source_paths`, `pin_sources`, workspace receipt pin equality, Git/live closure comparison, and CI seed comparisons unchanged.

For named target rows, v3 still requires exact metadata source path equality. Sources in the historical pin map must match both the recorded pin and live bytes. The only absent-pin exception is the exact set `examples/{bulk_scope_sweep,class_source_file,demand_workloads,inspect_class_header,navigate_artifact,resolve_and_analyze,task_operations}.rs`; each must be metadata kind `example`, repository-relative canonical POSIX spelling, resolve to the same in-root path without symlink traversal, exist as a regular file, and have selected product Git blob SHA equal to live file SHA. It then requires the set of metadata-unpinned sources and all metadata example sources to equal that exact set. The output reports each supplement SHA and explicitly states `historical_local_closure: false` and `historical_pins_modified_or_synthesized: false`.

## Adversarial review

- Extra or substituted path: rejected by exact set and target metadata checks.
- A test/bin/lib target routed through supplement: rejected by metadata `kind` check.
- Symlink leaf or parent traversal: rejected by `is_symlink` and resolved-path equality.
- Changed local example that is not in product commit: rejected by Git blob/live SHA equality.
- Changed historical source or changed closure count: existing before/after, independent closure, live and Git checks remain strict.
- Historical pin fabrication: no pin map is edited or extended; supplementary hashes live only in the final named-manifest result.
- Own CI seed evidence: `verify_ci_seeds` function is byte-identical to v2.

## Limitation

This correction establishes source identity for the seven Cargo example targets against the selected product tree and live checkout; it does not claim those paths were covered by the historical local closure. Root must run the v3 acceptor against the completed evidence and independently inspect its outcome.

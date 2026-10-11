# Static review — for-condition clean-main audit v1

- Audit script SHA-256: `b240ce95a0357dd5e0e22acf1e0a07fb25e42d51f0c3b8ae6d3ba6f9f4e22415`.
- Python syntax and symbol-table construction both pass using `ast.parse` and `symtable.symtable`; neither executes the audit.
- Base: `/private/tmp/jarde-cf11-delivery-prep-luna-v4/clean-main-audit.py` (its pinned SHA is recorded in `pins.json`).
- Root-v2 lint/build runner pin: `/private/tmp/jarde-for-condition-final-lint-build-root-v2.py`, SHA-256 `cd2bf4a5b507b029b80227fd98bc8763a5dd5290afea668ee15770a2fc85dda2`.
- Guard and scanner are pinned to their existing source SHA-256 values. CLI identity is the requested path/bytes/mode/SHA. The CLI path and build receipt did not exist during this preparation.
- Receipt SHA is a mandatory runtime argument. Receipt checks require schema `for-condition-final-lint-build-root-v1`, `passed_candidate_build_only`, matching runner-before/after SHA, candidate identity flags, clean target, 4718 source pins, and equal before/after source-pin maps. It deliberately does not read `source_pins_after_format`.
- The five command rows are compared to the root-v2 runner contract, including the exact workflow-derived clippy argv; all five must have exit 0 and no guard stop.
- Existing v4 Git proof is retained: actual runtime HEAD equals supplied HEAD/main/origin/main; local/remote main proof; each frozen source path is a blob in the expected tree and matches the live blob OID; raw Git streams and stdin digests are retained.
- Exactly 15 worktrees are required. Each gets a porcelain status capture, an ancestry query, and `target`/`fuzz/target` existence checks; all must be clean, ancestor, unlocked, and not prunable. Non-root worktrees must be detached. Free space threshold remains 5 GiB.
- Status: `prepared_only`; no audit, Git, toolchain, or CLI execution occurred. Product/docs completion and final receipt are still required before root runs this audit.

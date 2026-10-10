# Conditional boundary baseline preflight v2

This is a private, unrun full-class preflight draft for the five method shapes in `private-conditional-switch-boundaries-luna-v2`. It uses the already frozen **typed** CLI/build as the baseline, while the conditional-switch product remains unapplied. It does not use a future conditional CLI or claim a CFG/product result.

The collector pins and reads the actual typed artifacts:

- CLI `/private/tmp/jarde-proved-local-source-types-cli-v1`, SHA-256 `e6978d74d0935621e73db7d2640fc5a545e4ddfd4c090027ebb84930b5419403`, mode `0555`.
- Metadata `openspec/changes/recover-proved-local-source-types/results/candidate-cli-typed-root-v1.json`, SHA-256 `6295d4cc0c65bd5b5c476f71e44cf3abf513f77d854854bfdea777cbaf2be943`.
- Build execution `openspec/changes/recover-proved-local-source-types/results/validation-build-root-v2/execution.json`, SHA-256 `18a9750cdeb0d8bc45786f5d0766cbb256f1c8a87b1daf350965e910535a6413`.
- Source base `5c2c06f1ec8c3ff0560f2c2d89059ee7d6d06b02`; execution status `validation-passed-cli-frozen`; runner `run-validation-build-root-v2.py` SHA-256 `75e3b3494bf6a2f3179fc1d61e01865382cbdcb4bcefc527f9bc97f012687b87`.
- Guard is the absolute v9 template path from those frozen artifacts, SHA-256 `51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33`. The v2 loader binds `ROOT`, `OUT`, and `command_stream` through `module.run_command.__globals__` after `importlib` module loading; bytecode writing is disabled during this load.

The bound product/test/canonical pin maps contain 17, 10, and 50 paths. A read-only check found all current live hashes equal metadata, and the build's `source_pins_before` and `source_pins_after` equal those maps. The JDK manifest is hash-pinned; each `java`, `javac`, and `javap` path and SHA comes from that manifest and is checked live. JADX is pinned to 1.5.6 and compared using its resolved Cellar path, not the `/opt/homebrew/bin` symlink spelling.

The report schema was checked against an actual saved full-class report at `/private/tmp/jarde-cf12-post-pop-baseline-root-v4/reports/TestSwitchLabels.test/javac23/all/TestSwitchLabels$TestCls$Inner.json` (SHA-256 `6b837b671f78cc95b7a587b58063573e0c388998f06c63940bfbd15e4590de05`). A `methods[]` row has `item.identity`, `item.name.raw`, `item.descriptor.raw`, `item.index`, and `outcome.report.source_map.segments`. Each segment's `origin.primary` is an object and `origin.derived` is a list; each origin carries its physical identity in `origin.*.method` with raw `name`, `descriptor`, and owner. The collector requires and retains the enclosing exact ordinal/name/descriptor/owner, and validates the origin `method` identity and BCI against it. Missing identities or missing source maps fail the observation instead of becoming empty maps.

The collector requires a new exclusive `--out` directory. It compiles the complete class and unchanged Runner under both pinned JDK legs, captures original runtime and javap output, runs complete-class JADX decompile/compile/runtime, and independently runs the typed CLI for full-class `default` and `all` reports with complete-source compile/runtime. Candidate and JADX failures remain recorded; a JADX stage failure does not skip candidate profiles. Each command is run through the v9 guard and checkpointed immediately with exact argv, UTC start, duration, exit, guard state, and raw stream hashes. The final status is complete only when the full command matrix, both original oracles, both candidate profiles per JDK, source/map equality, and all starting/ending pins pass. An interrupt or exception is explicitly incomplete.

After root confirms the pinned typed inputs remain current, invoke the collector with a fresh path, for example:

```text
python3 /private/tmp/jarde-conditional-boundary-preflight-luna-v2/prepare-boundary-preflight-root.py \
  --out /private/tmp/jarde-conditional-boundary-preflight-root-v1
```

The output is only a baseline observation. It does not accept conditional-switch recovery, infer bytecode edges from Java source, or substitute candidate output as oracle. The default output used by v1 is not reused; v2 requires `--out` and refuses an existing path.

Only read-only schema/hash checks and Python AST parsing were performed. No JDK, JADX, CLI, Cargo, or Git command ran. See `static-schema-preflight.json` for the exact checked fields and hashes.

Collector SHA-256: `027a530e8e34b2ef3f22c9ebdce0f9d429cdd887bb49dbf4b5f635e00f2d3dfe`.

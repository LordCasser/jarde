# CF-07 candidate replay preparation v1

历史私有版本，root 静态拒绝，未执行：所抽取的 helper 不在 CF-07 collector 顶层，且最终结果漏查对照失败。禁止使用本版入口；当前版本见 README-cf07-candidate-root-v4.md。

`prepare-cf07-candidate-luna-v1.py` prepares an isolated replay at `results/cf07-candidate-root-v1`. It reuses the already accepted CF-07 baseline's `Recorder`, `compile_run`, `parse_javap`, and source helpers by loading only their pinned function/class definitions with AST. It never loads or calls the old collector `main`, and the old CLI is not a fallback.

Before invoking Java tools or the candidate CLI, the script closes and SHA-checks all 118 files in `cf07-loop-latch-baseline/baseline-root-v2`, checks the fixed source, Runner, JDK, JADX, candidate executable, metadata, and both candidate/test source-pin groups. Missing new CLI or metadata causes an explicit refusal. The replay then creates fresh original, JADX default/none, and candidate default/all whole-class compile/runtime legs on both fixed JDKs. It records full method BCI deltas, exact owner and UTF-8 span bindings, and requires the default/all text and source maps to match. The in-scope `andWhile@15` and `counted@30` must be mapped; `counted@20` and `lastIndexOf@25` remain separately reported as out-of-scope anchors.

Root acceptance steps after the new CLI and metadata have been frozen:

```sh
uv run --with blake3==1.0.11 python3 openspec/changes/preserve-proved-loop-latch-origins/results/prepare-cf07-candidate-luna-v1.py \
  --cli-sha256 <actual-frozen-cli-sha256> \
  --metadata-sha256 <actual-frozen-metadata-sha256>
```

Then independently review `cf07-candidate-root-v1/manifest.json` against the frozen inputs and raw streams. This preparation file has not been run; it does not report candidate acceptance or task completion.

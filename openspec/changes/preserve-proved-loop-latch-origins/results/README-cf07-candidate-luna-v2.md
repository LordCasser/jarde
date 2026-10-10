# CF-07 candidate replay wrapper v2

历史私有版本，helper 预检通过但完整候选流程未经接受。root 审查发现第二 JDK 断言被提前 return 跳过、javap 空格误比，以及 method_key tuple/string 键不匹配；当前入口见 README-cf07-candidate-root-v4.md。

This wrapper imports the SHA-pinned, already accepted CF-07 collector under a non-`__main__` name, then calls its reviewed `main` with only `OUT`, the new CLI path/SHA, and new metadata path/SHA overridden. Its original 29-command whole-class replay provides the fresh original 2, JADX 4, and candidate 4 compile/runtime legs. The wrapper adds candidate source-map checks: preserve each old method body and every prior origin record; record all per-method BCI/origin additions; bind origins to physical owner, BCI, and UTF-8 span; require derived `andWhile@15` and `counted@30` on their nonempty Jarde `while` spans; compare default/all class text, method text, and source maps. It records `counted@20` and `lastIndexOf@25` as out-of-scope physical anchors without claiming acceptance.

The helper-only preflight loads the collector and its actual helper namespace, checks helper globals, and closes the accepted 118-file baseline. It does not call the collector main or run JDK, JADX, or Jarde. Luna performed this read-only preflight before the new CLI existed; its script-bound record is `cf07-candidate-helper-preflight-root-v2.json`.

After root freezes `/private/tmp/jarde-loop-latch-cli-v1` and `candidate-cli-v1.json`, run:

```sh
uv run --with blake3==1.0.11 python3 openspec/changes/preserve-proved-loop-latch-origins/results/prepare-cf07-candidate-luna-v2.py \
  --cli-sha256 <actual-frozen-cli-sha256> \
  --metadata-sha256 <actual-frozen-metadata-sha256>
```

Before replay, the wrapper verifies metadata source/test/canonical pins, `build_result_sha256` against the exact `validation-build-root-v3/execution.json`, and the execution's runner, pin snapshots, and CLI freeze. Output goes to `cf07-candidate-root-v1`; an existing directory is never overwritten. The resulting manifest and candidate observation are replay evidence only, not independent acceptance or task completion. `prepare-cf07-candidate-luna-v1.py` remains preserved as the statically rejected first version.

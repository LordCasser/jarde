# 当前 foreach 产品自身 CI 验收

产品 `73adae72c6d89e8eba0a34e3cd92fe81e899cb3e`，自身 [CI 38181427452](https://github.com/LordCasser/jarde/actions/runs/38181427452) 已 root 实际捕获、独立接受。四 jobs / 52 steps、双 seed 各 362 targets / 3481 passed / 0 failed / 97 ignored / 3578 named outcomes，全部具名结果与 fresh 本地一致；96 batches、20 required、4744 source closure 同产品 Git/live/最终 CLI 字节绑定。

真实 verifier-v4 exit0，接受结果 SHA256 `c6acfceff814fcf1196f50d82510ebe6a6fcf76ef352a4ca1a823b5f5651aa19`；脚本 SHA256 `0fb8b747766e7bb1d2afe2e827afed3f062aebd5c05bf5d5b98738c16611aeb5`。v3 实际 exit1 是 expected final-build helper set 漏写已固定的 workspace acceptor；原始失败 invocation 保留。v4 只传入现有已核 path/hash、补齐 exact helper set 并禁止 duplicate rows，无生产/历史收据修改。

原始 capture、成功/失败 stdout/stderr、外层调用记录与准确脚本无损保存；仍引用原始外置路径，副本不冒充另一次执行。下一步按实际已推送 docs HEAD 执行 clean 审计。CF11 / 71 / 612 完成数不增加。

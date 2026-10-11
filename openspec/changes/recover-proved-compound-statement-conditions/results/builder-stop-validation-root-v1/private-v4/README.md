# Compound Builder Stop tests — private v4 pins

只对测试断言作 pin，基于 root 当前源 SHA-256 `a0c0f139b4d4eb4c347fec5b880c11b679f595eefe6f8a2249c701dea0756914`。proof 固定高预算 usage=52、Stop limit/实际 usage=51；emitter 固定 preflight usage=52、总 usage=68、Stop limit/实际 usage=67。预算 Stop 仍检查 `AnalysisSteps`、`written=0`，并固定 `at=Some(59)`。

取消注释明确它在 prepare 成功后、进入 region emitter 前发生；不表示异步运行中的晚期取消。未改生产逻辑，未编译或执行，未修改 workspace。

- base SHA-256: `a0c0f139b4d4eb4c347fec5b880c11b679f595eefe6f8a2249c701dea0756914`
- candidate SHA-256: `2cc8c5af0c7e27e75ba587d4d63532c09b9eeb2ada09f353a6ffb96e0a4335e7`
- patch SHA-256: `6d7fc892de9041b0088d5668df2c8f6c27932be75482993ea0bee417217cf0b8`

# Root 独立验收（2026-10-07，合并）

## 判据逐项

1. **门控复核**：`results/01-refusal-chain.out` —— OP2 拒在消费者白名单（`opcode=0x99 JumpIfZero`，形状判据全过）；门控 37 输入：5 移动（本片 fixture+OP2）、32 逐字节（含三既有消费位锚、scv 全家、负例）。
2. **diff 审查**：`build.rs` **+12 纯增量**（`boolean_position` 的分支臂）——本会话最小生产 diff；其余判据逐字。
3. **两处测量更正追认**：(a) `while` 位不是本臂可交付——读落在 loop header（自身 canonical 块）→ 跨 region 判据**先于**白名单拒（gate 注释本就 defers 该形），冻为负例、spec 改正为独立 scenario，归属 local-scope 的 elevation 片 ✓；(b) mid-chain 读**被本臂准入**（非先拒）呈嵌套 `x > 0 && (b && x < 100)` 源序——冻为正面锚 ✓。三元臂无需额外机制（任务书预警的独立发现不存在）。
4. **root 实测**：定向 5+3 ignored 回放绿；oracle ignored 3/3；OP2 main 行为对照经 driver 替换处置（main 是他族登记残差——`jarde_refused_body()` 标记恰 1 处断言 + 冻结源 driver 注入），双腿 `…/true/true/1` 一致。
5. **门禁（权威口径）**：全量初跑 336 ok + 1 FAILED = `p3_two_exit_return` scratch `AlreadyExists`（handoff 已登记家族），单测复跑 ×2 绿、复查全量 **0 FAILED**（337 ok 轮在实现片记录）；fmt OK；CI 逐字 clippy `Finished` 0；openspec **309/309**。
6. **corpus**：moved=5（fixture+OP2）；census `(873,3801,332,2359,8)→(879,3829,334,2445,8)`（插桩逐成员计量）；指纹 6+8 纯新增。
7. **CI**：合并推送后 run 为准（监控在案）。

## 裁定（实现者提请项）

- **while 位归属 elevation 片**——采纳（测量支撑，负例冻结）；
- **`if(flag){b=chain}else{return} return b?x:-1` 形恢复**（三元移入 then 臂，语义同）——**接受为未钉观察**（非第四锚；后续巡查如需再冻）。

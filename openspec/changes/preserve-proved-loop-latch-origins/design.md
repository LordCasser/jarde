## Context

见 proposal。root 已实际生成普通控制基线31命令/111文件，原2/JADX4完整类成功，Jarde4因另三个单臂 fallback 缺 return 而编译失败、零runtime。noPrefix 双 JDK/default-all 方法报告均 structured，但原 javap 的 goto@14 缺 map；只确认报告与来源缺口，不声称该方法已有完整类运行验收。独立基线 verifier 历史 v1/v2 的实际 schema 错误保留，准确接受须以后续 root 实跑为准。

root 读审 region.rs 的 header_tested_loop 两个成功构造路径、loop_body_sequence、walker 同 loop boundary、Region 字段，以及 build.rs loop origin fold、emit.rs replay。body 在回到当前 header 时作为 Straight run 停止，terminal transfer 没有显式 AST statement；普通构造未登记其来源。现有 gateway_origins 的语义就是由 loop 结构代替语句的隐藏 transfer，Builder 在真实 loop statement origin 加 derived BCI，Emitter 在非空 span 记录原 physical method，因此接缝已齐备。审计见 one-arm-loop-controls/implicit-loop-transfer-source-audit-luna-v1.md；其“源码正确”措辞只指结构观察，不代替未成功的全类运行。

## Goals / Non-Goals

**Goals:** 收敛到已证明普通 while body 末尾 Straight 的一个准确隐式 latch 来源；不改变 accepted shape、正文、物理 ownership 和外层 continuation。

**Non-Goals:** 通用 transfer/source 重建、多 latch、非 Straight 末尾、for update/do-while/无穷 loop 扩展；单臂续接、其它算术生产者来源债务与任何兼容层。

## Decisions

1. **在已有 loop 成功点补来源。** 从 body 最后一个 Region::Straight 的最后物理块定位 candidate，不扫描所有 goto；该节点必须为本自然循环已证 latch，terminal SSA 指令必须 goto/goto_w 且 Operation::Transfer，唯一正常边精确到当前 header，无未记异常/未知目标。只在既有覆盖成功且 for_header 为 None 的普通 header-tested 路径使用，包含现有 homogeneous header-test chain；已有 exit gateway/update origins 保留并去除同 BCI 重复。候选不满足时不扩形状、不换 source owner，维持既有正文与拒绝范围。
2. **利用末尾 Straight 限定隐式职责。** 如果最后区域是内层 Loop、If、显式 Continue/Break 或 fallback，本片不采集。这样不需要新增 recursive origin census、Region 属性或通用 source pass；跨层 transfer 由原语句处理，不因跳到某地址而冒归当前 loop。自然 latch、准确正常边与 terminal opcode 三者同时成立，不按地址顺序或仅凭终端 BCI 判断。
   root 相邻读审还发现 `loop_arm_join_source` 当前将所有非空 gateway_origins 一律拒绝；该字段同时表示来源而非新增 owner。为保持既有双臂 loop-join 接受，非空时只能沿用同一准确末尾 latch 来源证明，且 origins 必须恰为该单一 BCI；原完整自然循环 owner/唯一入口出口/closed body/Frame 门禁均保留。exit gateway/未知额外 origin 仍拒绝，不能简单删除非空条件。另一 `two_level_loop_join_sources` 专用于 Endless，两者的形状不重叠，本片不改它。
3. **保持预算及阶段边界。** 复用现有 poll/charge，candidate 指令与边扫描有界且每个实际检查计入既有 AnalysisSteps；Stop 原样传播。解码、Java8 dialect 验证、RuntimeProfile/JDK 选择均不变，本片只处理恢复后来源。Builder/emitter 无需改算法或可选 evidence 规则。现有 Runtime/标准类型足够，增加库无法修复 missing source 登记，故无新依赖或许可变化。
4. **以真实物理来源验收。** 永久测试复用冻结 PlainOneArmLoops class bytes 和相邻 loop-gateway recovery helper，noPrefix 所有原 BCI、BCI14 非空 while span、physical method 身份、唯一 block owner及默认/all一致。既有跨层 continue 和 gateway 反例检查来源未迁移；预算/取消检查零部分发布。候选 CLI 对照完整原控制仍必须如实保留三个拒绝/四编译失败，直到独立单臂修复完成；不编辑生成源码来获得单独 noPrefix 运行假通过。

## Risks / Trade-offs

- [隐藏 goto 被误归显式 continue 或内层 loop] → 只取末尾 Straight 的本 loop latch，准确 terminal transfer/唯一正常目标与既有 owner 同时核，不递归抓所有 BCI。
- [仅修了来源却改变正文或 optional evidence 行为] → fresh CLI 逐方法文本与冻结基线逐字比，default/all 正文与 source map 比；其余方法 map 恒同，noPrefix 只允许真实@14派生来源增加。
- [全类仍失败被当成修复失败或完整成功] → 单列 method 来源接受与全类 observation，最终整类运行依赖单臂片，再核完整物理 BCI，不降低门禁。
- [预算扫描新增未计量工作] → 每条实际候选/边检查沿用预算 charge，永久停止测试及相邻回归；机器20GiB/本仓target1GiB守卫满足后才运行Rust。

无需数据迁移或新实体。root 应用并实跑前，Luna patch 仅为候选；本片来源 spec 不代表覆盖其它 loop 形状。

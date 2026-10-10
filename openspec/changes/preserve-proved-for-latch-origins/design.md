## Context

动机见 proposal.md。上一片 fresh replay v3 实跑 57 命令：20 个完整类编译运行腿全部一致，八份 Jarde 生成类中原形全部 BCI 完整，Plain 三个 `(ZI)I` 方法仅缺 goto@20。独立 full verifier 已真实拒绝，不把观测成功当全来源接受。

region 层的 ForHeader 已证明唯一 latch、update_block、update_bci、phi 与 induction 更新。implicit_tail_latch_origin 当前排除所有 ForHeader。Builder 从 Region::Loop 的 gateway_origins 折入 LoopStmt 的 OriginSet，发生在 for/foreach 投影之后，能够直接承载此来源；无需 emitter 新字段。

## Goals / Non-Goals

**Goals:** 保持结构与正文不变，闭合已有 for 的准确回跳来源；同一 helper 同时服务循环创建及单臂 join gate，减少重复证明规则。

**Non-Goals:** 不扩大 ForHeader 候选集合、不改 normal/exception view、不替换 canonical CFG/SSA，不新增 Frame、公共 IR、pass、通用回滚或来源补洞机制。CF07 counted 的 if 汇合 goto@20 与 lastIndexOf 嵌套 else goto@25 是不同方法/证明，独立记录。

## Decisions

1. 扩展现有 implicit_tail_latch_origin，而非新建 for 专属实体。所有普通 while 既有前提保持：最后一个 body 区域必须是 Straight、最后一个 canonical block 必须是自然循环的唯一 latch，真实 terminal 必须是 goto/goto_w 与 Operation::Transfer，Normal successor 只有同一 header，canonical outgoing 准确唯一 Normal 边且无额外异常边。ForHeader 存在时额外核 update_block 就是该 latch、update_bci 与 terminal 前的已证明更新位置一致；不能只删除旧排除条件。现有证明负责更新 slot/phi，不重做整个 for 识别。
2. loop_arm_join_source 允许恰好一个由同一 helper 验证的 latch origin，包括 ForHeader。仍保留 While form 与原自然循环所有权、唯一 entry/exit、join、Frame/path/visited 检查，不能因增加来源把既有单臂恢复重新拒绝，也不能接受其它 gateway 组合。
3. 复用 gateway_origins、plus_derived 与现有 source-map 范围。新增来源指向完整循环语句；不按覆盖缺口给任意语句添加 BCI，不增加 transfer 语句，不动 init/update 已有来源。可复用现有标准 Rust 集合与 Runtime，无依赖安装或复制外部源码的必要。
4. 资源与实际证据分开验收。root 独占生产文件、Git 和工具链；Luna 提交私有 patch，root 全文审查和应用。机器 free ≥ 5368709120、本仓 target ≤ 1073741824，1 秒守卫中止进程组，完成后 cargo clean 本仓并保留冻结 CLI/raw。编译、JVM runtime 与来源证明是不同门禁；JADX 比较保持已钉住的 1.5.6 二进制与 JDK manifest，不把本地开发版测试通过冒充该发布版行为。

## Risks / Trade-offs

- [错误把内部 goto 算作循环来源] → 核自然循环唯一 latch、末尾 Straight、ForHeader 精确 update block/BCI 及 terminal canonical 边；永久反例验证不匹配和 Stop。
- [join gate 排斥新来源导致正文退化] → 创建和 join 共享 helper，永久单臂三方法与 iterator、相邻双臂回归，完整类对照不得删改成员。
- [计费增加使旧预算停止] → 保留既有 charge/poll 与 Result 传播；实际预算/取消测试核无部分产物，不隐藏 Stop。
- [源码能运行却来源仍缺失] → 独立 verifier 重新核所有物理方法/BCI、闭合文件和 default/all 恒同；旧 v3 的缺口及失败 raw 保留。

## Migration Plan

先独立接受 v3 的严格观测基线并核旧产品自身 CI；实现后独立冻结新 CLI 与 pins，跑永久及相邻回归、八份完整类与 CF07 对照，再提交推送捕获新产品精确 headSha 自身 CI。只有新完整来源验收才能关闭上一片 3.1，旧 ca43 CI 仅证明旧产品。失败版本/raw 不覆盖；根 handoff 与账本分别记录片段状态，不增 71 单元完成数。

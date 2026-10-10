## Context

动机见 proposal.md。前片产品 1f386686c6a31813254a85fed48d2af0af84a61c 的新 CLI 已独立接受 CF07 完整源码三方对照，唯一未覆盖物理 BCI 是 lastIndexOf@25。root 临时内部诊断 v2 实际 1/0/0，通过后恢复 52 pins，cargo clean 144.8 MiB，target 峰值 153324998 bytes；原 v1 的 E0277 与修正 raw 保留。证据见 results/diagnostic-acceptance-root-v1.json。

同一次真实分析给出 Loop(header5, While, for_header=None, gateway_origins=[])，body 最后直接 If(branch16, join=None)，then=Straight[19]、else=Straight[22]。block22 的 SSA=[22,25]，terminal goto25，完整出边=[Normal→5]；自然环 blocks=[5,11,22]、唯一 latch22、不可归约=false。回边所有权 then0/else1/If1/Loop1/method1，LoopContinue leaf0。return@21 在 block19，canonical 全边中该返回 block 无出边；返回 block 不属于自然循环但已经由既有终止返回范围和完整 Region walk 持有。

本地 JADX TestLoopCondition5 的 Java 测试断言 for 与两条 return；LoopRegionVisitor.checkForIndexedLoop 从 SSA phi、准确 increment/init 和实际循环使用证明索引循环。Jarde for_header_candidate 现仅允许 preheader 最后 Push(Int)+Store，因此 end-1 的 load/sub 初值不满足；这是独立 for 呈现边界，不能混作回边来源原因。

## Goals / Non-Goals

**Goals:** 在既有隐式尾回边证明中仅扩实际已确认的直接终止 If 形状，新增准确循环 derived origin。保持当前 Straight/ForHeader 证明和所有旧来源、正文。

**Non-Goals:** 不修改 If join、ForHeader 初值或更新、loop form、异常 Frame、返回提升和条件值折叠。不递归搜索 Sequence/Loop/任意嵌套 If，不按全 BCI 差集补 span，不增加通用证明实体、IR 字段、pass、依赖或诊断接口。整 CF07 仍待其 for 呈现和其余有效上游形态扩验。

## Decisions

1. **有限候选提取，证书共用。** 保留 implicit_tail_latch_origin 对最后直接 Straight 的既有行为。新增分支仅在 for_header=None、最后直接 If 且 join=None、两 arm 各为单 block Straight 时考虑；两个 arm 对称，其中恰一侧的 SSA 最末指令是物理 return opcode/Operation::Return，该 block 的完整 canonical 出边为空，另一侧才可成为候选。新增直接形状检查需先计费/poll。任一 arm 为 Fallback、Sequence、嵌套 Loop、非返回/双返回等均不扩大本片。
2. **复用完整回边证明。** 选中的 canonical block 必须通过现有准确 header、非不可归约自然环唯一 latch 成员、末尾真实 goto/goto_w/Transfer、正常流唯一 successor，以及完整 canonical 出边唯一 Normal→header 的证书。不得把 If.join=None 当自由目标，也不得把 If 汇合来源规则用于回边。完整树既有所有权/coverage 审查继续负责物理 block 唯一性，不建立新索引。
3. **返回边验证与原子 Stop。** 返回 arm 由同次 SSA/operations 与完整 canonical 图验证，扫描前按现有 AnalysisSteps 计费，扫描时 poll，Stop 传播不发布部分产物。新增来源仅在所有证明成功后进入既有 gateway_origins；Builder 已把它折入完整 loop stmt，预计无需 Builder 改动。
4. **复用代替依赖。** 标准库匹配和已有 Runtime/IR/OriginSet 足够，无新增库、许可或维护负担。通用递归尾提取会扩大未实测结构与计费范围；新 Frame/IR 没有必要。
5. **冻结和验收。** 新 CLI 与所有 product/test/canonical pins 独立冻结，原 source base/uncommitted 身份按实际记录不回写。CF07 双 JDK/default-all 完整类原样重编运行，并独立重新解析全部物理方法/BCI/owner/span；只允许四 profile 的 lastIndexOf25 derived 完整 loop 新增，其余正文/map 恒同前片，包括 counted20。旧 For/Postfix 完整控制原样重放。所有失败 raw 与版本保留。

## Risks / Trade-offs

- [内层回边或 If 汇合边冒充 loop latch] → 限直接末尾形状，保留 exact natural loop/header/latch/full outgoing 证书，并以真实字节码反例拒绝。
- [正常流投影隐藏 exception] → 返回和回边检查完整 canonical 出边，不只检查 NormalFlow。
- [条件链调用同一 helper 造成意外扩宽] → 两个现有调用均保留原条件证明；只接纳上述直接终止 If，不递归或跳过 join/return 门，旧 Boolean 链与 for 控制完整回归。
- [全来源被误计为语法追平] → computed-init for 是明确独立呈现债务；71/612 与整单元状态保持。
- [资源超限] → 用户已批准 5 GiB machine free/1 GiB target，一秒进程组守卫，串行 root 工具链，完成 cargo clean，保留冻结 CLI/raw/source/class。

## Migration Plan

先独立接受前片精确自身 CI38068627541及 clean 交付，再 root 应用 Luna 私有已审最小 patch；新永久测试、局部与完整类对照、自身精确 CI通过后更新账本/handoff，提交推送全部修改，实核 main/origin/工作区及无待合入分支占用。诊断与规划不等于生产完成。

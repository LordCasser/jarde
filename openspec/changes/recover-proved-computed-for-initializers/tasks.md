## 1. 精确诊断与基线

- [x] 1.1 在冻结 lastIndexOf 的同一次真实 IR 中确认初值 store/read/def/use、header phi、所有 induction-slot 读写、自然环/return leaf/frame、准确 update/goto 与已生成初始化语句。验收：root 实际诊断 raw、class/source/tool pins 与现有 reader/API 恒同，不凭 Java 文本推断 SSA。
- [x] 1.2 固定原/JADX/Jarde 完整类基线及副作用边界；复用已有 fixture 机制构造合法 pending-stack producer/独立调用和共享值 class。验收：root 双 JDK `-Xverify:all` 确认原 class 合法，记录调用/异常顺序及 materialization 或准确拒绝；不把手造 IR 当可验证 class。

## 2. 既有证明与消费扩展

- [x] 2.1 Luna 私有候选，root 审查后复用准确 preheader Store 的既有 SSA/语句消费，取消 Push(Int) 专用门并保持唯一消费者、phi/update/类型/上下文约束。验收：真实 computed-init 正例与额外消费者/上下文反例通过，不新增 expression walker/purity 表，不重排或重复 producer。
- [x] 2.2 在准确 frame 下提前计算一次已有 terminal-return 集合，允许其中仅 induction-slot 读取，并复用于 body frame。验收：真实 return leaf 准入，外部写、其他外部读、额外前驱仍拒绝，CFG/natural blocks 与 exactly-once 所有权不变。
- [x] 2.3 扩展既有最终 direct return/latch If 来源证明至准确 ForHeader update block。验收：goto@25 绑定完整 For span，准确 update/return/完整 canonical 出边与唯一自然环都核对；错目标、额外边、嵌套或 multi-block tail 不获新来源，原 Straight/while 控制保持。
- [x] 2.4 用永久真实 reader/API 与 IR 回归覆盖 for 正文、所有物理来源、初始化声明/赋值作用域、效果物化及可靠拒绝。验收：一条 for、两条 return、准确 return -1；pending/shared/throwing/handler 控制不重排或重复；ForBoundaries 的 observedAfter/twoUpdates/varyingBound/noUpdate 及 foreach/loop-transfer 控制保持。
- [x] 2.5 核预算与取消原子性。验收：实际 Stop dimension/location/usage、准确费用与取消阶段可追溯，没有正文/map/循环半产物，不以预取消代替证明内停止。

## 3. 完整对照与产品交付

- [x] 3.1 root 冻结实际候选 CLI，双 JDK 运行完整原/JADX/Jarde 类与固定 runner，核 default/all、所有物理 BCI/identity、UTF8 span 以及新增效果输入。验收：实际完整源码原样重编/运行，逐条 raw 结果一致；单独记录 JADX/环境失败，不称整个 CF07 或 71 单元完成。
- [x] 3.2 按 5 GiB free/target 1 GiB 一秒守卫执行格式、适用库与 integration 测试、同 CI Clippy、必要 workspace 检查及 OpenSpec strict。验收：真实源码 pins 前后相同，准确命令结果通过，冻结 CLI 后 clean；不借旧版本结果。
- [x] 3.3 提交推送实际产品并验收其自身全部 CI job/step/测试，再更新 ledger/verification/handoff 完成 clean main 交付。验收：精确 head、实际 CI raw 与分支/worktree/target 状态，不关闭尚未实际接受的任务。

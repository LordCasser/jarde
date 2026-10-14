# Delta Spec: Java 8 恢复中的前向条件栈携带值

本 delta 约束已完成诊断与范围裁定的有限前向恢复行为。已有冻结版本的局部证明与完整矩阵记录；workspace捕获的对象receiver回归已加入原断言及物理SSA控制，修复后的最终验证与交付仍开放，change 的完成状态以 tasks 和独立验收记录为准。

## ADDED Requirements

### Requirement: 有限前向同值栈 phi 只按唯一真实 reader 恢复

系统 SHALL 仅对物理 SSA 类型为 `Value::Int` 的整数携带值，在既有 Folded 条件值证明已建立一个有限的单前向 diamond 时，允许该 diamond 汇合处、同一 `Stack(slot)` 上的已退休同值 phi 不再重复计入其 producer 的真实 reader 数。候选 phi SHALL 恰有两个 Value 输入且两者身份完全相同；移除该两条已认证的 phi 输入记录后，producer SHALL 只存在一个由物理指令及 Value/Slot 身份确认的真实 reader。系统 SHALL 继续通过既有呈现路径输出，并保留源码求值次序、方法行为和物理来源映射。

系统 SHALL 将任何结构不完整或未认证的候选保守拒绝。此要求不改变普通 consumer census、通用 SSA use 列表、循环/增强 for、跨块表达式拼接或非本候选值的拒绝规则。

#### Scenario: 同值双输入退休栈 phi 有唯一 reader

- **GIVEN** root 已在完整 Java 8 classfile 上确认一个由既有 Folded 条件证明的单前向 diamond
- **AND** 汇合处的目标 phi 定义于同一 `Stack(slot)`、恰有两个相同 ValueId 输入，并由既有 SSA 事实证明已退休
- **AND** producer 在排除这两条经证明的 phi 输入后只有一个真实物理 reader，相关区间无 handler、回边、其他 reader 或未证明副作用
- **WHEN** 恢复该方法
- **THEN** 系统 SHALL 经既有 Folded 条件呈现路径恢复完整表达式
- **AND** 输出 SHALL 可编译并保持原运行结果、条件求值次序与唯一 reader 的执行顺序
- **AND** source map SHALL 保留 producer、条件两腿语义操作与真实 reader 的物理来源（既有纯 goto 映射缺口独立记录）

#### Scenario: 等价-looking 但未经精确证明的 phi 保持拒绝

- **GIVEN** phi 输入不是恰好两个相同 ValueId，或 phi 仍 live/mixed，或定义 slot 不同，或存在额外 reader/alias/另一 join
- **WHEN** 系统尝试候选证明
- **THEN** 系统 SHALL 拒绝新候选并保留原拒绝/Raw fallback
- **AND** SHALL NOT 通过忽略任意 phi operand、按类型/常量相等合并输入或扩展 reader allowlist 放宽规则

#### Scenario: 异常覆盖、循环或回边不会进入前向候选

- **GIVEN** 候选区间有覆盖 handler、回边、循环携带值或不能证明的控制流边
- **WHEN** 系统分析 producer 到 reader 的可达路径
- **THEN** 系统 SHALL 在产生条件表达式或方法投影前拒绝该候选
- **AND** BI/boolean-loop 的既有恢复或拒绝身份、正文与来源 SHALL 保持逐字不变；新证书 SHALL 拒绝其循环携带形

#### Scenario: 证明预算停止或取消不发布部分投影

- **GIVEN** 预算或 cancellation 在真实候选证明开始后、证明提交前触发
- **WHEN** 报告恢复结束
- **THEN** 系统 SHALL 保留已经完成的 Header/Code/IR 物理事实与此前已提交的 artifact；尚未提交的本次正文 SHALL 遵循既有 NotProduced 合同，不产出 text/map/declaration
- **AND** SHALL NOT 发布部分条件表达式、局部声明、method projection、source map 或候选缓存，也不得在停止后额外执行 raw recovery

#### Scenario: 完整冻结矩阵按各自真值验收

- **GIVEN** 未修改的完整 13 方法 admission class、原 Runner、基线钉定输入和冻结 manifest
- **WHEN** 在双 JDK、debug/none、default/all 的完整 16 个 Jarde cells 与既有 oracle/JADX cells 上执行验收
- **THEN** 所有 Jarde 输出/编译运行与其冻结期望一致
- **AND** 原 baseline 的 45 行、admission 的 33 行 oracle 行数和来源保持一致
- **AND** 编译器/JADX 的成功或失败 SHALL 按各自真实预期记录，不得通过裁剪方法、修改 Runner/helper 或重分类失败来满足矩阵


#### Scenario: 对象 receiver 继续受既有完整链证明约束

- **GIVEN** carried value 的物理 SSA 类型不是 `Value::Int`，包括跨条件的 StringBuilder receiver
- **WHEN** 新的前向整数 carry 证明尝试认证
- **THEN** 系统 SHALL 拒绝本候选并保持既有 construction/concat 的完整链证明和 whole-refusal
- **AND** `ICN.armCall` 的原双编译器负例 SHALL 保持既有正文与拒绝契约

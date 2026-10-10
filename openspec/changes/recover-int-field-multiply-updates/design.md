## Context

参见 proposal 与 `java8-recovery` delta。真实基线位于 `../../evidence/java-syntax-2026-10-10/em23-receiver-chain-next/baseline-root-v2`：33 条命令、126 闭合文件；六条原/JADX 完整类族运行成功，四条 Jarde 完整生成源码在 `jarde_refused_body()` 处编译失败。独立验收仍待执行，不改原 manifest 的失败。

原 javac8/23 `test2(I)V` 均为 `aload_0@0/getfield a@1/dup@4/getfield f@5/iload_1@8/imul@9/putfield f@10/return@13`。`test1(I)V` 则在 BCI1/5 分别读取 a，BCI8 读取 f、12 加、13 写、16 返回。现有 class-family writer 已 `prepared_static/projected`，无需处理新的内部类机制。

`CompoundAssignments::prove_field_update` 已证明准确 `putfield I`、同字段读写、一个 dup 的两份 SSA receiver、唯一消费者、连续 read/arithmetic/store、闭合 receiver 前缀与 RHS 区间。当前算术分支只有 iadd/isub；AST 和 `field_write` 也只有 Add/Subtract。`field::Plan::presented` 通过赋值节点的派生 origin 登记隐含字段读，必须同步认乘法，否则正文恢复后仍会错误报告 read 未发射。

## Goals / Non-Goals

**Goals:**

- 让已具备全部现有位置证明的 `imul` 走相同闭环，不放宽任何身份、消费者、顺序、依赖或预算条件。
- AST 与 emitter 直接表达 `*=`；物理 read、dup、imul、putfield、receiver 和 RHS 的来源完整，class-family 只复用既有拼接。
- 原完整类族及相同 Runner 原样重编运行；另测溢出、null/失败和公开停止边界。

**Non-Goals:**

不引入结构等价 receiver 合并、稳定 receiver pass、赋值规范化框架或新的字段计划；不扩大其它算术、数组/静态/非 int/有结果消费者路径。正向动态验收先限普通非 volatile 字段，不新增或改变既有字段属性规则，也不据此声称 volatile 语义验收。格式缩进债务独立处理。

## Decisions

1. **在准确现有算术接缝增乘法。** `imul` 与 `ArithmeticOp::Multiply` 同时匹配，沿用 `CompoundUpdate::Field` 现有运算字段。增加 `AssignOp::Multiply`、`*=` 拼写与 `field_write` 映射；现有 emitter 已通过 `spell()` 输出，无需新节点。保留旧内部 add-bci 字段命名以避免无关改名，也不扩展 array-update 的门禁。
2. **复用已有证据登记。** 将 Multiply 纳入同一 FieldAssign 的隐含读来源检查，维持当前精确字段名/BCI/member 身份，不新增第二张 read 表。查全 AssignOp 调用方，不能漏 initializer 或 unit-update 的拒绝逻辑。
3. **借鉴 JADX 的结构，使用本项目更明确的 SSA 证明。** root 已核 `SimplifyVisitor.convertFieldArith` 实际 guard：同 FieldInfo、receiver operand 相等，再用原算术创建 oneArgOp。其 wrapped receiver 相等是结构递归，不直接证明两次物理读取相同；本片保留现有同一个 dup 的位置证书，不照搬宽结构相等。基线 test1 保持普通赋值，记录与 JADX 的拼写差别，不把它误记为本片乘法语义失败。
4. **不新增库或外部算法层。** 标准算术、SSA 及 emitter 均已存在，只缺乘法在同一路径的封闭映射。新依赖不能补充这些证据，故无依赖/许可变化。

## Risks / Trade-offs

- [只加输出而不加 read 登记会留下虚假拒绝] → 精确核 read/write presented、完整 OriginSet、物理方法身份与 class-family 的 derived 范围。
- [把同名 receiver 合并会削弱位置证明] → 原显式双读取保持恒同，另检查读写身份不一致和额外消费者仍拒绝；不得用文本包含断言代替来源。
- [乘法溢出或 null 路径只靠正常值测试会漏掉] → 完整原/JADX/Jarde 类族对照及小组溢出/null/effect controls 使用相同 Runner，逐字比 exit/stdout/stderr。
- [预算测试只预取消却称中途停止] → 永久用例如实标注实际停止阶段；已有 budget poll/charge 复用，不虚称 late-stage 注入。
- [完整绿色测试误计成整个 EM23 完成] → 保持 71/612 分母与整单元状态，仅登记乘法窄片；未证明的双读取拼写优化另列。

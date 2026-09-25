## Context

证据与完整阶段见 `../../evidence/java-syntax-2026-09-22/boolean-field-stores/analysis.md`。真实 Fieldref 为 `Z`，JVM 从 int 形状值写出最低位；现有 `field_value` 对无 boolean 证明的 int 拒绝。最近 B/C/S 字段改动已把拒绝来源改为 `quoted_bcis(at)`，但不能让 Z 写入成功。`BinaryOp::Remainder` 和 `BinaryOp::NotEqual`、表达式呈现类型、运算优先级均已有，因此无需等待位运算模块。

## Goals / Non-Goals

**Goals:** 仅对已证明 Z 字段写入的已呈现整数值恢复最低位布尔结果，同时保留调用次数、异常优先级、字段值和来源；对既有 boolean 证明及普通控制不回归。

**Non-Goals:** 不把任意 int 当 boolean，不处理 `ireturn Z` 或 boolean[] `bastore`，不推导整数值域、局部/phi 类型，不优化表达式风格或重构通用 `meeting_position`；被拒方法的整类可编译空操作问题仅在本次能恢复的 Z 写入处消除，其他拒绝仍由独立债务负责。

## Decisions

1. 准入沿用 `field_value` 的真实 `putfield`/`putstatic` 或 verified accessor，且字段 descriptor 精确为 `Z`。已证 boolean 值和 0/1 literal 保持旧 `boolean_spelling`；否则仅当值 `presented` 为 byte/char/short/int 时，可在此消费位置构造 `value % 2 != 0`。未知、引用、long、float/double 保留 `quoted_bcis` 拒绝；不凭 JVM frame 的 Int 形状把无法写出的值放行。
2. 用既有 `Binary` 的余数与不等号节点，`%` 对 int 非零常量 2 不引入异常；对所有 32 位 int，余数为 -1、0、1，不等于 0 与低位为 1 等价。比新设 BitAnd 节点/事实更小，也不修改正在规划的位运算 opcode 闭环。发射器自身的优先级规则负责需要的括号，不能靠拼接字符串绕过 AST。
3. 直接字段写入只在 `field_write` 实际 put BCI 完成转换，接收者仍先渲染，值的 producer 仍在 null 检查前且恰一次；verified accessor 调用共享 `field_value`，其转换属于实际 call site，赋值语句另保留 accessor 内 put BCI 的跨方法派生来源。未验证 accessor 不能获得准入。新节点保留相应消费 BCI 与子表达式真实 BCI；失败走原有 `quoted_bcis`/accessor 拒绝路径，预算/取消/深度走原 AST 发射通道。
4. 永久 fixture 从已冻结 49 Code 的补丁 class 裁出 Z 正面输入或沿原 class 保留 B/C/S 健康控制；源阶段裁剪，不编辑反编译输出。比较原/补丁/JADX/jarde整类各阶段；JADX若 `javac` 失败，只记录失败。固定 0、1、2、-1、极值、producer、null、静态/实例及普通 boolean 写入，逐行核对字段值、次数和异常。
5. 不增加外部依赖。Fieldref descriptor、presented 类型、Binary AST、来源与预算事实已存在；外部求解器不能替代真实消费位置的转换授权。

依据：[JVMS putfield](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.putfield)、[putstatic](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.putstatic)、[JLS 余数](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.17.3)。

## Risks / Trade-offs

- `value != 0` 会把 2、-2 误判为 true；用永久奇偶边界和 JVM oracle 校验 `% 2 != 0`。
- 在构造 boolean 表达式时复制 producer 会改变调用次数、异常和 null 顺序；runner 固定三者及字段旧值。
- 无来源值/错类型被普通 boolean 位置误收；field 与普通调用、返回、数组相邻回归保持各自原边界。
- 整类 `javac` 可能在引用留下的空方法上成功；必须把 95 项执行对照和零引用都作为正面门禁。

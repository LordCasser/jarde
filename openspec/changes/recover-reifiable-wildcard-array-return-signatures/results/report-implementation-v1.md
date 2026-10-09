# report.rs 实现记录

## 变更

`GenericReturnValue` 增加 `ArrayCreation { array_type, allocation_bci, return_bci }`，供同次 class-source Signature 投影消费。`array_type` 保存 `Type::spell` 对应的点分 Java 擦除类型，例如 `java.util.Collection[][]`，不是 JVM descriptor。

`generic_return_candidate` 只在返回表达式为 `NewArray` 时调用新的窄候选证明。候选要求无形式参数、单条完整返回、非 ragged、Code 未停止、无异常处理器、单 SSA block、无 phi，且初始化器 NewArray 的 AST presented 类型、由实际元素类型和 rank 得出的数组类型、物理方法返回 descriptor 完全相同。根 allocation 的解码 Operation 也必须是同元素类型、同 rank 的 NewArray。

证明会按同次 Code 顺序逐项比对 SSA instruction、canonical effect 和 decode Operation，并拒绝缺失或多余的 Operation。除 NOP 外，每个物理 BCI 必须出现在返回语句或完整 NewArray 表达式来源中；新增来源遍历逐节点及逐 anchor 使用同一个 `Budget`，并原样传播取消、预算停止和递归边界停止。

SSA 证明要求 `areturn` 只读一个栈值。该 `ValueId` 必须由 `dup` 产生；每一步复制值都只有唯一 reader，复制输入与输出类型相同，且 BCI 严格向前回溯，直到命中实际根 NewArray 输出。该链不会把别的子数组或另一分配的返回值当成根数组。

## 私有测试

新增 `generic_array_return_candidate_tests`，使用冻结 fixture 的 `javac8` 和 `javac23` `Main.class`，要求两腿都得到 `java.util.Collection[][]`、allocation BCI 1、return BCI 38 的候选。测试通过 `recover_for_class_source_with_anonymous_ast` 的既有 AST-retention 参数取得同次 Program，再直接检验候选证明；普通 `recover_for_class_source` 包装器不保留此 AST。focused-v1 的正候选断言已通过，失败发生在普通包装器未保留 AST；改用已有 retention 入口后的 focused-v2 已通过该测试，两条真实 class 均取得候选。此后新增的 ragged 和多语句拒绝断言尚未重跑。

负控制修改候选输入以覆盖缺失 initializer、错误 presented 类型、错误 return 来源、移除初始化副作用来源后完整物理闭集拒绝、ragged Program 和额外语句。预算为零时，数组 rank 计费先于来源遍历，因此要求 `IrItems` 在根 allocation BCI 1 停止；预取消也要求在 BCI 1 原样返回 `Cancelled`。另一项按真实 anchor、rank 和 Code 成本校准预算，要求 SSA 逆序查找在 BCI 37 内部停止。除上述 focused-v2 已通过的正例外，最新两个拒绝断言当前尚未执行。

## 边界

数组表达式自身的初始化元素、存储顺序、构造 Site、赋值兼容和 handler 闭合仍由既有 Builder/init/facade 同次证明承担。本候选只交接其已呈现的完整根数组形状，不推导 wildcard 语义；bounded、具体参数、不同 rank/leaf、无候选等 Signature 限制由 class-source 实现负责。OpenSpec tasks 不在本实现者范围内更新。

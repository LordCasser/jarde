## Context

见 `proposal.md` 与[独立三方审计](../../evidence/java-syntax-2026-09-25/boolean-array-initializer/analysis.md)。合法 Java 8 补丁类 `BoolInit.values()[Z` 为 major 52、189 bytes；补丁只将方法 descriptor `()[I` 改为 `()[Z`、`newarray` atype `10→4`、五条 `iastore→bastore`，其余 Code 不变。原/补丁 class 在 `java -Xverify:all` 下分别读回 `[0,1,2,3,-1]` 与 `[false,true,false,true,true]`。JADX 1.5.6 输出非法 `new boolean[]{0,1,2,3,-1}`；Jarde 当前在整体 `areturn` BCI 23 引用。root 已独立核对补丁字节、原/补丁 JVM 执行、JADX javac 失败及当前 Jarde 引用。

`prove_array_initializer` 在同一块内已验证分配、每个 dup/index/store、元素 SSA 独占消费、数组组件与 opcode 匹配、共同异常处理器和最终 consumer。循环中同时知道 `stored_value` 与 `store.bci()`，但 `ArrayInitializer.elements` 仅存 `ValueId`；平铺的 `owned` 虽含 store BCI，却不能稳定对应每个元素。`render_value` 遍历元素时将整体 consumer BCI 传给 `array_initializer_element`，因此不能把普通 `array_write` 的低位规则直接复制过来，否则来源会误指 BCI 23。

## Goals / Non-Goals

**Goals:** 只在已有完整证明的 initializer 计划中保留逐元素真实 store 位置，复用最低位 AST，实现可编译且语义/来源正确的 boolean 初始化器。

**Non-Goals:** 不设计通用数组重建 pass，不改变未闭合链、跨处理器或别名/逃逸证明，不放宽普通赋值/调用/局部类型、普通 `bastore` 或 `[B`/`[C`/`[S` 规则。

## Decisions

1. **在现有证明循环配对元素与 store。** 将 `ArrayInitializer.elements` 中每项表示为值与对应真实 store BCI 的有序配对，或在同一计划里保留等价的逐元素 store BCI 列表；只在已通过 `array_store_opcode_matches`、独占消费、区间闭包与 handler 证明后写入。相比从 `owned` 平铺来源倒推，这不会把 dup/index BCI 当 store，也不引入另一套全局分析。既有元素顺序、`element_sources` 与 `owned` 的责任不变。
2. **分开表达式求值位置与写入转换位置。** 保留当前 `render_value(value, consumer_bci, ...)` 取得元素表达式的求值顺序；将配对 store BCI 传给 `array_initializer_element` 的类型适配。已证明 boolean 值/0/1 保留原拼写；仅当组件为 `Type::Boolean`、配对指令为真实 `0x54`、值呈现 B/C/S/I 时调用 `integer_low_bit_boolean(value, store_bci)`。这样每个 `% 2 != 0` 只有一份原值子树，来源包含自己的 producer 与 store，而整体 NewArray 仍拥有分配及 consumer 来源。
3. **保留保守出口。** 证明失败时不构造 `ArrayInitializer`，由原语句路径或既有 fallback 负责；缺组件/值类型、错误 opcode 等不推测 boolean。不得靠把元素先赋进临时 boolean 局部来修编译，这可能改写初始化/异常顺序。新配对作为已有 proof 的数据，不是新预算维度；遍历它仍用既有 poll/charge 与原子提交合同。
4. **JADX 只提供线索。** 本地 `TypeUpdate.arrayPutListener` 从数组传播元素类型，`InsnGen` 的 APUT 直接赋值。其初始化器提取能指出结构候选，却没有编码 boolean `bastore` 的最低位，也没有我们需要的逐 store 来源；Jarde 以已证明闭包和原 JVM 输出为 oracle。只改 source recovery，不碰 parse、方言检查、runtime selection 或 verifier。新外部库无法替代这些本地 SSA/来源证明，且增加维护与许可负担，因此不加依赖。

依据：[JVMS `bastore`](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.bastore) 与 [JLS 整数余数](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.17.3)。

## Risks / Trade-offs

- 把 consumer BCI 23 当五个 store 的来源 → 证明计划显式保留 6/10/14/18/22 的逐元素配对，并检查每个转换片段。
- 只靠 `[Z` 而没有真实 opcode → 配对写入时继续要求原 `array_store_opcode_matches`，消费时再核实际 `bastore`。
- 元素副作用被折叠重复/提前 → 有副作用的 source-only 正例及原/Jarde 严格执行对照；证明不足时维持拒绝。
- 新计划字段改变现有数组初始化器 → 复跑 byte/int/reference 初始化器、来源、预算和完整类回归，不顺带修改其他类型转换。

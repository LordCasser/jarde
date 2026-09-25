## Context

见 `proposal.md` 与[永久三方证据](../../evidence/java-syntax-2026-09-25/boolean-array-lowbit/analysis.md)。原 Java 8 `RawBool` / `Order` 先编译为 `int[]`，仅对目标 descriptor 与 `iastore`/`iaload` 各一条 opcode 做等长补丁，得到 verifier-valid 的 `[Z`、`bastore`/`baload` class。`RawBool` 的原 JVM 对 2/3 分别写 false/true；`Order` 的成功、null、越界、值异常 trace 均为 123。Jarde 当前在 BCI 3/11 可定位拒绝；JADX 1.5.6 直接生成 int 赋给 boolean，完整源码在 javac 阶段失败。root 已独立核对冻结 class SHA、原/补丁字节差异、JVM 输出及两端原样阶段。

`build.rs::array_write` 已从数组自身事实取得组件类型，并用 `array_store_opcode_matches` 检查 opcode。它按数组、下标、值渲染原操作数，最后构造 `IndexAssign`。`Some(Type::Boolean)` 目前仅接纳 `boolean_literal` / `boolean_proven`，而 `integer_low_bit_boolean` 已为 Z 字段写入及 `ireturn Z` 构造 `value % 2 != 0` AST，附消费 BCI 来源。基础设施齐全，缺的是本消费位置的准入。

## Goals / Non-Goals

**Goals:** 精确证明 `[Z` + 真实 `bastore` + B/C/S/I 呈现值；沿单份值 AST 应用最低位转换，并以完整类执行和来源/停止契约验收。

**Non-Goals:** 不从 `bastore` 猜数组组件，不推断 boolean 局部或 phi，不折叠数组初始化器，不改变 `[B` 写入、普通赋值/参数/字段/返回或有问题的 JADX 源码。

## Decisions

1. **准入仅在数组写入消费分支。** 保留 `array_element` 的已证明组件和当前 opcode 检查；`Some(Type::Boolean)` 且实际指令 `0x54` 时，已证明 boolean 值走原 `boolean_spelling`，否则仅在值的 `presented` 为 B/C/S/I 时调用最低位 helper。缺组件证明仍由既有 `None` 路径拒绝，`byte[]` 仍走窄数值 cast 路径。相比放宽 `meeting_position` 或依 opcode 猜组件，这能防止对普通布尔赋值和 `[B` 误授权。
2. **复用唯一最低位 AST。** 使用已有 `integer_low_bit_boolean(value, at)`，让原值只出现一次，并把 store BCI 作为派生来源。其 `% 2 != 0` 对所有 Java int 与 `(value & 1) != 0` 同真值，且除数常量 2 不会抛错。无需新节点、通用类型推断或外部求解库；后者既不能代替数组组件证明，还增加维护、许可证和集成成本。
3. **不移动三操作数。** 继续由 `array_write` 按当前 `render_value(array)→render_value(index)→render_value(written)` 顺序建立 `IndexAssign`，只包住 value。发射的 Java 在 store 检查前求值三者，最低位计算纯且不抛异常，因此 null、越界和生产者异常顺序维持原 JVM。任何无法证明可表达的延期 producer 仍沿 `quoted_bcis` 拒绝，不能通过复制调用强求完整类。
4. **复用既有有界输出。** 来源组合沿 helper 的 operand 原来源 + store 派生来源，array/index 仍为原来的真实来源；默认/all/replay 只改变证据详略，预算与取消仍在原子提交前停止。不要为此新增数组专用来源表或缓存。
5. **分层与 JADX 参考。** 本项只改 source recovery，不改变 class parse、方言检查、runtime selection 或 verifier。JADX 的 `TypeUpdate.arrayPutListener` 会把组件类型传给值，`InsnGen` 的 APUT 则直接打印赋值；可借其“从数组取得组件类型”的定位思路，但冻结结果证明这种传播不足以表达 `bastore` 低位语义。Jarde 沿当前数组事实与 JVM 原 class 运行做更严格转换，不照搬 JADX 输出。

依据：[JVMS `bastore`](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.bastore) 与 [JLS 整数余数](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.17.3)。

## Risks / Trade-offs

- 未证明的 `[B` / `[Z` 共用 `bastore` opcode → 以数组组件事实与匹配检查准入，冻结 byte 控制及未知组件拒绝。
- 把非零误当 true → 正负奇偶与整数极值的原 JVM runner 逐项验收。
- 为写入转换复制或提前调用生产者 → `Order` 的 trace 123、异常类型、单次调用及来源测试同时验收。
- 只验证单方法文字而漏完整类依赖 → 永久 patched class 与生成的零引用完整类分别用同一 runner 在 `-Xverify:all` 下执行；JADX 仅记实际失败阶段。

## Why

[冻结 Java 8 双方法对照](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-field/analysis.md)表明，`(a && b()) || c()` 与 `(a || b()) && c()` 都将三次判断汇入同一个静态 `Z` 字段写入。JADX 1.5.6 的完整类与原 class 16 条值/调用路径相同；Jarde 当前引用正文虽可编译，8 条路径的字段值错误，并重复把汇合点称为循环。现有同极性链只允许每个后续测试有一个前驱，无法描述这里第三测试的双前驱。

## What Changes

- 将现有 `ShortCircuitValue` 的内部候选从线性测试列表推广为有预算的小型无环决策图，允许测试之间有已证明的合流；保留两个 1/0 生产者、唯一 Phi、唯一 `putstatic Z` 的严格消费证明。
- 从解码跳转目标与 canonical 正常边生成惰性求值的现有 `Conditional` 表达式；在互斥 Java 分支重复共享测试文本时逐路径核对调用次数，限制深度及展开成本。
- 对外部入口、异常边、第二消费者、独立效果与不完整图继续原子引用。本 change 在 [逐指令来源](../preserve-region-fallback-instruction-origins/tasks.md)和 [重叠所有权拒绝](../refuse-overlapping-region-ownership/tasks.md)验收后实施，不靠正例覆盖缺陷来隐去负例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对可证明闭合的混合 `&&`/`||` 字段值，输出保持每级短路、字段值和副作用次数的 Java 8 表达式；不可证明时完整拒绝。

## Impact

主要涉及 `jarde-java::region` 的局部短路值所有权、`jarde-java::build` 的 CFG/SSA 证明与已有 `Conditional` 发射。无需新增公开 AST、IR 或全局条件重写 pass。JADX 的分支合并顺序可参考，其在[三元值嵌入 OR](../../evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/analysis.md)上 16/32 条路径错误，不能复制其未经物理边和 Phi 验证的极性决策。

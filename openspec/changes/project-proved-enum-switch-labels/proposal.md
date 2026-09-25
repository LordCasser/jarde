## Why

Java 8 的枚举 `switch` 在目标方法中只留下整数表读取和整数 `case`；常量到整数的关系写在另一合成类的 `<clinit>`。现有 `enumswitch@1` 如实恢复该整数选择，却不能把 `case 1/2` 写为 `case RED/BLUE`。原 class 与 JADX 可执行的完整枚举源码说明有恢复空间；只交换合成表两条整数写入的合法补丁又证明按字段名或 case 编号猜标签会改变程序。

## What Changes

- 在现有方法级枚举表读证明之上，按选定运行环境有界地读取真实表定义、初始化体、枚举常量及 `values()` 数组来源，证明每个已用整数键与一个常量的对应关系及其求值/异常边界。
- 仅在完整跨类证明成立时，将该 `switch` 的源码投影为枚举 selector 和已证常量标签；保留原方法、合成表、物理身份和推断来源供审计。交换映射补丁必须交换标签，而非沿用原字段名顺序。
- 缺失/歧义依赖、额外表写入、未知映射或副作用不能被 `switch(enum)` 隐去，保持当前整数路径或完整拒绝。跨类读取与投影受预算、取消及证据选择约束。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在跨类映射得到证明时恢复源级枚举 `switch` 标签，并保留不可证明时的真实整数分派。

## Impact

影响 `enumswitch@1` 的同次结构化结论、类源码装配和必要的按需依赖读取；复用现有 reader/resolver、Method IR、来源和预算，不增加通用 JVM IR 形状或第三方运行依赖。原/JADX/Jarde 三方冻结证据见 `../../evidence/java-syntax-2026-09-24/enum-switch-labels/`；合成表修改与反射可观察性的广义语义等价不在本项宣称范围内。

## Why

固定 JADX `TestNestedLoops` 已恢复两层 typed foreach 和原名，但 Jarde 完整源码仍输出 `s1.equals((Object) s2)` 与 `l1.remove((Object) s2)`。调用和元素值已有同次分析事实，可以进一步证明哪些包装仅用于保守展示，达到固定黄金用例的源码质量。

## What Changes

- 在已有 List body 投影范围内，证明精确 `String.equals(Object)` 与 `List.remove(Object)` 的源级绑定不变后，移除唯一的展示包装；复用已有物理元素 cast→store→load 证明和 retained AST 事务。
- 将必须的 `add(E)` 类型推断与可选的 equals/remove 清理分开。可选缺证保留包装，不能撤回可靠泛型或 typed foreach；remove 参数不得推断 List 元素类型。
- 用 javac8/23、原源码/JADX/Jarde 完整源码重编译和 JVM 对照验收；保留 primitive int、自定义 equals 重载、真实参数 CHECKCAST、缺证和 Stop 边界。

前置产品 ea2aaf042 和关闭文档 33de3deee 已交付。当前固定正例及展示差距已实际确认；两版平台 API 已核实，独立控制矩阵正在采集，尚不声称其通过。非目标：通用去 cast、完整 Java 重载解析器、新 IR/pass/公开 API、poly 表达式推广、其他嵌套循环功能；不增加 CF11 整单元或 71/612 完成数。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在已有结构化 List body 中，仅对调用绑定和物理来源闭合的 equals/remove 实参移除展示包装；保留真实转换、原始事实及可靠投影。

## Impact

主要为 `src/facade.rs` 的 List body 证明和 staging/final 重验证，以及既有公开回归；复用 `report.rs` 的 opaque AST 包装识别与事务。Builder 的 descriptor-only 入参规则保持原职责。无新依赖，继续使用标准库、现有预算、源码投影和已固定 JDK/JADX 工具；参考 JADX 绑定保护思路，不复制其代码。

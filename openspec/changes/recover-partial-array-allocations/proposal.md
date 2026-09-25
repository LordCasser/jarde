## Why

普通 javac 会以 `anewarray` 或 `multianewarray` 创建只分配前几维的多维数组，例如 `new int[n][]` 和 `new int[a][b][]`。jarde 当前只在已分配维数等于数组总维数时解码创建，合法输入因此引用字节码，整类不能编译。

## What Changes

- 从现有常量池数组描述符保留数组总维数，同时沿用指令的已分配维数；用现有 `NewArray` 表达式写出末尾未分配的 `[]`。
- 保持尺寸表达式从左到右、只求值一次；保留负尺寸、零尺寸与尺寸计算抛错的行为及来源。
- 不新增数组推断 pass、类型求解器或新的求值机制。重载参数的数组协变、数组初始化器与不具证明的数组值仍属其他问题。
- 实施须在共享延迟值顺序修改验收后串行接入；此处仅固定基线与任务。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：由数组创建指令与常量池描述符共同恢复部分维度分配的 Java 源码。

## Impact

涉及 `decode` 的数组创建事实、`NewArray` 表达式的维数、`build` 的数组类型证明、`emit` 的括号输出及相邻测试。根证据见 `../../evidence/java-syntax-2026-09-22/numeric-conversions/partial-array-allocation/`。69 项核心样本原 class/JADX 执行一致；当前 jarde 的整类编译失败，不存在其运行对照结论。

## Why

合法 bastore/castore/sastore 会将已有 int 值窄化后写入数组。当前 array_write 将其作为普通 Java 赋值处理，拒绝了指令已经说明的转换；普通窄局部回读也会遇到同一缺口。

## What Changes

- 在真实窄数组写入位置，以实际 opcode、数组元素类型和已呈现整数值恢复 byte/char/short 窄化。
- 复用现有 Cast、ArrayWrite 与来源，保持数组、下标和值各自的求值次数，以及生产者、null 和越界异常顺序。
- 保留普通同型、合法 widening 和既有常量拼写；不放宽通用赋值或调用参数转换规则。
- 任意整数写入 boolean[] 的最低位语义、窄字段写入、返回转换、显式 conversion opcode 分别处理。本项不做窄局部类型推理、范围分析或数组初始化折叠。
- 实施在共享求值顺序任务验收后串行接入；本案目前只有基线和设计，没有宣称恢复已实现。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：保留已有可呈现整数值在 byte/char/short 数组写入指令处的窄化及求值语义。

## Impact

主要修改 jarde-java 的 array_write 消费位置与相邻测试。既有 SSA 指令携带有效 opcode，数组事实可区分 B/C/S/Z，Cast 和 ArrayWrite 已能表达结果；不新增解码阶段、类型求解器、AST 类别或依赖。独立证据为 `../../evidence/java-syntax-2026-09-22/numeric-conversions/narrow-array-stores/`，JADX 完整源码编译失败必须保持阶段事实，不能作为执行 oracle。

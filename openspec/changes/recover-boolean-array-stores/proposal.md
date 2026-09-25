## Why

合法 Java 8 class 对 `boolean[]` 执行 `bastore` 时，JVM 只把原始整数值的最低位写入数组。当前 Jarde 已能证明数组为 `[Z`，但对没有 boolean 证明的整数值保留引用，因而漏掉可编译且可验证的写入；JADX 1.5.6 对同一输入直接生成 `boolean[] = int`，整类无法编译。[冻结三方证据](../../evidence/java-syntax-2026-09-25/boolean-array-lowbit/analysis.md)同时固定了奇偶值、异常和三个操作数的求值顺序。

## What Changes

- 在真实 `bastore` 且数组元素已有 `[Z` 证明的消费位置，把可呈现的 B/C/S/I 值写成最低位布尔表达式，继续使用已有 AST、来源和预算路径。
- 保留已证明 boolean 值的直接拼写；未知数组元素类型、不匹配 opcode、未知或非整数值继续可定位拒绝，不从 `bastore` 猜测 `[Z`。
- 固定永久 Java 8 补丁 fixture，对原 class、JADX、Jarde 的完整阶段及成功、null、越界和值生产者异常进行独立重编执行对照。

先决条件是现有 reader、SSA、`array_element`、`array_store_opcode_matches`、`array_write` 与已验收的整数最低位 AST helper。本项不处理数组初始化器折叠、boolean 局部/phi 推断、字段或返回转换，也不把 `[B` 的同 opcode 写入当作 `[Z`。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加有证明 `boolean[]` 的真实 `bastore` 对已呈现整数值的最低位写入、执行次序、来源和保守拒绝要求。

## Impact

主要修改 `crates/jarde-java/src/build.rs` 的现有数组写入消费分支，增加永久 fixture、集成测试和 OpenSpec 证据；不改公开 API、reader/JVM IR、依赖或 CLI 分层。

## Why

[布尔混合位运算巡查](../../evidence/java-syntax-2026-10-05/boolean-int-bitwise-patrol/README.md)实证：javac 把布尔位运算的**操作数 int 化**（`!b` → `xor 1` 产 int；`r ^= x` 累积 → int 计数器），呈现层遇到"一侧 int 一侧 boolean"的位运算拒绝："the bitwise operator `^`/`&` … has operands presented as `int` and `boolean`, which no Java integer…"。两形受影响（布尔累积 xor、`a & !b`）；**jadx 两形皆有解**（boolean 累积变量 / `z & (!z2)`）。

同型位运算（boolean^boolean、int^int、Kernighan 循环、超宽移位、常量移位折叠）**全部恢复**——缺口仅在混合型。

## What Changes

在位运算操作数呈现处新增 **int 化布尔的回投**：当一个 int 操作数的**全部生产链都在布尔位运算上下文内**（由 `!`/条件产生的 0/1、或 boolean→int 的累积计数器，且其消费只有位运算与既有 boolean-from-int 出口 `% 2 != 0`）时，把它回投为 boolean 参与呈现——`a & !b`、`r ^= x` 直接呈现布尔形。**判据保守**：int 值若有任何非布尔消费（算术/比较/存储为 int），不回投、保持现状拒绝。

## Impact

- **代码**：`crates/jarde-java/src/build.rs` 位运算操作数呈现处（拒绝文本 "bitwise operator … operands presented as" 发出处，task 1.1 定位）。
- **测试**：`BW` fixture + 同型零回退 + 负例（int 与 boolean 真混合算术形仍拒）。
- **账本**：summary.md 登记行关闭。

## Non-Goals

- **不**做通用 int→boolean 推断（仅限位运算上下文内的完整生产链）；
- **不**碰同型位运算与 boolean-from-int 出口（`% 2 != 0` 既有域）；
- **不**处理 `|`/`&` 短路语义混叠（源写 `&&` 的不会被编为位运算——javac 语义保真，无需处理）。

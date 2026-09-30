## Why

[CF-10 巡查](../../evidence/java-syntax-2026-10-01/cf10-crossing-array-read-patrol/README.md)追踪到账本登记债务的失败路径：`i += 2` 本身已不构成差距（F1 对照完整恢复、正确保持计数循环）；真实缺口是**跨保护区域局部**的写值形状白名单不含数组元素读——`sum = sum + data[i]`（写@15 树含 `iaload@13`）在 `all_reads_reach_presented_writes` 处无法判定为可呈现写值，方法拒绝（F2 固定复现）。该形态（循环累计数组元素 + 体内 try/catch）是真实代码高频写法。

## What Changes

- `presented_int_store_value` 写值白名单新增"同块数组元素读"：数组操作数与下标表达式均落在既有白名单（int 字面量/同块局部 load/静态调用/加法树）内、值单用途、求值位置不变（store 前同块连续区间纪律沿用）；元素类型经既有 `array_of_value`/descriptor 通道。
- F2 完整恢复且行为一致（`14`）；F1 对照逐字不变；共享值、跨块、副作用序、多维/引用数组越界形态不放宽（负例钉死）。
- 不改界门（crosses_exception → 路径证明）与读侧 phi walk——它们的现有行为已被 F2 复现证明是正确的最后一环。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：跨保护区域局部的数组元素累计写值可呈现，循环+try/catch 叠加形态完整恢复。

## Impact

仅 `crates/jarde-java` 私有 build.rs 写值白名单及测试；与 saved-return/array-slot 等既有细化通道同族，无新机制。既有 foreach/数组/finally 切片零回退。

## Why

仓库只有最终架构文档，尚无可执行能力或验收基线。先建立 P0 的事实读取闭环，使后续 Query 与 Decompiler 共用可追溯、有预算的输入底座。

## What Changes

- 建立纯 Rust、同步 library-first workspace 和 JSON CLI。
- 打开不可变 CLASS/JAR/WAR 快照，逐项枚举物理 entry，保留重复名称和原始字节身份。
- 按 entry 请求 Header、按方法请求 bytecode；保留原始 MUTF-8、attribute spans、BCI 和异常表。
- 提供预算、取消、部分结果、诊断和逐能力版本状态，区分结构读取与 JVM verification。
- 固化依赖评估、支持矩阵、P1–P5 后续路线和测试门槛。

## Capabilities

### New Capabilities

- `artifact-snapshots`: 不可变物理快照、有界归档遍历、entry 身份与按需读取。
- `classfile-inspection`: 有界 Header、版本能力、无损字符串、共享 bytecode 解码。
- `analysis-contracts`: provenance、coverage、execution、预算和同步公共 API。

### Modified Capabilities

无。

## Impact

未来实现拟新增根 crate `jarde` 和 `crates/jarde-cli`。底层优先复用 noak、rawzip、flate2、blake3、serde、thiserror、clap；评估版本、准入测试和排除项记录于 [依赖选型](../../dependencies.md)。本 change 不实现 XRef、runtime selection、resolver、CFG/SSA 或 Java 恢复，后续能力分别验收。当前只交付 OpenSpec 文档，尚无 P0 实现。

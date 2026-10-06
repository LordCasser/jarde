## Why

[java-time-temporal 巡查](../../evidence/java-syntax-2026-10-05/java-time-temporal-patrol/README.md)：`DateTimeFormatter.format(dt)`（LocalDateTime→TemporalAccessor）、`Duration.between`/`ChronoUnit.between`（→Temporal）、`LocalTime.parse` 位——java.time 是 Java 8 最高频新 API，全部日期格式化/时差计算被 "no safe reference conversion evidence" 拒。源与目标**都是平台类型且都不在快照**：platform-interface 单边机制（快照 header 举证）与既有表（java.util 集合树/CharSequence/Comparable/Serializable/EnumSet）均不覆盖。jadx 完整解。

## What Changes

按已落地的扩宽表机制（build.rs `platform_interface_argument_widens`，与四表同一函数）新增 **java.time 行集**：以 release-8 javadoc/header 为准的封闭行（MVP：LocalDateTime/LocalDate/LocalTime → `java.time.temporal.Temporal`、`java.time.temporal.TemporalAccessor`；ZonedDateTime/Instant/OffsetDateTime 等按 javap 核对后入表；数组位谓词自动覆盖）。零新机制、既有通道逐字不动。

## 硬不变量

1. 既有表/通道/单边机制渲染逐字节不变；非 java.time 层级成员仍拒；
2. 行集只收 javap 实测的 release-8 header 声明（同 widening-row-sources 证据协议）；
3. 诊断文本零变化。

## 验收

- JT.fmt/spans、DT.direct/parse 恢复（0 引注），剥离编译 exit 0、`-Xverify:all` 行为与原一致（`2026/01/02 03:04 -> 3:4`、`287`、`03:04/03:04`）；JT.basic 零回退；
- 全门禁 + corpus 指纹。

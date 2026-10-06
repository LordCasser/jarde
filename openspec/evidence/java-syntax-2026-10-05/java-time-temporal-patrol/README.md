# java.time 家族巡查（2026-10-06 root，第 151 前沿）

## 探针

[fixture/JT.java](fixture/JT.java)（`--release 8`）：LocalDate/LocalDateTime 工厂+链（plusDays/plusMonths/withDayOfMonth/atTime）、DateTimeFormatter 双向（ofPattern/format/parse）、Duration/Period/ChronoUnit。[fixture/DT.java](fixture/DT.java)：直接参数判别（format/parse 位）。

## 结果

- **basic() 完整恢复**（工厂+链、`plusDays(10L)` long 装箱如实）；
- **fmt()/spans()/DT 三方法 = 第 6 族 platform→platform 新位点**：`LocalDateTime presents ... requires java.time.temporal.TemporalAccessor`（format 实参位）、`→Temporal`（Duration.between/spans）、parse 位——**源与目标都是平台类型且都不在快照**，platform-interface 单边（快照 header 列目标）与四表（java.lang/java.util 系）均不覆盖；jadx 完整解；
- 拒绝面 SAFE（幸存缺 return）。

## 归因

与 DIRECT_EDGES 表族同构：java.time 层级是 javadoc 封闭事实（LocalDateTime implements Temporal/TemporalAccessor/ChronoLocalDateTime<LocalDate>…；LocalDate/LocalTime/Instant/ZonedDateTime 同族）。**窄片：java.time 行入扩宽表**（与已落的四表机制完全一致，零新机制）。

## 处置

窄片 `recover-temporal-argument-widening` 候选；MVP 行集=JT/DT 锚命中的最小封闭族（LocalDateTime/LocalDate/LocalTime → Temporal/TemporalAccessor，先 javap 核对 release-8 header）。

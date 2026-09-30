## Why

[TestFinally 家族巡查](../../evidence/java-syntax-2026-09-30/testfinally-patrol/README.md)的末片 `TestFinally3`（Tf3）`test()` 主线整方法回退（`jre_region_exception_edge`@0 + 5 未覆盖块）。形态是家族内最复合的：null 局部 lead（`[aconst_null, astore_1]`）、受保护正文含**条件流**（`bytes` 字段判空 + `validate()`）与**提前 `return null`**、异常表为**分段两行** `[2,18)→53` 与 `[24,47)→53`（间隙正是早返回块，其 close 副本不受保护）且**无自保护行**、共**三份**同形 `aload_1; invokestatic close` 副本（早返回 18–19、正常 47–48、异常 54–55）。既有证书均不覆盖此组合：Test13 `SegmentedFinally` 是五行/两段/早返回但 lead 与清理形状不同；Test5 `MultiReturnLoopFinally` 无分段；Tf1/Tf2 无早返回副本。固定 JADX Java-input 结构可恢复（其 `inputStream2` 重命名失真仅作参照）；原 class 是行为基准。

## What Changes

- 在现有 FINALLY Guard/Region/Builder 通道新增"分段 null-lead 三副本"有界证书：两行同 handler 的 any 表、lead `[aconst_null, astore s]`、正文条件流与同槽赋值、早返回块（间隙内：值保存 + 副本 close + areturn，其 close 不受任何行保护）、三份副本逐参数同形（`aload s` + 同目标静态/虚调用，实参槽即 s）、两个保存返回（null 与正文值）身份、无自保护行的语义（清理抛错按 JVM 边覆盖）由行表空缺直接陈述；输出唯一 `try/finally`：条件正文 + `if (bytes == null) { …; return null; }` + 一份 `close(inputStream);`。
- 三副本 BCI 全部映射唯一清理体；lead/正文/早返回/正常/异常全部物理块被证书拥有；canonical 边全集恰等。复用 `Plan::lead`、既有 If/条件正文呈现与 Tf1 的 null 出处判据（`local_null_handler_provenance` 形态）；不新增公开 IR 节点、CLI 开关或依赖。
- 以固定 Tf3 类与探针变体做原/固定 JADX/Jarde 三方 Java 8 重编、五路径（正常已缓存、正常未缓存、validate 拒绝提前 null、正文抛错、清理抛错）`java -Xverify:all` 行为对照、verifier 有效负例验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：受证的分段两行表 + null 局部 lead + 早返回副本 + 正文条件流可恢复为唯一 `try/finally`，早返回与正常/异常完成共享同一清理呈现。

## Impact

仅 `crates/jarde-java` 私有 Guard/Region/Builder 及测试；Test13 分段、Test5 多返回、Tf1/Tf2/Tf4 条件与直体家族证书零放宽。上游 `TestFinally3.test2NoDebug` 标 `@NotYetImplemented`（JADX 自身 noDebug 未完成）属分母调整项，本片只证 JVM classfile 形态。

## 1. 固定基线与负例

- [x] 1.1 重放固定 Tf4：核 class/源 SHA、`test()` 双行 any 表、lead 两指令、八指令对称副本与主线整方法回退诊断（`jre_region_exception_edge`@0、5 未覆盖块）；以 patrol 证据的 `Tf4.base.json`、`javap` 与 `java -Xverify:all` 记录可重放基线。
- [x] 1.2 构造并冻结至少六个 verifier 有效负例：无正文置真写、两次置真写/写后含其它语句、条件反转 `ifeq`、两副本 K 或字段不一致、lead 初始化非 0/多指令、副本清理改调用或增删指令；各自 `java -Xverify:all` 通过并记录变更前后的拒绝。

## 2. 标志条件证书

- [x] 2.1 在 Guard 内新增有界 prove：两行 any 表与自保护绑定行、`[iconst_0, istore s]` lead、正文恰一次 `iconst_1; istore s`（保存返回之前、区间之内）、保存返回/`astore`/`athrow` 身份、两份八指令副本逐参数同形（slot/F/K/op）、canonical 边与物理块全集恰等；以固定类命中与 1.2 全部负例拒绝验收。
- [x] 2.2 证明正文（call、`result++`、置真、保存返回）的完整所有权与 SSA 值序；预算/取消原子回滚，不发布半个结构。

## 3. 区域与源码交付

- [x] 3.1 Region：lead 呈现 `boolean success = false;`，正文经既有 finally 通道完整拥有，恰在正常清理前结束；失败回滚 visited。
- [x] 3.2 Builder：折叠两份副本为唯一 `if (!success) { result -= 2; }`，复用既有字段复合更新拼写；两份副本 BCI 均有来源映射；完整类 `javac --release 8` 通过。

## 4. 三方对照与验收

- [x] 4.1 同布局探针变体（`call()` 可注入失败、清理可注入失败）下原 class/固定 JADX Java-input/Jarde 三方 Java 8 重编，`java -Xverify:all` 正常（`result==1`）、call 抛错（`result==-2` 且异常传播）、清理抛错（新异常覆盖）逐路径一致；记录输出 SHA。
- [x] 4.2 回归 Test14 条件清理、Test2/5/9/11/12–17 与本批 preceded-catches 切片；`cargo test -p jarde-java --tests --locked`、workspace check、`cargo fmt --all -- --check`、CI 同款 Clippy（新码零告警）、`openspec validate --all --strict`、diff check，清理专用 Cargo target。
- [ ] 4.3 root 独立复核证书边界、副本折叠来源与三方行为，更新 CF-16 清单与巡查账本；仅标记固定 Tf4（TestFinallyExtract）JVM 切片。

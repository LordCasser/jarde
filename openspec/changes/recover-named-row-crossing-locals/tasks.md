## 1. 基线与负例

- [x] 1.1 重放[巡查证据](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/README.md)：核 fixture SHA、C1.five/C4.constructNamed 基线回退诊断、C3 对照组与 `twrNamed` 基线输出；记录可重放基线。（基线输出与诊断见 [results/crossing/*.base.txt|json](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/results/crossing/) 与[复放记录](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/results/crossing/crossing-replay.md) §1.1）
- [x] 1.2 构造并冻结至少六个 verifier 有效负例：catch-all 行 + 完成 store 前置（保持降级）、行起点劈开构造表达式（栈深非零）、构造与 store 间插入语句、构造结果双用途、跨块构造、无 store 直接消费；各自 `java -Xverify:all` 通过并记录实现前后行为。（`fixture/crossing/X1–X6`，前后逐字节对照见[复放记录](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/results/crossing/crossing-replay.md) §1.2）

## 2. 门槛判别与写值扩展

- [x] 2.1 `guard.rs::resources` 增加具名行-完成-store 判别（design 决策 1），注释按"try 之前是不是本资源自己的初始化"的模块问题表述；以 C1/C4.constructNamed 命中、1.2 负例保持、TWR 家族不回退验收。（判别以 `normal_close`/`closes_something`/可读运行三守卫补足字面条件——TWR 主行可携带 `Throwable` 类型且紧跟完成 store，见判别注释与复放记录 §判别摘要）
- [x] 2.2 `build.rs::all_reads_reach_presented_writes` 写值接受集扩展（design 决策 2）；构造写值呈现为声明初始化器；C1.five 的五连空 catch + 语句式 append 全部呈现，local 0 声明可放置；预算/取消原子回滚。（`presented_reference_store_value`，站点表达式恰好覆盖 store 前整段运行）

## 3. 翻转与回归

- [x] 3.1 N1/P3StorePrefix 期望翻转为正例（恢复为普通 try/catch），同步 `p3_preceded_catches`/guard 单测与 `recover-preceded-statement-catches` 证据中的边界说明（记录结构论证）；catch-all 与其余 7 负边界逐项保持。（[preceded 验证记录第五节](../recover-preceded-statement-catches/verification/post-merge-regressions.md)）
- [x] 3.2 全量回归：C3 对照组逐字不变；`twrNamed` 输出与基线一致；FinallyOnce 全类、Test2 SHA、Tf1–Tf4、dt14 折叠、M1/M2 不回退。（[复放记录](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/results/crossing/crossing-replay.md) §3.2：FinallyOnce 与 `fo.exp.java` 逐字节一致、Test2 SHA `59d5c8ca…` 一致）

## 4. 三方对照与验收

- [x] 4.1 C1/C4.constructNamed 与翻转后的 N1：原 class/固定 JADX Java-input/Jarde 完整 Java 8 类重编，`java -Xverify:all` 正常与注入异常路径逐路径一致；记录输出 SHA。（[results/crossing/three-way/](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/results/crossing/three-way/) 与[复放记录](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/results/crossing/crossing-replay.md) §4.1，八条腿组 SHA 逐路径一致）
- [x] 4.2 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 同款 Clippy 新码零新增、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 4.3 root 独立复核判别边界、写值呈现与三方行为，更新 CF-15 清单与巡查账本；仅标记具名行-完成-store 家族。（root 于合并主线 1196e7ad 复核：C1.five 五连空 catch 完整恢复〔即上游 TestEmptyCatch 的 JVM 形态〕、C4.constructNamed 恢复、twrNamed 与基线一致保持拒绝、全仓 2709/0、fmt/openspec 223/223。实现者在 root 判别上补的三个守卫复核通过：single_statement 可读防畸形程序、normal_close/closes_something 排除多资源 TWR 的具名 Throwable 行〔对 root 结构论证的有效反例修补〕，均有界且由冻结负例钉死。）

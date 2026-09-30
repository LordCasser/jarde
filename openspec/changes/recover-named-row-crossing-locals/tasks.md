## 1. 基线与负例

- [ ] 1.1 重放[巡查证据](../../evidence/java-syntax-2026-09-30/cf15-crossing-patrol/README.md)：核 fixture SHA、C1.five/C4.constructNamed 基线回退诊断、C3 对照组与 `twrNamed` 基线输出；记录可重放基线。
- [ ] 1.2 构造并冻结至少六个 verifier 有效负例：catch-all 行 + 完成 store 前置（保持降级）、行起点劈开构造表达式（栈深非零）、构造与 store 间插入语句、构造结果双用途、跨块构造、无 store 直接消费；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 门槛判别与写值扩展

- [ ] 2.1 `guard.rs::resources` 增加具名行-完成-store 判别（design 决策 1），注释按"try 之前是不是本资源自己的初始化"的模块问题表述；以 C1/C4.constructNamed 命中、1.2 负例保持、TWR 家族不回退验收。
- [ ] 2.2 `build.rs::all_reads_reach_presented_writes` 写值接受集扩展（design 决策 2）；构造写值呈现为声明初始化器；C1.five 的五连空 catch + 语句式 append 全部呈现，local 0 声明可放置；预算/取消原子回滚。

## 3. 翻转与回归

- [ ] 3.1 N1/P3StorePrefix 期望翻转为正例（恢复为普通 try/catch），同步 `p3_preceded_catches`/guard 单测与 `recover-preceded-statement-catches` 证据中的边界说明（记录结构论证）；catch-all 与其余 7 负边界逐项保持。
- [ ] 3.2 全量回归：C3 对照组逐字不变；`twrNamed` 输出与基线一致；FinallyOnce 全类、Test2 SHA、Tf1–Tf4、dt14 折叠、M1/M2 不回退。

## 4. 三方对照与验收

- [ ] 4.1 C1/C4.constructNamed 与翻转后的 N1：原 class/固定 JADX Java-input/Jarde 完整 Java 8 类重编，`java -Xverify:all` 正常与注入异常路径逐路径一致；记录输出 SHA。
- [ ] 4.2 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 同款 Clippy 新码零新增、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 4.3 root 独立复核判别边界、写值呈现与三方行为，更新 CF-15 清单与巡查账本；仅标记具名行-完成-store 家族。

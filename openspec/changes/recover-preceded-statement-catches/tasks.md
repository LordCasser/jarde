## 1. 基线与负例冻结

- [ ] 1.1 重放[固定证据](../../evidence/java-syntax-2026-09-30/finallyonce-main-catches/README.md)：核对 fixture SHA、M3/M5 主线整方法回退、N1 `jre_guard_handler`、N2b `jre_concat_split`@32 与原始 `FinallyOnce.class` 的 BCI 65 拒绝；记录可重放基线。
- [ ] 1.2 构造并冻结至少四个 verifier 有效近邻：具名 catch 前置语句换成 `putfield`/`getstatic` 消费语句等非 store 形态（应恢复）、store 前置降级（应拒绝）、split 链前存在更早同 owner `toString`（归属正确）、多资源 TWR 正例（应恢复）；各自 `java -Xverify:all` 通过并记录变更前后行为。

## 2. resources 门槛与 split 归属实现

- [ ] 2.1 在 `guard.rs::resources` 候选循环按 design 决策 1 增加"前置非 `Store` 则不当资源头"分支，注释与既有门槛族同一问题表述；以 1.1/1.2 命中与拒绝测试验收，TWR 家族（多资源、可空、`r = open()` 降级）现有测试不回退。
- [ ] 2.2 在 `concat.rs::verify` 按决策 2 实现值流归属：同 owner、`bci > head` 的 `toString` 候选，接收者经 append 返回值有界回溯（≤16 层）回到本链 allocation/dup 才算消费点；跨块消费点维持 `jre_concat_split`，否则不以此码拒绝。预算内完成，超限按不可达处理交回 walk。以 N2b、M3 第三条链、1.2 近邻验收归属与准入。

## 3. 家族与完整类交付

- [ ] 3.1 以新测试文件（或按仓库惯例扩展既有 p3 文件）覆盖：M5 家族正例（含 1.2 非 store 变体）、M3 全形态、N1/N2b 负例与预算/取消原子性；断言 M1/M2 输出逐字不变。
- [ ] 3.2 原始 `FinallyOnce.class` 全类 class-source 零 not-recovered；`main` 与固定 JADX Java-input 结构一致，`handled`/`escaping` 输出不变；全部物理 BCI 来源可查，失败/停止无半成品。

## 4. 三方对照与主线验收

- [ ] 4.1 fresh CLI 重建：原始 class、M5 家族、M3 的原/JADX/Jarde 完整 Java 8 源码重编，`java -Xverify:all` 下正常与各异常路径行为逐字一致；记录输出 SHA。
- [ ] 4.2 回归既有 finally/concat/TWR/monitor 测试与 Test3/4/5/11/16/17 已验收切片；`cargo test -p jarde-java --tests --locked`、workspace check、`cargo fmt --all -- --check`、CI 同款 Clippy、`openspec validate --strict`、diff check，清理专用 Cargo target。
- [ ] 4.3 root 独立复核门槛分支、值流归属、三方运行与来源，更新 CF-16 清单与账本；仅标记非 store 前置具名 catch 家族与 `FinallyOnce.main` 切片，不外推 store 前置家族或 DEX。

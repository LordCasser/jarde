## 1. 基线与负例

- [x] 1.1 重放固定 G1（SHA 核对）：确认 `max(xs)` 拒绝点与级联；javap 核对调用位；记录基线。
- [x] 1.2 构造并冻结至少四个 verifier 有效变体/负例：HashMap→Map、HashSet→Set、ArrayList→Collection 双跳、嵌套泛型集合传参（正变体）；用户类 `MyList implements List` → List（保持拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 闭集扩展

- [x] 2.1 `platform_reference_argument_widens` 集合闭集表 + walk（design 决策 1–2，逐对 javadoc 注释与 JDK 反射机械核对）；G1.use 完整恢复、整类重编运行一致（`zeta:a`）；`List→Iterable` 既有回答与全部既有转换测试不变。
- [x] 2.2 变体族逐项恢复；负例保持拒绝；预算/取消不变。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 throwable-widening、List→Iterable 既有测试）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 3.2 G1 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [x] 3.3 root 独立复核闭集逐对、呈现与三方行为，更新 EM 账本与巡查记录。（root 于合并主线 f5ed5ada 复核：G1.use 恢复 `max((java.util.List) local0)` 及结果局部、重编运行逐字一致（`zeta:a`/`6`）；40 对严表（含 Abstract* 骨架与 Stack/Properties）经 JDK 8 机械核对 A–E 全过、JDK 23 预期失败即 release 门禁依据；表外负例逐字节不变；全仓 2783/0、fmt/openspec 237/237。实施模型 qwen/deepseek-v4.1-flash（glm 双通道配额受限期间的替代，交付质量与先例持平）。）
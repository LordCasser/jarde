## 1. 声明与使用证明分离

- [x] 1.1 复核 [EM-01 单 child 基线](../../evidence/java-syntax-2026-09-27/em01-declarations/single-baseline/summary.json) 与现有静态成员家族路径，建立无构造站点的唯一双向 relation、完整 root/child、无字段和合法抽象方法准入；以 `SingleAbstract.A` 正例及第二 child、错 relation、额外字段/Signature 负例验证。
- [x] 1.2 让声明型静态成员无需伪造构造目标，仍保持原构造型路径和预算/取消原子性；以现有 `static-member-basic` 与低预算回归验证。

## 2. 根源码与来源

- [x] 2.1 在最终根源码中输出一次 `public static abstract class A`、`A()` 和无 Code 的 `test2();`，保留物理 child 报告及派生身份锚点；以类源码/报告定向断言和 Java 8 重编验证。
- [x] 2.2 为 [EM-01 replay.py](../../evidence/java-syntax-2026-09-27/em01-declarations/replay.py) 增加 fixed 断言，重放 `--fixture single` 三方完整源码重编与 `java -Xverify:all`，保持 `--fixture multi` 仍按独立多 child/泛型缺口拒绝；执行适用 Rust 回归、workspace check、格式与 `openspec validate assemble-proved-static-member-declaration-only --strict`，记录未覆盖形态。

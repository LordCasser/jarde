## 1. 冻结三方与拒绝证据

- [x] 1.1 重放有/无调试表的原/JADX/Jarde 完整类、方法类型参数/异常反射及强类型调用方；以 [replay.py](../../evidence/java-syntax-2026-09-24/body-method-local-generic-throws/replay.py) 退出 0、[analysis.md](../../evidence/java-syntax-2026-09-24/body-method-local-generic-throws/analysis.md) 的 SHA 与两种失败形态验证。
- [x] 1.2 冻结 verifier-valid 未绑定/擦除矛盾方法 Signature、非空正文及覆写/本类调用绑定负例；以严格 JVM 验证、物理回退、完整类 Java 8 重编和可重放脚本验证，并区分更早阶段的拒绝。根代理已独立复跑 [negative/replay.py](../../evidence/java-syntax-2026-09-24/body-method-local-generic-throws/negative/replay.py) 退出 0；[负例分析](../../evidence/java-syntax-2026-09-24/body-method-local-generic-throws/analysis-negative.md) 明确同类调用物理回退的完整类不可重编是已知限制，未把它计入可重编样本。

## 2. 同轮方法局部异常投影

- [x] 2.1 从无正文泛型方法路径抽出或复用结构化 `<X>`、返回、参数、`throws X` 头部拼写，保持既有 NoBody/静态参数返回门；以无正文和有正文泛型方法定向 Rust 回归验证输出及来源不变。
- [x] 2.2 对顶层非泛型 Object 直接子类的无参 `void` 方法复用现有 `EmptyVoid` 同轮候选及 reader 方法局部作用域/异常擦除证明，仅在单个合法局部变量与同名 `throws X`、无本类/继承绑定风险时原子发布完整泛型方法头；以两种 debug 正例完整类 Java 8 重编、反射、强类型调用方运行验证。
- [x] 2.3 覆盖伪造签名、非空正文、handler、Object 同名方法、本类调用、预算/取消和相邻类级 `throws E`、构造器与普通泛型方法；以定向 Rust 测试、负例 replay、`cargo fmt --all -- --check` 和 `git diff --check` 验证，记录首片拒绝范围。

## 3. 独立验收

- [x] 3.1 root 审读方法局部作用域、共享声明拼写、正文与调用绑定 proof、来源及原子发布；独立重放正反例，运行 reader/query、类源码/相邻泛型回归、适当 Clippy、格式和 `openspec validate recover-proved-body-method-local-generic-throws --strict`，记录结果并清理私有 Cargo target。结果见[独立验收](verification-root.md)。

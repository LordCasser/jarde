## 1. 冻结对照与拒绝边界

- [x] 1.1 重放 `-g`/`-g:none` 原 class、JADX、Jarde 的完整类、反射、强类型调用方与 SHA；以 [replay.py](../../evidence/java-syntax-2026-09-24/body-generic-throws/replay.py) 退出 0 和 [analysis.md](../../evidence/java-syntax-2026-09-24/body-generic-throws/analysis.md) 对照验证。
- [x] 1.2 冻结 verifier-valid 伪造 Signature、真实正文效果、处理器/继承或本类调用绑定负例；以 [negative/replay.py](../../evidence/java-syntax-2026-09-24/body-generic-throws/negative/replay.py) 独立退出 0、逐例 JVM 验证、局部物理回退、Java 8 重编和 [analysis-negative.md](../../evidence/java-syntax-2026-09-24/body-generic-throws/analysis-negative.md) 验证。继承例在更早的类作用域门拒绝，不宣称它命中了正文门。

## 2. 同轮空正文证明与声明投影

- [x] 2.1 扩展现有 class-source 方法候选以证明完整 AST、Code 和 SSA 精确为空 `void` 正文；以正例两种 debug 变体和正文效果负例的定向 Rust 测试验证，不改变普通方法报告。
- [x] 2.2 复用 reader 的异常签名/擦除及已发布类变量作用域，仅在顶层 Object 直接子类、无接口、无 `Object` 同名方法或本类同名调用、无参 `void` 与完整空正文证明时原子写出 `throws E`；以完整类 Java 8 重编、反射和强类型调用方执行验证。
- [x] 2.3 覆盖未绑定/擦除矛盾签名、处理器、继承/调用、来源标记、essential/all、预算/取消及相邻无正文/有正文泛型方法不变；运行定向 Rust 测试、格式与 `git diff --check`，记录仍拒绝的形状。

## 3. 独立验收

- [x] 3.1 root 审读候选 AST/Code/SSA 完整性、异常来源、类作用域和绑定门；独立重放正反例，运行 reader/query、类源码/相邻泛型回归、适当 Clippy、`cargo fmt --all -- --check` 与 `openspec validate recover-proved-body-generic-throws --strict`，记录于 [verification-root.md](verification-root.md) 并清理私有 Cargo target。

# DT-11 int 构造实参首切片验收

主线实现 `477a762f`，root 将外部 owner 的 synthetic class 标志也纳入保守拒绝后为 `5ed6b360`。同次 raw Code/BCI 证书现在消费 enum `<clinit>` 的常量前缀，按原求值顺序保留 literal、一次 `getstatic:I` 和 `getstatic:I + literal`；跨类字段使用选定环境的完整类与字段事实证明，整组失败时不发表部分常量。普通方法的物理 `<clinit>` fallback 与诊断仍可见，不靠其 AST 分类交错 `new; dup; getstatic; invokespecial`。

root 在 `5ed6b360` 上运行 [固定 replay](run_audit.py)，原始 class 与 JADX 的三个案例都通过 `javac --release 8` 和 `java -Xverify:all`；Jarde 的 `LiteralOnly` 与 `IntArgs` 也完整重编运行，分别输出 `OK LiteralOnly`、`OK IntArgs`。[summary.json](generated/summary.json) 记录逐组退出状态，修后 [IntArgs 源码](generated/jarde-IntArgs.java) 保留 `Ints.THREE` 与 `Ints.THREE + 1`。`StringVarargs` 仍按独立边界降级，javac 报 `enum constant expected`，没有运行阶段，不能算已恢复。

独立门禁：`cargo fmt --all -- --check`、`cargo test -p jarde --lib --locked`（101/101）、`cargo check --workspace --locked`、`cargo build -p jarde-cli --locked` 与 `openspec validate prove-enum-int-arguments --strict` 均通过。根目录 `cargo test --workspace` 的全包编译仍被既有匿名分配集成测试 API 解构漂移挡住，见[独立债务](../../jadx-feature-inventory-2026-09-27/anonymous-allocation-test-api-drift.md)，不把它归因于 DT-11。临时 Cargo target 已清理。

本项只关闭 DT-11 的有界 int 实参差距；String varargs 和其他构造参数表达式继续在清单中，不把单例成功外推为整个 DT-11 追平。

## 1. 在现有值证明内准入整数尾返回

- [ ] 1.1 对 `Region::ShortCircuitValue` 的直接 `int ireturn` 分支证明两个 int 常量 producer、同栈深、唯一 phi/消费者和完整测试边；以 CF-04 BCI 1/5/12→15/19→20 的正例及额外使用/非 literal/异常边负例验证。
- [ ] 1.2 复用原条件树 builder 输出整数 `?:`，跳过只对 1/0 布尔叶合法的简化；保留 AST/SourceMap 物理来源、预算与停止的原子性。

## 2. 固定三方与回归

- [ ] 2.1 更新 [CF-04 脚本](../../evidence/java-syntax-2026-09-27/cf04-ternary/replay.py) 的修后门槛：原 class、固定 JADX、Jarde 的**完整 `TernaryCases` Java 8 源码**均重编、`java -Xverify:all` 运行 13 行一致；`TernaryBasic` 11 行继续一致。
- [ ] 2.2 运行相关短路/条件值、布尔 1/0、构造器及预算/来源目标测试，`cargo fmt --check`、`cargo check --workspace` 和 `openspec validate recover-proved-nested-int-conditional-return --strict`；记录未覆盖的任意表达式 producer 边界并清理 Cargo 产物。

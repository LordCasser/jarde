## 1. 冻结输入与拒绝边界

- [ ] 1.1 将 `anonymous-interface-basic` 的原/JADX/Jarde 三方源码、class 哈希及 Java 8 重编运行脚本作为基线；执行 `python3 openspec/evidence/java-syntax-2026-09-27/anonymous-interface-basic/replay.py`，确认三方均输出 `7`，且旧 Jarde 仍写 `$1` 构造。
- [ ] 1.2 利用已冻结双分配、跨类身份用例，并构造捕获/有初始化效果/正文不完整的最小负例；用集成测试证明它们在内联前均保留物理构造，不以方法数代替 BCI 数，也不以 `contains_statements` 代替完整正文。

## 2. 类级证明

- [ ] 2.1 消费 typed 匿名声明和选定子类事实，证明准确 `EnclosingMethod`、`Object` 父类、单接口、零字段及唯一平凡 `()V` 构造器；用正例与捕获/字段/构造效果负例测试证明候选选择。
- [ ] 2.2 对调用者全部有 `Code` 方法的同次 `AnonymousAllocationScan` 及所选输入范围的完整 owner XRef 做唯一性证明；用单站点正例、同方法双站点、跨类使用、未决/不完整/预算停止测试核对严格拒绝。
- [ ] 2.3 仅当子类每个需呈现的方法体质量、覆盖、原始 Code 及 Java 8 声明均完整时产出候选；用一个缺正文和一个混合质量负例证明不能输出空体或半体。

## 3. 原子匿名表达式投影

- [ ] 3.1 从同次方法 AST 的准确直接返回 `New` 节点与子类方法 AST 构造匿名接口表达式，并由发射器写出完整源码和物理来源；集成测试断言 `new I() {`、`value()`、`return 7`、使用点/子类方法 BCI，及非直接返回保守不变。
- [ ] 3.2 整类源码只在所有证明和发射完成后发布；预算/取消、双分配与跨类负例保持原文本。执行定向集成测试，并对入口类与接口依赖执行 `javac --release 8 -g:none`、`java -Xverify:all`，比较原/JADX/Jarde 的输出与效果。

## 4. 架构师独立验收

- [ ] 4.1 Root 独立复跑冻结脚本、定向与受影响回归、`cargo fmt --all -- --check`、`openspec validate inline-proved-anonymous-interface --strict`，复核源码及物理报告的身份、预算、来源约束；只有这些通过后更新 DT-05 盘点状态并提交推送。

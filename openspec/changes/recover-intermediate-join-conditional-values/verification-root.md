# Root 独立验收（2026-09-26）

Region 仅在内层 `If` 的未认领 join 同路径、同 scope、正常边和预期前驱/后继完整闭合时，把后续 Straight 纳入外臂；失败回滚 visited 并引用整段候选。值证明分别绑定内层 Phi、桥接 `Push(int) → iadd → Transfer`、外层 Phi 的两个真实前驱 exit、唯一消费者和 `int` 类型。Builder 先局部构造子条件，在桥接渲染期间临时可见，完成外条件后才发布根 Phi 与折叠分支；拒绝时引用每条候选的物理 BCI。

Root 用重建 CLI 对冻结 Java 8 class 直接生成完整类，未编辑源码。`choose(I)I` 呈现为 `arg0 > 0 ? (arg0 > 1 ? f1() : f2()) + 3 : f3()`，默认与 `all` 正文相同。生成类经 `javac --release 8` 与 `java -Xverify:all`，六条正常/异常输出均与原版和 JADX 逐字一致。报告中内 join 18、桥接 BCI 19/20、外 join 26 有物理来源；外层条件无 fallback。

`cargo test -q -p jarde-java` 全通过（206 个库测试及全部集成测试），其中双 join 定向测试 4/4；`cargo fmt --all -- --check`、`git diff --check` 和 `openspec validate recover-intermediate-join-conditional-values --strict` 通过。verifier 有效的独立调用、抛错除法、外部入口与 handler 负例均保留完整引用，预算及取消停止不发布半成品。严格 Clippy 因 24 条本轮之外的既有告警失败；未在本轮处理。验收后清理隔离 Cargo target。

任务 1.2、2.2、3.1 保持未完成：额外 SSA use、Call 边和 owner 重叠尚缺各自完整的 proof-unit 负例矩阵；现有正例和负例不能外推为任意桥接表达式均可恢复。

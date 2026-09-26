## 1. 固定两层汇合的事实

- [x] 1.1 核对冻结 Java 8 class、原/JADX/Jarde 完整类、六条正常/异常 runner 与哈希，并以 `region-trace.txt` 验证外 join 26、内 join 18 和 BCI 18 未被旧 Region 认领；运行证据目录的 `shasum -a 256 -c SHA256SUMS.txt`。
- [x] 1.2 为额外消费者、桥接独立效果、外部入口、异常/Call 边与 owner 重叠准备 verifier 有效的 class 或直接 CFG/SSA 负例；逐项验证真实边、use 与预期拒绝位置，不用不可验证的字节码替代语义反例。

## 2. 闭合 Region 臂内续走

- [x] 2.1 让外臂在内层 `If` 返回未认领的前向 join 后沿同一 `Frame` 续走到外臂边界；以冻结样本断言 BCI 18–20 进入真臂且每个可达块只有一个 owner，旧两臂与同 join 树测试通过。
- [x] 2.2 对续走的正常边、前驱、路径、scope、边界与 visited 差集做原子闭包核对；以 1.2 的外部入口、重叠和异常/Call 边反例及预算/取消测试验证失败时整段引用或既有停止，不留下 BCI 18 式遗漏。

## 3. 跨 join 值证明与呈现

- [x] 3.1 复用内层两臂证明，专门核对外层非直线臂与桥接指令的两组 Phi 输入、唯一消费者、真实有序依赖和可呈现类型；以冻结样本及额外 use/独立效果/未知运算负例验证只接受完整组合。
- [x] 3.2 将子条件、桥值与外条件表达式局部构建并一次发布根 Phi、分支折叠和来源；以直接断言验证无空臂、无重复调用、BCI 18/19/20/26 只归属一次，失败时完整引用且不留下局部计划。

## 4. 整类对照与独立验收

- [x] 4.1 用重建 CLI 不编辑生成文本地输出完整类，以 `javac --release 8` 与 `java -Xverify:all` 比对原/JADX/Jarde 六条运行结果；核对默认/all 正文、物理 BCI/成员来源和 Java 表达式分组。
- [x] 4.2 Root 独立审查 Region 闭包、两组 Phi 身份、求值顺序、唯一所有权及拒绝路径；运行相关 Rust/Java 回归、`cargo fmt --all -- --check`、`openspec validate recover-intermediate-join-conditional-values --strict`，并清理隔离 Cargo target。

## 1. 冻结基线与边界

- [x] 1.1 冻结 `NestedArrayInitializer.class` 的 Java 8 原源码、JADX 1.5.6、当前 Jarde 完整类及 `javap`；验证原/JADX `-Xverify:all` 两行一致、Jarde 两方法缺 return，记录 SHA 和编译结果于[分析](../../evidence/java-syntax-2026-09-25/nested-array-initializer/analysis.md)。
- [x] 1.2 冻结 `JadxOrderingControl` 的物理写入顺序，验证原/Jarde 的 `12` 与 JADX 重编的 `21`，并定位 JADX `ReplaceNewArray` 的按索引排序路径于同一分析。
- [x] 1.3 将正例及有副作用乱序负例登记为永久 Java 8 fixture；至少增加一个 verifier-valid 子数组额外 use 或异常边拒绝控制，验证原 class 可执行、Jarde 不发布部分嵌套表达式。

## 2. 扩展现有数组初始化证明与呈现

- [x] 2.1 在现有 `ArrayInitializers` 中先证明内层再组合父层，只接受同块、按物理 `0..N-1` 次序、唯一 `aastore` 值消费、精确数组组件及闭合 SSA/效果区间；以正例两个方法结构恢复和 1.3 拒绝控制验证。
- [x] 2.2 复用现有 `NewArray` AST、类型与发射使带初始化器的已证明总 rank 可呈现 `new T[][]{…}`；用 Java 8 编译 `int[][]`、`String[][]` 整类并检查元素类型与现有一维、部分维度分配回归。
- [x] 2.3 父子来源互不重复认领，元素各求值一次，失败/预算/取消原子回退；验证正例所有真实 BCI source map 非空、异常/顺序 trace 与原 class 相同，并以低预算及预取消定向测试检查无部分提交。

## 3. 独立对照验收

- [x] 3.1 root 独立重建 CLI，对原/JADX/Jarde 三套完整源码作 Java 8 编译与 `java -Xverify:all` 运行比较；正例值、trace 及异常一致，乱序反例保持 `12` 或完整拒绝，并记录 CLI/class SHA 与报告。三方正例 class 逐字节同原，乱序原/Jarde 为 `12`、JADX 为 `21`，见 [root 验收](verification-root.md)。
- [x] 3.2 root 审读分配身份、物理顺序、父子来源及拒绝边界；运行新测试和既有数组初始化/部分维度/数组写入/短路数组回归，检查 `cargo fmt --all -- --check`、适用 Clippy、`git diff --check` 与 OpenSpec strict，记录剩余架构债与 Cargo target 清理。15/15 定向测试、普通 Clippy、格式、diff 与 strict 通过；既有 17 条严格 Clippy 告警另案，root target 待后续验收完成清理。

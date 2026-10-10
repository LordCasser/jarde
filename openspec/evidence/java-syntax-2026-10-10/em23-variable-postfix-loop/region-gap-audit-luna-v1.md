# EM-23 `ArmsDoNotMeet` 局部审计

JADX 的 `TestVariablesDefinitions2.TestCls.test(List<String>)` 是 `i=0`、`list != null`、增强 `for`、内层 `str.isEmpty()` 后 `i++`、最后 `return i`；断言这些形状且不产生 `i2`（[测试源码](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/variables/TestVariablesDefinitions2.java:14)）。Jarde 冻结基线的对应报告是 `explanation_only / jre_region_arms_do_not_meet`，定位 branch block 0。原字节码 BCI：null 分支 3→45；循环头 13/19；内层条件 36；回边 42；返回 45/46。报告没有给出触发该 reason 的内部 guard。

`region.rs` 入口在 1366–1380 行反复调用 `Walker::region_at`；`unclosed_tail_at` 会将全方法映射为 `ArmsDoNotMeet`。最贴近 null guard 的单臂路径在 3443–3456 行：arm 的 `next` 必须是 join，或被 `continue_early_return_arm` 证明；否则设置该标记。相同 reason 还可能来自两 successors 都被读作 join（3494–3506）、双臂 loop/inner-join continuation 失败（3783–3796）、或两臂 join reachability 检查失败（3861–3905）。最终 explanation 不能区分这些位置。

现有架构理论上有处理循环的路径：`loop_region`（9098–9177）先取 natural loop、拒绝 irreducible，再尝试 latch/header-tested 形式；header-tested 路径经 `header_test_chain`、`proved_header_test`、`loop_body_sequence` 检查条件分支、出口及 body 覆盖。其 test 允许被分支消费的 `Invoke` 留在表达式（`test_expression_instruction`，7151–7218），所以不能仅凭 `hasNext()` 是调用就断言循环被拒。内层 `isEmpty()` 仍需通过 loop frame、ownership 和 join 检查。当前报告不足以证明哪一项成立或失败。

JADX 对照：`RegionMaker.traverse` 将 loop-start 交给 `LoopRegionMaker`、IF 交给 `IfRegionMaker`（`RegionMaker.java` 81–110）；`LoopRegionMaker` 选 exit 并构造 body（61–181、187–245）；后续 `LoopRegionVisitor.checkIterableForEach` 核对 `iterator/hasNext/next` 使用与变量范围后注入 foreach（246–339）。这是测试形状的识别路径，不代表两边 CFG guard 等价。

**最小判别探针：** 同一 frozen class 单次恢复，在每个 `ArmsDoNotMeet` 返回点临时记录 site 标签、branch BCI、join、succ、frame boundary/loop target、arm `next` 与末尾 Region；branch 3 另记 block 13 的 loop 结果/出口是否为 45。已有 `JRE_JOIN_PROBE`（3394–3407）只记 join 候选；`JRE_PREFIX_PROBE` 位于 `unclosed_tail_at` 早退之后（1382 起），该早退不会打印。无需扩展 fixture。

Atlas 已 `project(open)` jarde，并 scoped 查询 `crates/jarde-java/src/region.rs`：`ArmsDoNotMeet` 搜索完整，`Walker::region_at` 返回源码；没有全仓索引。其余判断按上述源码行只读核实。

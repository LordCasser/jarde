# EM-23 `ArmsDoNotMeet` 增补：null guard 到循环头

冻结诊断 `results/join-diagnostic-root-v1/` 的原始 stderr 只有两条：

```text
P3VISITED blocks=[0] visited=[0]
P3JOIN branch=0 ipdom=Some(45) then=Some(6) else=Some(45) ft=false fe=true frame_boundary=None
```

`P3JOIN` 足以确认 branch 0 的立即 post-dominator 是 45，另一条臂从 BCI 6 前进，故一臂读取选 45。它没有记录递归臂的 `next`/Region。另一个重要边界是 CLI 对同类的 `<init>` 和 `countEmpty` 分别调用恢复，而 probe 没有方法身份；按方法顺序，第一条 `P3VISITED` 可来自单块构造器。因此它不能证明 `countEmpty` 没走 `unclosed_tail_at` 早退，也不能拿来排除一臂的 `next` 不匹配。

从固定 CFG 与当前 walker 控制流可构造一个具体、但尚未由日志证实的早退链：一臂由 `region_at(walk=6, frame.arm(boundary=45,...))` 进入。普通 setup 块被加入 `prefix` 后到 loop header 13；`region_at_inner` 的 loop-header 分支（`region.rs:2727–2741`）只有 `prefix.is_empty()` 才转调 `loop_region`，否则立即返回 `Straight(prefix), Some(13)`。若本例路径正是这样，循环体以及 BCI 36 的内层条件尚未被遍历。外层一臂接着比较 `arm_next=13` 与 join 45，调用 `continue_early_return_arm`；该 helper 首先要求 arm 结果恰为单个 `Region::If`（`3971–3982`），而该候选结果是 `Straight`，会直接拒绝。即使结果形状通过，helper 还要求布尔返回（`4001–4005`），但 `countEmpty` 返回 `int`。失败会在 branch 0 设置 `unclosed_tail_at` 并返回 `ArmsDoNotMeet`（`3447–3456`），与报告代码和 block 0 定位相容。

这条链说明了一个可能的边界：arm walker 在带有非空前缀时把循环头作为 continuation 交还调用方，而外层 null-guard 只为布尔早返回接受跨越其 join 的单个内层 `If`。它不证明运行时正好经过了这一分支；`P3JOIN` 没覆盖后续递归，且日志未区分两个方法。若继续定位，只需为 branch 0 的递归结果记录 `arm_next`、最后 Region 形状，并给 `region_at` 的 loop-header return/`loop_region` 入口各加临时 site 标签；无需新 fixture。当前没有足够证据将此判断写成产品缺陷或实施边界。

此前 Atlas 已先 `project(open)` jarde，并只对 `region.rs` 做 scoped 查询；本增补只读该文件及已冻结诊断 raw，没有运行工具链、Git 或修改产品。

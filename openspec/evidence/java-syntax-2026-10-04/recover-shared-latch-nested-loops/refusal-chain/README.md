# CF-11 拒绝链转录（为何未达成 → 为何这样修）

全部由**临时插桩**（`CF11DBG`/`JRE_JOIN_PROBE` 环境变量门控的 `eprintln!`，验后已全部移除，工作树
最终 diff 不含任何插桩）在改动前基线实测。`JRE_JOIN_PROBE` 是仓库既有探针（region.rs 分支 join
选举处），非本片新增。

## 复现方法

对基线 `region.rs` 施加下方插桩补丁 → `cargo build --locked -p jarde-cli` →

```sh
cd openspec/evidence/java-syntax-2026-10-04/shared-latch-patrol/fixture
CF11DBG=1 JRE_JOIN_PROBE=1 <target>/debug/jarde-cli class-source \
  --policy single-class --input S5.class --class S5 2>&1 | grep -E 'CF11DBG|P3JOIN'
```

## 插桩点（8 处，全部验后移除）

1. `loop_region` 入口：header/latches/blocks 转录；
2. `loop_region` 三分派点（latch_test_chain HIT / body_branch / effectful_dual_exit 后）；
3. `region_at_inner` 嵌套循环头分支（2544 行处）`prefix_empty`；
4. `loop_body_sequence` 每步；
5. `header_tested_loop` 单测路径出口/门信息；
6. `header_tested_loop` covers 检查 missing 列表；
7. `P3JOIN`（既有）join 选举；
8. `latches.len() != 1` 的门控放宽实验（`CF11EXP`）。

## 实测转录（S5.outerContinueInner，基线代码）

```
CF11DBG nested-header bci=4 prefix_empty=false latch_edge_own=false targets=[]
CF11DBG nested-header bci=4 prefix_empty=true  latch_edge_own=false targets=[]
CF11DBG loop_region enter header_bci=4 latches=[35] blocks=[4, 9, 15, 18, 35, 20, 25]
       nodes=[(0,0),(1,4),(2,9),(3,41),(4,15),(5,18),(6,35),(7,20),(8,25)]
P3JOIN branch=9 ipdom=Some(35) then=Some(15) else=Some(18) ft=false fe=false frame_boundary=Some(4)
CF11DBG nested-header bci=20 prefix_empty=false latch_edge_own=false targets=[1]
CF11DBG seq step block_bci=9 node=2
CF11DBG seq step block_bci=35 node=6          ← 体走自 If 直接续行到 latch
CF11DBG header=4 HTL covers missing=[7, 8]    ← 节点 7,8 = BCI 20,25（内层循环两块）
```

产物诊断（与巡查记录逐字一致）：

```
jre_region_loop_shape       the loop whose header is the block at BCI 4 has a test, an exit or a
                            latch this subset does not prove
jre_region_uncovered_blocks 3 live block(s) are reachable only through edges the normal-flow view
                            leaves out: [41, 20, 25]
```

链路解读：块 9 的 `ifne`（源码 `if (i % 2 == 0) continue;`）两臂因内层退出边（20→35）与外层
continue 边（15→35）**共享 latch 35** 而在 35 汇合 → 后支配者 ipdom=35 当选 join；else 臂（60 一侧，
即 BCI 18→嵌套循环头 20）以 `next=Some(20)` 结束但分支发布 next=35 → 嵌套循环头被越过且不再被重访 →
内层 `loop_region` **从未被调用**（无 header=20 的 enter 行）→ covers 缺 [20,25] → LoopShape@4，
其 fallback quote `[4,9,15,18,35]` 与未覆盖 `[41,20,25]` 与巡查 README 逐字一致。

## 决定性实验（证伪 root 钉的"现约束"落点）

仅放宽 `latch_tested_loop` 的 `latches.len() != 1`（`CF11EXP` 环境变量门控，其余不动）：

```
CF11EXP=1 渲染 S5 → "not recovered" 计数 2（outerContinueInner、labeledOuter）
基线     渲染 S5 → "not recovered" 计数 2（同两形）
```

放宽前后完全一致 → **该判据在 S5 流程从未被行使**（外层 while 形走 `header_tested_loop`；内层循环因
上述链路从未进入 `loop_region`；两循环各自的自然 loop latch 数均为 1）。root 的六个锚点/行号记载本身
属实，错在"该判据即本片要放宽的现约束"这一前提——与 getClass 片的 spec 前提被证伪同性质。

## 对照：修复后的行使轨迹（同插桩点）

```
CF11DBG slj: branch=9 elected=Some(35) latch=6(=35) continue_target=6 ... then_next=None else_next=Some(20)
CF11DBG slj: bridge entry=Some(15) ok=true            ← loop_continue_bridge 命中（continue 边）
CF11DBG slj: nested=20 exits=[35]                     ← 嵌套退出边集合 = {latch}
CF11DBG nested-header bci=20 prefix_empty=true        ← 体走在重选举 join（18）处续行
CF11DBG loop_region enter header=20 ...               ← 内层循环进入构建
CF11DBG header=20 HTL covers missing=[]
CF11DBG header=4  HTL covers missing=[]               ← 外层 covers 通过
```

（S3 家族的 header-latched 形轨迹：`elected=Some(20)=dest(header)`、bridge=57、nested exits=[153]、
landing 经一次 transfer 107→153、accounted={57,153}==held。）

# `one(ZZ)V` 的 Region/SSA 定位（2026-09-25）

本记录只定位 `MixedShortCircuitField.class`（SHA-256 `e69f52f563c4c7e6b8be1aeb7be946f4af09649c48280863f56ed0fe42033fd1`）当前的 `jre_region_ownership_overlap`，不修改 Region owner validator 或恢复实现。诊断运行使用私有 `/tmp/jarde-instance-diagnostic-target`。临时在 `short_circuit_value` 无 consumer anchor 的返回处打印 Canonical/SSA，在 `overlapping_owner` 命中处打印尚未改写的 Region 树；打印代码运行后已撤销。

## 物理图与值

`javap -c -p -v` 与诊断输出一致：Canonical 块起点为 `0, 8, 14, 20, 24, 25`，路径均为 `[]`；块 0 覆盖 BCI `0,1,4,5`，块 8 覆盖 `8,11`，块 14 覆盖 `14,17`，块 20 覆盖 `20,21`，块 24 覆盖 `24`，块 25 覆盖 `25,28`。Canonical 的完整边列表如下，**全为 Normal**：

```text
0→8   0→14
8→14  8→20
14→20 14→24
20→25 24→25
```

因此 BCI 20 的两个入边正好来自图内测试 BCI 11 与 17 所在块；BCI 25 的两个入边正好来自两个 1/0 producer。此类没有异常表；诊断列表中也没有 Exception/Call/Return 边或外部前驱。BCI 1 的 `target:(Z)Box` 调用确实有效果，但它位于首个测试 BCI 5 以前的 prefix，产生接收者 `ValueId(13)`，后续沿栈深 0 原样传递；它不是测试/producer 闭合段中的独立效果。

SSA 在 BCI 20 写 `Stack(1)=ValueId(16)`（`iconst_1`），BCI 24 写 `Stack(1)=ValueId(18)`（`iconst_0`）。BCI 25 的 `Stack(1)` Phi 是 `ValueId(11)`，输入恰为 `[16,18]`；同块 `Stack(0)=ValueId(13)` 是 BCI 1 的接收者。`putfield` BCI 25 的读集为 `[(Stack(1), ValueId(11)), (Stack(0), ValueId(13))]`，写集为空，后随 BCI 28 `return`。这个事实同时说明：短路值只有一个直接 field-value consumer，但实例字段指令有两个栈操作数；现有仅允许单栈读的静态字段证明不能直接复用。

## 首个 `None` 与重复 owner

`Walker::short_circuit_value` 从外层 BCI 5 开始时已经收集到测试 `5,11,17`、producer 块 `20,24`、consumer 块 `25`，并通过其前置真实边/精确前驱检查。临时诊断只在其 consumer-anchor 查找失败的 `else` 分支打印，实际输出是：

```text
INSTANCE_NO_CONSUMER_ANCHOR outer=5 consumer=25 opcodes=[(25, 181), (28, 177)]
```

十进制 opcode `181` 是 `0xb5 putfield`。该 anchor 目前只接受静态字段写、`ireturn`、调用或局部 Store；故最先失败的是 `region.rs` 的 consumer-anchor 枚举，不是外部真实边、异常边、效果或预算门。由于候选返回 `None`，`region_at_inner` 进入通用 `if` 构造，诊断得到 owner validator 执行前的具体树：

```text
If(branch 0 / BCI 5, join 14)
  then_arm: If(branch 8 / BCI 11, join 14)
    else_arm: Straight [20, 25]          ← 第一 owner
  else_arm: Straight []
If(branch 14 / BCI 17, join 25)
  then_arm: Fallback Loop [20]          ← 第二 owner，先命中 BCI 20
  else_arm: Straight [24]
Fallback Loop [25]                     ← BCI 25 也重复
```

通用路径先把 BCI 14 当作前段 `if` 的 join，并沿 BCI 8→20→25 把后段共享 producer/consumer 纳入前段 arm；随后在 BCI 14 的另一个分支沿 14→20 重访，`visited` 把 BCI 20 判作 Loop fallback。`overlapping_owner` 对完整树按物理块顺序检查，首先命中 BCI 20，于是**正确地**整体 quote，报告 `jre_region_ownership_overlap`。不能删除或放宽该 validator：在当前通用树下两次 owner 是真实的表述冲突；要避免冲突，应先让有完整证据的专用短路候选取得一次所有权。

## 相邻边界与最小落点

既有 `MixedBooleanField.andOr` 的同类测试/1/0/唯一 Phi 图在共享 consumer 处使用 `putstatic result:Z`，因此 `short_circuit_value` 能取得 anchor，先认领整段再交给 `prove_short_circuit_value`，字段 16 路径验收通过。实例样本与它不同的是 prefix 中的接收者计算及 `putfield` 的双栈读取；图的正常前驱闭合本身没有新增外部入口。`ExceptionShortCircuit.assign` 虽使用 `putstatic Z`，却有保护 `[0,18)` 到 handler BCI 21 的真实异常边；既有测试要求它整体拒绝并报告 `jre_region_exception_edge`。实例字段扩展不得借由放宽边或 owner 检查来吞掉该负例。

最小生产落点首先是 `region.rs::short_circuit_value` 的 field-write anchor：在该具体闭合图中识别 `putfield`，保持原有精确前驱、无额外边、无回边及一次 owner 检查。随后 `build.rs::prove_short_circuit_value` 需要区分静态单栈读与实例双栈读，核对 `Stack(1)` Phi 和 `Stack(0)` 接收者的精确 SSA 身份、field plan 的 `Z` 描述符与接收者/value 形状，并证明接收者不会被第二次输出。`build_short_circuit_statement` 的静态字段分支目前要求 `evidence.is_static`、`shape.receiver.is_none()`，实例分支须沿现有 `FieldAssign { receiver, value }` 写接收者再写惰性 RHS，保留 BCI 1 调用只执行一次及空接收者在 RHS 后才于 BCI 25 抛异常的 16 路径合同。这些是后续实施位置，不是本次已通过的证明。

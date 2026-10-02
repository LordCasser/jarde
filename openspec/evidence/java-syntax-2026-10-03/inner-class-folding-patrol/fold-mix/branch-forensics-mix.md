# 混合族静态子集折叠取证（task 1.1/1.2，change `recover-inner-class-static-mixed-folding`）

基线：worktree 主线 `438e1a2d`（实现前重放，debug 构建）。N1 fixture SHA 与
[`../results/fixture-sha256.txt`](../results/fixture-sha256.txt) 逐项核对一致；原 jar
`java -Xverify:all` 运行 `10`/`7`/`13`（`../fixture/orig.out`）。

## 1. 分支取证（`src/member_inner.rs::scan_family_root`，主线 L1248-1401）

混合族行序（N1 的 InnerClasses）：

```
Inner = class N1$Inner of class N1   (flags 无 static —— 非静态行)
static Stat = class N1$Stat of class N1 (0x0008)
```

逐行循环里静态行**从不进入** `candidate`：`if row.access_flags & 0x0008 != 0 { static_members.push(next); continue; }`
（L1348-1351）。静态子集被挡出 `StaticMembers` 的先行条件在**循环之后**，共两条：

1. **≥2 静态 + 非静态候选**：`if candidate.is_some() && static_members.len() >= 2` → 整个 scan
   `Refused("multiple direct static member rows are outside the one-child family subset")`（L1387-1391）。
   实测 MV3（StatA+StatB+Inner）正是此形态。
2. **1 静态 + 非静态候选**：`if let Some(selected) = candidate { return Ok(FamilyRootScan::Candidate(selected)) }`
   （L1393-1394）提前返回——静态行被**静默丢弃**（不进任何通道）。实测 N1/MV2 正是此形态。

另有相邻挡板（本片不动其纯静态侧）：`static_members.len() == 1 && abstract && direct_rows != 1`
→ `Refused("declaration-only static abstract member requires one direct child row")`——该门只对"单静态
经 Candidate 选择"的形态有意义；混合族里静态行走折叠通道后不再经过它（`candidate.is_none()` 收窄）。

非静态候选的消费路径（保持不动）：`Candidate` → facade `prepare_class_source_member_family`
（capture/calls 证明）→ 投影。N1 实测：capture
`Refused("constructor capture prologue or exception range is outside the proved shape")`、calls refused、
投影 `Refused("capture proof is incomplete")`——`member_family.state="prepared"`、无嵌套声明、
根文本对 Inner/Stat 均保持池拼写（`N1$Inner`/`N1$Stat`）。

并存路径结论：折叠文本对**非折叠成员**（Stat.use 里 `outer.new Inner(9)` → 恢复文本
`new N1$Inner(arg1, 9)`）保持池拼写——折叠目标列表只含静态子集，重写扫描只对目标 token 生效，
与分离平铺同口径；锚定沿用上片"全部覆盖 source-map 段 + 指令 CP（Class/FieldRef/MethodRef owner）
或 catch handler"泛化，无法锚定即拒绝折叠该子（保守）。

## 2. 变体冻结（task 1.2，`fixture-variants/`，`javac --release 8 -g:none`）

| 变体 | 形态 | 原类 `-Xverify:all` 输出 | 实现前 member_family |
| --- | --- | --- | --- |
| MV3 | 三子混合：StatA + StatB(extends StatA) 两静态 + Inner(base 捕获) 一非静态 | `6`/`6` | refused（multi-static 挡板） |
| MV2 | 单静态 Stat + 单非静态 Inner（`new MV2().new Inner(4)` 限定构造） | `11`/`8` | prepared（Inner），投影 refused——Stat 静默丢弃 |
| MV1 | 纯非静态族（负例：不折叠） | `9` | prepared（Inner），calls refused |
| N1 | 巡查固定 fixture（静态 Stat + 非静态 Inner + 限定 new ×2 + access$000 桥） | `10`/`7`/`13` | prepared（Inner），投影 refused——Stat 静默丢弃 |

class SHA 见 [`results/fixture-variants-sha256-mix.txt`](results/fixture-variants-sha256-mix.txt)；
实现前逐类文本见 [`before/*-mix-before.txt`](before/)。

## 3. 提取点（design 决策 1 的落点）

`scan_family_root` 循环内静态行收集**已经**与非静态候选解耦（互不 push）；解耦点在返回序：
新增 `StaticMembersWithInstance { statics, candidate }` 携带两者，facade 在消费缝上分解为
`Candidate(candidate)`（窄通道照旧）+ `static_fold_rows`（上片折叠通道照旧）——两条通道互不阻塞，
`≥2 静态` 拒绝门随之删除（该形态现入折叠），单静态挡板的纯静态侧保留。

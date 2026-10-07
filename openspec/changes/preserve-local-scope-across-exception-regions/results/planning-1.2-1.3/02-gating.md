# 1.2/1.3 切片：gating 实验（先量后做）

命令：`sh openspec/changes/preserve-local-scope-across-exception-regions/results/planning-1.2-1.3/02-gating.sh both`
（自测：每个渲染先断言 `// jarde: presentation of` 自头部；脚本每次应用补丁后重建 CLI、渲染、再 `git checkout` 还原；
最终状态经 `git status` 复核只有本切片的新增文件）。

两份补丁（`patches/`）都是**只读测量**，不是提交内容：

1. `01-classification-only.patch`：跳过 `has_unpresented_access`（`build.rs:882`）——规划不再对
   “使用集合含引注 fallback”的局部回答 `Incomplete`。
2. `02-classification-plus-declaration-region.patch`：在 1 之上再关掉 `declaration_region` 的 fallback
   否决（`build.rs:3368`）——规划里再没有任何基于 fallback 的拒绝。

`gating/*/` 只保留每个输入的**渲染正文**（`--format text` 的 stdout）；CLI 的 stderr bookkeeping
平面（usage/source_map 等）不入库，可用同一命令重放，逐局部诊断与 region 树见
`baseline/region-trees.txt`。

## 结果（渲染与 `baseline/` 逐字节比较）

| 渲染 | baseline（clean HEAD） | a：仅分类改动 | b：分类 + `declaration_region` |
| --- | --- | --- | --- |
| `LK`（锚 12） | 2 句 region 拒绝（BCI 7 handler shape、BCI 32 finally-copy merge） | **逐字节相同** | **逐字节相同** |
| `IO`（锚 13） | 2 × `local 1 crosses a quoted fallback region …` | 变句：2 × `local 1 spans accesses with no complete recovered lexical owner` | 半正文 + region 拒绝（`block at BCI 17/27 leaves through exception handler 0`），0 个整方法拒绝 |
| `ScopePlanCrossing`（本切片 fixture：同一批形状） | 2 × `local 1 crosses a quoted fallback region …` | 同上变句 | 同上：半正文 + region 拒绝 |
| `ScopePlan`（本切片正例控制） | 全成员恢复 | 相同 | 相同 |
| `ExceptionScope`（1.1 控制） | 全成员恢复 | 相同 | 相同 |

## 结论

* **分类改动单独翻转的锚：没有。** 锚 12 三个方法逐字节不变（其拒绝全在 region/guard 层，且规划对
  `take()` 的两个局部给的是 `Local`、对 `tryLockQuick()` 无话可说）。
* 锚 13 只把拒绝句**推后一层**（`crosses a quoted fallback` → `no complete recovered lexical owner` →
  region 层），正文始终不完整：因为 region 树里的 `Fallback(exception_edge)` 与
  `Fallback(uncovered)` 在规划运行前就存在，且 `finally_body_supported`（`region.rs:2400`）不允许
  带循环的 finally 体。
* **负向控制没有被放宽**：`ScopePlan`、`ExceptionScope` 在两种补丁下逐字节不变；补丁 b 的半正文里
  出现“只声明不赋值”的局部（`java.io.BufferedReader local1;`），正是 soundness 守卫要防的
  “compilable-and-different”方向——这解释了为什么 fallback 阻断不能简单删除。

因此 1.2/1.3 的“三分类 + 覆盖校验”本身可以照常落地与验收（见 `04-1.2-1.3-tests.md`），但锚 12/13 的
恢复不由声明规划决定，见 `03-boundary.md` 的 re-slice 说明。

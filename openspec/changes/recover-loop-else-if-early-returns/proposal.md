# 循环内 else-if 阶梯早退恢复（recover-loop-else-if-early-returns）

## Why

[binary-search 巡查 + root 2026-10-06 归因修正](../../evidence/java-syntax-2026-10-05/binary-search-twopointer-patrol/README.md)：`while 循环 + else-if 阶梯（≥2 条件臂）+ 阶梯内早退 return` 的方法形状在**一切类上下文**中被 `jre_region_ownership_overlap` 整方法拒绝（"canonical block at BCI N … has more than one owner in the completed Region tree"）。二分查找是该形状的最高频实例（`loopElseIfRet`/bsearch 同形）。同文件对照矩阵钉死判别变量：

| 形状（同 CB 类，双腿） | 结果 |
| --- | --- |
| while + else-if 阶梯 + 早退（`loopElseIfRet`） | **拒**（canonical-overlap，BCI 56 join 被双主张） |
| while + else-if 阶梯 + 无早退（`loopElseIfNoRet`） | 恢复 |
| while + 单 if-else + 早退（`loopIfElseRet`） | 恢复 |
| 无循环 + else-if 阶梯 + 早退（`noLoopElseIfRet`） | 恢复 |

拒绝面安全（body 空=缺 return 编译失败=响亮）。jadx 完整解（嵌套 if-else）。**注意**：巡查原记载的"类级上下文确定性缺陷"归因已被 root 二分实验撤回（隔离形同样拒绝；详见巡查 README 归因修正段）——本片按方法形状立项。

## What Changes

- Region 走查的 join 选举处理"循环体内 else-if 阶梯的早退臂"：阶梯各臂的出口汇合块（现为 BCI 56 形的多 owner）须由**单一**区域主张（阶梯整体作为循环体的一个 if-else 树，早退臂的 return 是树的终止叶而非 join 的第二主张者）。
- 复用既有 If/join 选举机制（`region.rs`）；不新建几何层实体；单臂 if-else+早退（已恢复形）的判据零改动。
- MVP：单层阶梯（一个 else-if 链）+ 单一早退臂 + 无异常表；多层阶梯、switch 混合、异常区交叉保持拒绝并如实记录。

## 硬不变量

1. 对照三形（无早退/单 if-else/无循环）渲染逐字节不变；
2. 不得产出"可编译且行为不同"文本（早退语义=控制流事实，非改序片但同守恒）；
3. irreducible/crossing-exception 等既有拒绝零触碰。

## 验收

- `loopElseIfRet` 与 bsearch（`BS` 全类）恢复（0 canonical/0 引注），剥离编译 exit 0、`-Xverify:all` 行为与原一致（BS 原 class 输出 `2/-2/...` 按 fixture 实测钉死）；
- 对照三形 + 负例（异常表交叉形、双层阶梯 MVP 外形）按上表钉死；
- 全门禁 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：循环内 else-if 阶梯的早退臂按单一 if-else 树呈现，方法行为完整。

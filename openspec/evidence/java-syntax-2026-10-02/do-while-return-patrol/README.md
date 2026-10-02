# do-while(false) 体内 return 巡查（2026-10-02）——含静默错编面

[dj 验收](../loop-double-jump-patrol/README.md) 中 root 复核发现的独立缺陷族（主线 `83ed9a2e`）。固定转录 [fixture](fixture/)（D1 探针 + L5 双面；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）。

## 双面表现（同根因，两严重度）

| 形态 | 主线 Jarde | 重编行为 |
| --- | --- | --- |
| L5.dblJumpDoWhilePlain：while 体内 `do { if(%3) break; if(>7) return "early"; } while(false);`，return 前无副作用语句 | **部分呈现 + BCI 26/28 引注**（引注区含 return） | 类可编译但 return 被静默丢弃：`i=10` ≠ 原类 `early` ——**静默错编**（最危险类） |
| D1.retInDoWhile：同结构但 return 前有 `hits++` 计数副作用 | 整方法拒绝（3 处引注） | 不可编译（安全失败） |

## 根因

do-while(false) 降低体（javac 一次通过 + break/return 出边）中的 **return 边**不被区域树拥有：含可呈现副作用（hits++）时块计费不平→整方法拒绝（安全）；无副作用时区域走查给出部分 if/else-if 呈现、return 边落入引注区——引注行被剥离 harness 删除后方法仍因尾 return 可编译，**静默改变行为**。

## 处置方向

`recover-return-in-do-while-false`（恢复 + 失败闭合双义务）：
1. **恢复**：do-while(false) 体的语句集（含条件 return 与 break）由该区域拥有并按 `do { … } while (false);` 呈现（region 归属扩展，与 dj 切片的边归类相邻）；
2. **失败闭合（不变量）**：区域证明失败时**必须整方法拒绝**——呈现层不得产出"引注区含控制流改变语句（return/throw/break/continue）且剥引注后仍可编译"的部分体；此不变量以 L5 形回归测试钉死（修前=可编译静默错编，修后=恢复或整方法拒绝）。

原 class 为行为基准。

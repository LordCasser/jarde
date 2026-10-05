# 非循环标签块 + 无限循环巡查（2026-10-05 root）

## 健康面（负结果——无限循环族全过）

[fixture/LB.java](fixture/LB.java)（`--release 8`）：
- **`while(true){…break}`** → `do { … } while(反条件)`——`while(true)` 本就至少执行一次体，do-while 归一**行为等价**；
- **`for(;;){…return}`** → 同 do-while 归一（等价）；**`do{…}while(true)+break`** → `while(cond)` 归一（等价）；
- 三形行为验证 `6/5/104` 段逐行一致。

## 发现：非循环标签块（第 10 个新证缺口，响亮拒绝）

`outer: { inner: { if(x==1) break outer; if(x==2) break inner; … } … }`——**break 到非循环标签**（跳过 join）被拒："the intermediate join, bridge and outer conditional do not form a closed value"。

**jadx 有解**：if 嵌套等价形（`break outer` → 外层 skip 剩余、`break inner` → 内层 skip）——[results/jadx-LB.java](results/jadx-LB.java)。用 jadx 形 stub 后**行为逐行一致**（`100/110/111` 段验证）。

## 处置

第 10 个新证窄缺口（呈现域：非循环标签 break 的 if 嵌套等价呈现——与循环标签 `break loopN` 的既有通道不同落点）。登记 + 立窄片。

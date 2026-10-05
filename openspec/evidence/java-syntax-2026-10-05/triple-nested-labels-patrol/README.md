# 三层嵌套循环标签矩阵巡查（2026-10-05 root）——可恢复性缺口（SAFE）：多层标签多出口整方法拒

## 发现

两方法全部**整方法响亮拒绝**（"local N crosses a quoted fallback region"，空 body 缺 return → 编译失败 = SAFE）：
- `matrix`：**三层** `outer:/mid 嵌套/inner`，内层含 `continue outer` + `break outer` + 裸 `continue` **三种出口并存**；
- `findMid`：仅两层但 `continue mid` + `break mid` 并存。

**判别 vs #11 monitor 巡查（两层标签恢复）**：#11 的标签语句是单一出口用途（只 break-outer 或只 continue-outer）；本探针是**同一内层体含 ≥2 种不同标签出口**或**三层深度**——循环控制局部的定义-使用切片跨多层区，渲染层过引到整方法。jadx 全解（for/while 混合形+`break loop0`）。

## 处置

**local-scope 片第 9 锚**（同 crosses 机制族，新形状：多层标签多出口矩阵——解析器/矩阵遍历真实形状）。

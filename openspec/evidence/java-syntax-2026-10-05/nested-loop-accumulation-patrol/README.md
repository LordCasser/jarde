# swap/手动复制/二维遍历巡查（2026-10-05 root）

## 发现：嵌套循环累积 crosses——整方法拒（安全）但应可恢复

`sum2d`（`for(i){ for(j){ s += m[i][j]; } }`——**嵌套循环中局部累积**）整方法 not recovered（"local 1 crosses"，anchors 跨 7 BCIs）——幸存文本空 body=**编译失败（缺 return）= 第一不变量不违反（安全）**。jadx 完整解（双层 for++=）。

归 [local-scope 片](../../../changes/preserve-local-scope-across-exception-regions/) **第 8 锚**：**嵌套循环累积**——与第 7 锚（循环×switch 交叉）同 crosses 诊断族、又一控制流形状；**判别点**：单层循环累积（#103 sumSquares `t += x*x`）恢复、嵌套层累积拒（内层循环体的赋值使局部定义-使用切片跨外层）。

## 健康面（负结果）

- **静态字段 swap**（`int t = a; a = b; b = t;` temp 局部）恢复；
- **数组元素 swap**（`t = xs[0]; xs[0] = xs[1]; xs[1] = t;`）恢复；
- **手动数组复制**（new + for 逐元素拷贝）恢复；
- 行为 `2,1/[9, 7]/[4, 5]/10`（前四方法经渲染验证一致）。

## 处置

local-scope 片补第 8 锚（锚家族现 8 形：异常区×4+catch链+finally+equals+扫描器+嵌套循环）。

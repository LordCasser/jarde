# 复合解析循环（词法扫描器形状）巡查（2026-10-05 root）

## 发现：for+switch 复合中局部 SB crosses——整方法拒（安全）但应可恢复

`scan`：`for + switch(char) 分派（含 continue/空 case 组/digits++/SB.append/prev 状态）`——**词法扫描器最常见形状**——整方法 not recovered（"local 1 crosses a quoted fallback region"，local1=SB out；anchors 跨 12 BCIs）。**幸存文本为空=第一不变量不违反（安全方向）**，但 jadx 完整恢复（loop+switch+SB+continue 全解）——可恢复性缺口。

归 [local-scope 片](../../../changes/preserve-local-scope-across-exception-regions/) **第 7 锚**：局部 SB 在 **for+switch 复合**中跨分支消费（crosses）——与既有 6 锚（异常区×4 + catch链+finally + equals 无异常表形）同诊断族、新控制流形状（循环×switch 交叉）。

## 健康面（负结果）

- `sumSquares`（for-each 循环乘积累积 `t += x*x`）恢复——循环内乘法复合在**局部**位不触发失败面（与 #99 实例位对照：局部累积全类型恢复）；
- main 中 scan 调用+字符串拼接正常呈现（级联仅 scan 本体）。

## 处置

local-scope 片补第 7 锚；行为 `ABAZ/4/3/13`。

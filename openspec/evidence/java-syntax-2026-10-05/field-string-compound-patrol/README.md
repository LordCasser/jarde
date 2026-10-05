# 字段 String 复合拼接巡查（2026-10-05 root）——critical 第 11 锚（copy+依赖链混合）

## 发现：`this.field += "[" + x + "]"`（业务代码最高频字符串累积模式）整段静默丢失

`SC.add`（链式 `add("x").add("y")` 方法体）渲染仅剩 `return this;`——**隔离编译 exit 0、`f` vs 原 `f[x][y]`**（new SB + 整条 append 链 + toString + putfield 全吞）。javap：`new SB; dup_x1@8`（receiver 在 builder 下压栈）——receiver dup_x1 跨整条链。诊断混合：copy 族（"copy at BCI 3"）+ 依赖链族（7×"not bounded"）+ "value at BCI 32 comes from an Other"——三族交汇的最复杂形。

## 判别（同巡查健康面——负结果）

- **局部 String 复合循环**（`s += "-" + i` 在 for 内）：完美折回 `local2 = local2 + "-" + local3`（javac 的 SB 链被完全还原为 + 链）；
- 单次局部复合恢复；**嵌套 lambda**（`() -> () -> ...`）与**柯里**（`x -> y -> x+y`）全恢复——伴生重命名链（`lambda$nested$3$jarde`）+ 内联外层体，行为精确；
- 族计数：**11 锚 / 3 族**——本锚为 copy×依赖链交汇（字段复合 + 引用型 SB 链）。

## 处置

soundness spec 的三族 Requirement 已覆盖（诊断文本族枚举含 copy 与 chain）；ledger 更新至 11 锚；恢复侧锚补入 chained-field-assignment 片（字段复合的 String/引用型 SB 链形）。

# equals/hashCode 契约巡查（2026-10-05 root——真实代码最普遍的两个方法）

## 结果

- **hashCode 完美恢复**：`31 * result + field` 素数累积、`Double.doubleToLongBits` + `(temp ^ temp>>>32)` XOR 折叠、hex float 字面量（`0x1.4000000000000p1d`）逐字精确——IDE 生成形全过；
- **toString 完美恢复**（SB 链 + char 实参 `'}'`）；
- **equals 部分引注——安全（编译失败方向）但为可恢复性缺口**：`HE that = (HE) o` cast 局部触发 "local 2 crosses a quoted fallback region"，内层字段比较链引注掏空、`getClass() !=` 分支体空——剥离后 javac **缺少返回语句 exit 1**（第一不变量不违反，不可编译）。equals 契约（getClass 严格形+cast+字段链）是**最普遍的方法形状**，应可恢复——归 [local-scope 片](../../../changes/preserve-local-scope-across-exception-regions/)新锚（同 "crosses" 诊断族、**无异常表形**：普通分支 + cast 局部跨内层引注区）；
- main 双构造消费=第 4 诊断族（3 consumers）第 2 位点再证（构造器局部 `new HE(...)` 三消费——族广度确认，NI 之外）。

## 处置

local-scope 片补 equals 契约锚；不立新片（安全+已有 owner）。

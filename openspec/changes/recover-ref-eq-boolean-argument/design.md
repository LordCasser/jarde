## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/string-ops-patrol/README.md)：S1 BCI 148–156（`if_acmpne 155; iconst_1; goto 156; iconst_0; append(Z)`）。既有布尔上下文切片（赋值/return/条件位）建立了"分支选择 0/1 = boolean 值"的 SSA 证明形态（`BooleanProofContext` 一族）；实参位转换判定（build.rs 调用实参处）对 int 值→boolean 形参无该形态入口。第一个取证义务：定位实参位的类型判定点与既有布尔证明的最近接入位（可能在 invocation_argument 的 presented 类型解析处）。

## Goals / Non-Goals

**Goals:** `if_acmpXX`/`if_icmpXX` 双臂 0/1 合流值在 boolean 形参位呈现 `a == b` / `a != b`（goto 后取反臂按反义拼写）；S1 恢复且行为一致；既有布尔位证明与负例零回退。**Non-Goals:** 非相等类分支（<、> 等已由条件证明覆盖的形态）；boolean→int 反向；装箱 Boolean；多级布尔表达式嵌套（既有条件通道）。

## Decisions

1. **实参位接入既有布尔证明**：调用实参解析时，若值定义为比较双臂合流（既有 BooleanProof 的分支值形态）且形参 boolean，呈现比较表达式；判据复用而非复制。
2. **验收锚定**：S1（`falsetrue`/`falsetrue` 尾段）+ 变体（`!=`、数值相等入 `assertEquals(Z,Z)` 双参、比较结果存后传参）；负例（0/1 非比较来源的 int → boolean 位保持拒绝）。

## Risks / Trade-offs

- **把任意 0/1 误判为比较** → 判据锚定双臂合流的分支形状（iconst_1/iconst_0 各一臂 + goto 汇合），非比较来源负例钉死。

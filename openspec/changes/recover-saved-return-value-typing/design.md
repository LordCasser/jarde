## Context

复现（主线 c082ce70，`target/debug/jarde-cli`）：

```
class-source --input openspec/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/fixture/T2.class --policy single-class --class T2
# voidBodyReturnInside 输出：Object local1 = "in"; return local1;（方法返回 String，不可编译）
```

字节码（T2.class）：`12: ldc "in"; 14: astore_1`（saved store 在受保护体内）、清理链后 `19: aload_1; 20: areturn`。既有钉死期望 `p3_java_recovery.rs:1389` 的 `Object local1 = null;` 是 null 保存值（无窄型）的合理拼写。`written_type`（build.rs 24316）已实现值→类型细化（`array_of_value` + `value_type`），Test2 曾以 `array_of_value` 修复同类"栈宽类型压窄值类型"缺口。诊断第一步：定位 TWR saved-return 声明实际走的决策点（`decide_types`/声明规划处对 saved store 的槽类型来源），确认它为何得到 Object（未调用 written_type、或 SSA 该值类型本身是宽型、或 saved-return 有专用硬编码）。

## Goals / Non-Goals

**Goals:** 非 null 保存值的声明按生产者类型拼写（字面量/调用/构造/数组沿用 `written_type` 既有答案）；null 保持 Object；细化失败回退现拼写；`voidBodyReturnInside` 整类可重编。

**Non-Goals:** finally 家族的 saved-return 呈现（其证书已有各自拼写，若恰好同病，仅当测试证明才顺带修复并在报告区分）；跨块/phi 保存值类型推断（回退）；`Object` 语义改变。

## Decisions

1. **复用 `written_type`，不加新机制。** 在 saved-return 声明决策点调用 `written_type(ssa, operations, 保存值)`；`Some(ty)` 用之，`None` 或 Err 回退现拼写。若诊断发现决策点根本不在 build.rs（如 report/emit 侧），落点跟随事实但同一复用原则。
2. **验收锚定**：T2 整类重编通过；`p3_java_recovery.rs:1389` 的 null 期望不变；Tf/Test5/7/9 等 saved-return 家族测试全绿（套件内钉死）；新增回归测试覆盖 String/int/构造/调用返回四种保存值。

## Risks / Trade-offs

- **细化改变了正确钉死的既有输出** → 只影响非 null 保存值；既有期望逐字对照（null 不变）；若发现既有期望依赖 `Object` 拼写的非 null 场景，如实上报交 root 裁决而不是改期望。
- **与 finally 家族拼写通道冲突** → 本片只触 TWR saved-return 决策点；finally 各证书拼写不碰。

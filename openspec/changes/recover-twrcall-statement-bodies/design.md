## Context

固定 T2（[巡查证据](../../evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/README.md)）：`popBody` 体为 `aload_0; invokevirtual toString; pop`，主线整方法回退；`voidBody`（同布局 void 调用）恢复——唯一变量是"非 void 调用 + pop"。TWR 体证明在 `guard.rs` 资源证书内逐语句核验正文（`Unproven::Body` 的子集），该子集现有 void `invoke`、return 等。

## Goals / Non-Goals

**Goals:** 正文语句子集接受"非 void `invoke` + 紧随 `pop`"为一个调用语句：pop 位于调用后同块紧邻、pop 的唯一读值是该调用的结果值（SSA 身份）。呈现复用现有调用语句通道（结果丢弃即源码分号）。

**Non-Goals:** 调用结果被赋值接收的体（主线既有 saved-return 通道**已恢复**该形态，design 初稿表述过时，以主线行为为准）；`Pop` 之外的栈消费形态；字段读/自增等其它 statement-expression（各自另片）；17b 的包围具名 catch；finally 家族正文（其证书自有文法）。

## Decisions

1. **形态判据放体证明的语句枚举处**：对每条正文语句，新增"非 void `invoke` + 紧随 `pop`（pop 读值 == 调用写值，SSA same）"接受分支，与 void 调用同一呈现通道；注释说明 statement-expression 语义（结果被源码丢弃）。预算沿用体证明现有计费。
2. **验收锚定 T2 与变体**：popBody/popBodyVoidTouch 恢复且行为等价（结果丢弃无观察效应——探针以可观察 void 调用 + 不可观察调用混排验证顺序）；voidBody/voidBodyReturnInside 逐字不变；结果被消费变体（赋值接收）保持拒绝。

## Risks / Trade-offs

- **误收有观察效应的 pop**（pop 前结果被 dup/存储） → 判据要求 pop 是调用后紧邻且唯一读者；"结果另读者"负例拒绝。
- **与 17b 交叠** → 本片不触碰 `enclosing_clauses` 与 catch 呈现；C4.twrNamed（叠加形状）在本片后仍拒绝（17b 范围）。

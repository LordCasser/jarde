## Why

[objects-enumset 巡查](../../evidence/java-syntax-2026-10-05/objects-enumset-patrol/README.md)：`EnumSet.of(Flag.A, Flag.C)`（枚举集合工厂——枚举 API 最常用方法）整方法响亮拒绝，诊断 "the parameter 0 of the invocation at BCI 6 is declared `java.lang.Enum` presents `OB$Flag` but the invocation requires `java.lang.Enum`"。jadx 直接解（`EnumSet.of(OB.Flag.A, OB.Flag.C)` 无 cast）。

**平台实参扩宽族第 4 员——首个类位（ superclass）扩宽**：`EnumSet.of(E first, E... rest)` 擦除为 `(Enum, Enum[])`，javac 在调用点发射 `checkcast java/lang/Enum`。前三员（CharSequence/Comparable/Serializable，[java-syntax-2026-10-05](../../evidence/java-syntax-2026-10-05/) 各巡查）均为 java.lang **接口**位扩宽；本员是 **java.lang.Enum 父类位**扩宽，同一机制、新表行。



> **root 追加级联位点（[诊断普查](../../evidence/java-syntax-2026-10-05/diagnosis-census/README.md)）**：OB.flags 内 `retainAll(fs)` 的 `EnumSet presents java.util.Collection`（Collection 接口位第 2 例）——Enum 锚同方法伴随行；实现时一并覆盖（同一 widen 通道）；若 Collection 位在其他类独立出现（非 EnumSet 实参）则升第 5 表。

## What Changes

- 平台扩宽表新增 `java.lang.Enum` 行（枚举常量实参 → Enum 形参位；`EnumSet.of`/其他 `Enum` 参数 API 的调用点 checkcast 投影）；
- 与三张姊妹表同构：结构判据零放宽、封闭行、三方一致（源/池/调用点描述符）；
- **四表合并派发**（charsequence+comparable+serializable+enum——同一 owner 同构通道）。

## Impact

- specs/java8-recovery/spec.md：扩宽 Requirement 增 Enum 行 scenario（`EnumSet.of` 两常量形恢复 + 诊断不改 + 无枚举调用点零漂移）；
- 复用 [recover-charsequence-argument-widening](../recover-charsequence-argument-widening/) 的实现表协议（每行 javadoc + hex + 三方判据）。

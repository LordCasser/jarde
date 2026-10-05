# java8-recovery 平台实参扩宽（Enum）修订

## ADDED Requirements

### Requirement: 枚举实参到 Enum 形参位的平台扩宽调用点恢复
jarde 对 `java.lang.Enum` 形参位上的枚举常量/枚举型实参调用点（擦除后 javac 发射 `checkcast java/lang/Enum`），SHALL 呈现无冗余 cast 的调用（如 `java.util.EnumSet.of(OB.Flag.A, OB.Flag.C)`），其三面判据（源形/池形/调用点描述符）一致。

#### Scenario: EnumSet.of 枚举常量形恢复
- **WHEN** 输入为固定 `OB` 的 `flags`（`EnumSet.of(Flag.A, Flag.C)` + retainAll + contains）的 Java 8 class 并恢复
- **THEN** 该方法 SHALL 恢复为 `EnumSet.of(...)` 调用无 `(java.lang.Enum)` cast 呈现；行为输出 `hasA/no` 与原 class 一致；诊断文本族不变

#### Scenario: 非枚举实参调用点零漂移
- **WHEN** 输入不含枚举实参的类（Objects.equals/hashCode/toString/deepEquals/requireNonNull+supplier 全形）
- **THEN** 渲染 SHALL 与修订前逐字节一致（扩宽表新增行不触碰无枚举路径）

#### Scenario: 真 Enum 型实参（枚举变量直传）恢复
- **WHEN** 实参本身已是 `java.lang.Enum` 声明类型（无 checkcast 发射）
- **THEN** 调用点呈现 SHALL 不引入新 cast（现状保持）

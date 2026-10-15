> **历史范围说明（2026-10-15）：** 以下“双侧物理定义/单边快照外”的表述记录本片最初MVP范围。后续[实参单边扩宽](../../../recover-platform-interface-argument-widening/specs/java8-recovery/spec.md)及其[standalone用户子类到外部用户父类的pin迁移与Root追认](../../../recover-platform-interface-argument-widening/verification.md)、[异构数组元素条款](../../../recover-heterogeneous-array-init/specs/java8-recovery/spec.md)已各自说明适用的header名称终点。旧two-sided正负fixtures继续作为原支持子情形；source及实际必经的中间header仍需既有环境证据。参见[缺少Mid的负控](../../../recover-covariant-child-array-initializers/specs/java8-recovery/spec.md)与[已归档停止设计的Mid/Greet边界](../../../archive/2026-10-15-preserve-class-source-dependency-proof-stops/design.md)。这些引用按各自站点解释；method站点快照外用户interface终点及terminal重复物理定义的独立政策在本次文档澄清中保持未决。

## ADDED Requirements

### Requirement: 同快照用户类层级的实参上转型可呈现

系统 SHALL 在调用实参的呈现类型 T 与要求类型 U 均为本快照物理类定义、且 T 沿其类文件 header 的 extends 链与 interfaces（传递、有界）可达 U 时，按既有 widening 模式呈现该实参并保留要求类型拼写。平台闭集、数组、同名与 Object 回答 SHALL 先行且零变化；任一侧不在快照、链不可达或超界的形状 SHALL 保持既有拒绝。

#### Scenario: 用户实现传接口形参恢复

- **WHEN** `viaInterface(new En(), "v")`（En implements Greet）家族三方 Java 8 重编运行
- **THEN** 调用完整呈现，`java -Xverify:all` 逐路径与原 class 一致（含全部输出行）

#### Scenario: 既有回答与负例不变

- **WHEN** 输入走平台闭集/数组/同名/Object 分支，或快照内无关联两类、单边快照外
- **THEN** 输出与本变更前逐字一致

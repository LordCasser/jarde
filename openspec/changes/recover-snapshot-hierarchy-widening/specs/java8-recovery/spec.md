## ADDED Requirements

### Requirement: 同快照用户类层级的实参上转型可呈现

系统 SHALL 在调用实参的呈现类型 T 与要求类型 U 均为本快照物理类定义、且 T 沿其类文件 header 的 extends 链与 interfaces（传递、有界）可达 U 时，按既有 widening 模式呈现该实参并保留要求类型拼写。平台闭集、数组、同名与 Object 回答 SHALL 先行且零变化；任一侧不在快照、链不可达或超界的形状 SHALL 保持既有拒绝。

#### Scenario: 用户实现传接口形参恢复

- **WHEN** `viaInterface(new En(), "v")`（En implements Greet）家族三方 Java 8 重编运行
- **THEN** 调用完整呈现，`java -Xverify:all` 逐路径与原 class 一致（含全部输出行）

#### Scenario: 既有回答与负例不变

- **WHEN** 输入走平台闭集/数组/同名/Object 分支，或快照内无关联两类、单边快照外
- **THEN** 输出与本变更前逐字一致

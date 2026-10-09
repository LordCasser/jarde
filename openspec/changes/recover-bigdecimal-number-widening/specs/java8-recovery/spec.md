## ADDED Requirements

### Requirement: Release-8 BigDecimal Number widening

系统 SHALL 在现有 Java release-8 平台契约适用时，将实际呈现为 `java.math.BigDecimal` 的值赋给 `java.lang.Number` 调用形参或引用数组 initializer component；不得借此放行未证明的其他类型对或改变真实值身份。没有其他准确事实时，不同 release、反向赋值或不同目标 MUST 保留拒绝。

#### Scenario: Main-only BigDecimal array
- **WHEN** release-8 Main-only 输入创建 `Number[]{new BigDecimal("1.25")}`，随后读取数组长度及首元素输出
- **THEN** 系统 SHALL 完整呈现数组声明与输出语句，保留一次构造、实际数组写入及各表达式来源，独立重编后的 stdout/stderr/exit 与原程序一致

#### Scenario: BigDecimal invocation argument
- **WHEN** release-8 方法以实际 BigDecimal 值调用 descriptor 明确要求 Number 的已支持调用
- **THEN** 系统 SHALL 复用该准确平台关系呈现实参，保持原 descriptor 所需类型和单次求值

#### Scenario: Unproved platform pair
- **WHEN** 除本闭集事实外没有其他证明，输入要求 BigDecimal→不兼容类、Number→BigDecimal、表外类型→Number 或不同 release 的该事实
- **THEN** 系统 MUST 保持原拒绝，不以名称相似、已验证字节码或本正例推断任意 subtype

### Requirement: Preserve complete ordinary builder semantics

系统 SHALL 区分字符串拼接优化拒绝与完整正文拒绝；普通调用链能够正确呈现时，保留原 StringBuilder new/append/toString 求值顺序和次数 MUST 视为有效恢复。预算不足或取消仍 MUST 保留准确停止状态，不发布半个数组 initializer 或省略原始必要效果。

#### Scenario: Array length inside the ordinary append chain
- **WHEN** 上述 BigDecimal 正例的 concat 优化仍拒绝 arraylength，而普通调用路径可呈现完整链
- **THEN** 系统 SHALL 保留显式调用链，数组长度、常量、数组元素按原 append 次序求值，无 bytecode fallback 片段且完整程序行为匹配

#### Scenario: Stop during recovery
- **WHEN** 现有 initializer 或正文恢复路径实际触发预算停止或取消
- **THEN** 系统 MUST 沿用准确停止报告与原子提交边界，不把本平台关系当作忽略停止的许可

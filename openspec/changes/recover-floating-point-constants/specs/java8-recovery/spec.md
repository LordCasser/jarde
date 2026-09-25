## ADDED Requirements

### Requirement: Floating constants retain their type and observable bit pattern

系统 SHALL 在已有可恢复区域内呈现有限float/double常量、正负无穷以及标准正quiet NaN，保留数值类型、有符号零、分组及可观察值。系统 MUST NOT 把float常量无声地加宽为double，或以NaN比较结果相同为理由删除其符号及payload差异。

#### Scenario: Finite boundaries and ordinary values retain exact bits

- **WHEN** 方法返回或消费正负零、次正规值、最小正规值、最大有限值及普通float/double常量
- **THEN** 实际恢复正文 SHALL 可按目标Java语法重编译；在记录的JDK对照环境中，返回值的raw bits与原class一致

#### Scenario: Overloads and nested expressions retain meaning

- **WHEN** 浮点常量经过局部、算术、取负、比较或有float/double重载的调用实参
- **THEN** 恢复结果 SHALL 保留原操作数静态类型、分组、比较极性及每个调用的求值次数与顺序，不生成词法自减或错误重载

#### Scenario: Representable special values preserve the tested values

- **WHEN** 原class包含正负无穷或标准正quiet NaN常量
- **THEN** 系统 SHALL 用无额外运行时调用的常量表达式呈现，并在记录的JDK对照环境中保留raw bits及其参与运算的行为

#### Scenario: Other NaN bit patterns remain explicit boundaries

- **WHEN** 常量具有本项不能精确呈现的NaN符号、payload或signaling位模式
- **THEN** 系统 SHALL 明确引用该常量及其失败消费，不归一为默认NaN，不伪造正确恢复状态；其它已经呈现的语句及必要生产者效果保持可见

#### Scenario: Compiler folding must not silently change a runtime operation

- **WHEN** 合法字节码实际执行闭合浮点常量运算，而直接恢复为Java常量表达式不能证明保留原位模式，包括标准NaN后的fneg/dneg
- **THEN** 在已有位置与作用域证明覆盖的形状中，系统 SHALL 复用非final局部保存必要操作数，使真实操作留在运行时，并通过实际完整正文重编译保留记录环境中的原始raw bits；无法证明位置时保留明确引用及必要常量、操作、消费者来源，不发布被javac折叠后改变NaN符号的表达式

### Requirement: Floating constant recovery preserves evidence and bounded publication

浮点常量的构造、文本和来源 SHALL 遵循既有预算与产物提交契约，合成呈现不伪造独立原指令。

#### Scenario: Evidence selection retains the same source text

- **WHEN** 同一方法以默认及完整来源请求恢复，预算足够
- **THEN** 正文 SHALL 逐字相同，默认请求不发布来源表，完整来源关联常量及消费者的真实BCI和成员身份

#### Scenario: Output and source bounds preserve committed artifacts

- **WHEN** 正文或来源计费达到边界
- **THEN** 系统 SHALL 保持原停止语义，不提交未支付文本；来源不足时已提交正文保持不变，未完成来源如实报告

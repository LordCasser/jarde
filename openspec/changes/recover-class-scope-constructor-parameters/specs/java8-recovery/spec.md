## ADDED Requirements

### Requirement: 类作用域构造参数的忠实泛型恢复

系统 SHALL 对已发布类泛型作用域、物理擦除一致且完整正文已证明的构造器参数恢复其 Signature。引用类变量的参数 MUST 保留原类 binder，MUST NOT 为同名类变量新增构造器形式类型参数。完整文本、物理成员身份、初始化与字段写顺序 SHALL 保持。

#### Scenario: 初始化后直接保存类变量参数
- **WHEN** Hold<T> 在 Object() 初始化后仅按原序执行 this 字段的原参数赋值，参数声明、字段身份及所有参数使用均获证明
- **THEN** 输出 SHALL 恢复 Hold(T) 与可安全发布的 T 字段；完整类 SHALL 在真实 javac8/javac23 的 debug/no-debug 四条腿重编、验证执行和泛型反射一致

#### Scenario: 数组、上界、多变量与宽槽
- **WHEN** 相同闭合正文使用 T[]、T extends Number、多类变量或 long/double 前缀参数，参数槽和类型擦除与原类一致
- **THEN** 输出 SHALL 恢复对应构造参数类型与 binder；long/double 不得使随后参数错位，字段值行为和声明泛型反射 SHALL 保持

#### Scenario: 类变量的未使用构造参数
- **WHEN** 已证明的 Object() 空体构造器声明类作用域 T 参数但不使用该参数
- **THEN** 输出 SHALL 恢复 T 参数，构造器形式类型参数数量 SHALL 保持为零

### Requirement: 构造参数与字段投影的单向源码证明

系统 SHALL 独立证明构造参数发布，再依据实际发布的参数类型决定字段投影。未发布的构造器 Signature MUST NOT 冒充实际源码参数；构造器发布 MUST NOT 依赖字段先投影，也 MUST NOT 因字段与参数擦除相同而合并不同 binder。

#### Scenario: 不同类变量写入字段
- **WHEN** CrossHold<T,U> 的 U 参数通过字节码中没有保留检查的源级 cast 写入 T 字段，且构造器的直接参数正文可独立证明
- **THEN** 系统 SHALL 允许恢复 U 构造参数并可靠拒绝不兼容的 T 字段投影；完整文本 SHALL 可编译且行为一致，不虚称该字段泛型反射一致

#### Scenario: 方法级同名类型变量
- **WHEN** 构造器声明自己的 T，与类 T 同名但 binder 不同
- **THEN** 系统 MUST NOT 把构造器 T 当成类 T；已有可证明的方法级构造器 SHALL 保持正确归属，其字段投影 SHALL 遵循真实赋值类型

### Requirement: 构造器证明不完整时保守且原子拒绝

系统 SHALL 只在完整正文证明覆盖全部构造器参数使用时改变声明。未知调用、改写参数、控制流汇合、异常处理、非 Object 前导、this 委派、字段身份不闭合或源码作用域不可用 SHALL 保持明确拒绝。预算和取消 SHALL 传播既有停止状态，MUST NOT 发布半个声明或丢失字段写。

#### Scenario: 未证明正文和继承前导
- **WHEN** 构造器含参数改写、phi/call RHS、异常处理、非 Object 父类或 this 委派
- **THEN** 本片新增路径 SHALL 不发布泛型参数，原正文、物理参数与拒绝来源 SHALL 保留；不能把响亮拒绝计为完整恢复

#### Scenario: 停止不冒充完成
- **WHEN** 候选提取、作用域/字段证明或源码发布遇到预算或取消
- **THEN** 系统 SHALL 保持停止状态和成员级原子性，未完成构造器 Signature MUST NOT 参与后续字段赋值证明

#### Scenario: 未证明的 this 委派调用目标
- **WHEN** 原 Object 参数的 this 委派调用者指向可独立恢复 T 的构造器，但调用实参源码类型尚未证明，或实际构造接收者身份未知
- **THEN** 新增 class-scope 目标投影 SHALL 保持物理参数与明确拒绝，完整类 SHALL 保持基线可编译和行为；系统 MUST NOT 仅凭物理 descriptor/arity 发布目标泛型参数

#### Scenario: 独立 raw new 调用不冒充 this 委派
- **WHEN** 同类构造调用的唯一 SSA 接收者是独立 new-site，实际源码以 raw 类分配且既有完整绑定证明成立
- **THEN** 系统 SHALL 保持该构造器类作用域参数恢复，不得仅依据 caller 为构造器或常量池引用把它当作 this 委派

#### Scenario: 既有候选消费者
- **WHEN** 同次输出还进行匿名类构造前导隐藏或静态成员空构造器证明
- **THEN** 直接字段赋值候选 MUST NOT 被当作可隐藏的空体或完整 super 转发；已有冻结成员类、匿名类与方法级泛型构造器行为 SHALL 保持

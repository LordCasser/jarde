## ADDED Requirements

### Requirement: Blank static final writes preserve field binding

系统 SHALL 在已可恢复的普通类静态初始化区域内，将同源声明证明的当前类 blank static final 字段写入呈现为合法的简单字段赋值；MUST 保持字段身份、指令处的求值次序、异常和写入次数。恢复局部名与合成名 MUST NOT 遮蔽这些字段。系统 MUST NOT 为此搬移初始化效果、伪造局部赋值、删除其它限定符或推断缺失字段声明；此规则不承诺恢复接口声明初始化或非法 final 写入。

#### Scenario: Initializers preserve order and both branches

- **WHEN** 普通类依次用两个有可观察计数的调用写入 blank static final 字段，或在既有可恢复的两个条件分支中分别给同一字段赋值
- **THEN** 恢复完整类 SHALL 能以目标 Java 8 配置重编译，受控执行的字段值、调用次序和计数与原 class 相同；不能限定赋值左侧、删掉分支或把调用移到字段声明

#### Scenario: A generated local would shadow a field

- **WHEN** 当前类有名为 `local0` 与 `local0_2` 的 blank static final 字段，初始化字节码同时需要一个无 debug 名称的局部变量
- **THEN** 局部变量 SHALL 获得未占用的稳定名称，每个简单字段写仍绑定原字段；完整类的编译与执行 MUST 通过，不能因简单去限定而写回局部

#### Scenario: Other field contexts retain their identity

- **WHEN** 写入不具备本项前提，包括其它类字段、非 final 字段、实例字段、ConstantValue 字段、缺失声明或同名歧义
- **THEN** 系统 MUST NOT 以本项规则去掉限定符或伪造字段事实；既有可恢复形式或明确拒绝与生产者来源 SHALL 保留，接口字段初始化缺口不得被掩盖

#### Scenario: Physical sources and bounded publication remain consistent

- **WHEN** 调用者选择默认证据、全部来源、正文预算不足或来源预算不足来恢复简单字段写
- **THEN** 默认请求 SHALL 保持相同正文并省去未请求来源；完整来源 SHALL 映射实际字段写与必要生产者 BCI/物理成员，正文停止不得发布半条语句，来源停止不得改变已提交正文或发布未支付的来源

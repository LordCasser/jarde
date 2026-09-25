## ADDED Requirements

### Requirement: Complete class source projects only reconstructible bridge forwards

系统 SHALL 仅在桥接方法被完整证明为保留的源级覆写之纯转发，且当前类的已解析继承契约要求该擦除签名、Java 8 编译会重新生成等价桥接调用时，将该方法从完整类的 Java 声明中投影掉。物理方法表、桥接 flags、原方法恢复结果、转发目标与投影归属 MUST 保留供审计；完整类源码 MUST NOT 同时声明仅返回类型不同的同名同参方法。

#### Scenario: Generic interface override has a compiler bridge
- **WHEN** Java 8 类实现已解析的泛型接口，以 `String get()` 覆写接口的擦除签名 `Object get()`，其原 class 还声明一个只调用该覆写的 `ACC_BRIDGE|ACC_SYNTHETIC Object get()`
- **THEN** 恢复的完整 Java 类 SHALL 编译并由 javac 重新生成可调用的 `get()Object`；对直接调用、接口擦除调用及零引用执行路径的结果 SHALL 与原 class 一致，桥接方法的物理报告仍可定位

#### Scenario: Bridge carries an independent effect
- **WHEN** 标记为桥接的方法在转发前写字段、执行额外调用或含未证明的转换
- **THEN** 系统 MUST 拒绝把它当作可重建桥接声明投影，MUST 保留效果、原方法身份与拒绝原因，MUST NOT 仅改 Java 方法名并声称保留原擦除签名的调用

#### Scenario: No inherited erased method requires regeneration
- **WHEN** 方法即使带桥接标志且与另一个方法同名同参，已解析的父类或接口中仍没有需要 javac 重新生成的擦除签名
- **THEN** 系统 MUST 拒绝投影该方法；以原签名调用它的能力 MUST NOT 被静默删除

### Requirement: Bridge projection remains proof-bound and atomic

桥接投影 SHALL 由同一次成员恢复的结构化桥接结论和按预算取得的类级继承事实决定，MUST NOT 反解析已发射的 Java 文本、凭 `ACC_BRIDGE` 单独判断或二次恢复方法体。缺失目标、歧义身份、未知继承关系、不可证明的返回类型关系或未呈现效果时 MUST 保守拒绝。正文、物理成员与证据状态 SHALL 遵守既有预算、取消和选择契约。

#### Scenario: Evidence selection does not alter class source
- **WHEN** 相同 class 分别请求 essential 与完整证据，且纯转发和继承需求均可证明
- **THEN** 两次完整类正文 SHALL 相同；完整证据 SHALL 指出原桥接成员、源级覆写和真实调用 BCI，MUST NOT 伪造源代码中不存在的显式桥接方法

#### Scenario: Projection cannot be committed
- **WHEN** 继承读取、证明遍历、来源或输出预算耗尽，或请求被取消
- **THEN** 系统 SHALL 按既有停止契约响应，MUST NOT 发布已删桥接声明却缺少相应覆写或投影证据的半成品完整类

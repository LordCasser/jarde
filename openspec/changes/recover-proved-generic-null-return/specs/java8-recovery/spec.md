## MODIFIED Requirements

### Requirement: Complete class source preserves proved method-local generic signatures

对于声明了可完整解释且与真实方法 descriptor 擦除一致的 Java 8 方法泛型 `Signature`，若已恢复方法体及受影响调用在该泛型声明下仍可证明为合法且保持原绑定，完整类源码 SHALL 在方法声明中保留该方法自己的类型参数、上界和参数/返回位置的类型变量。无参数实例方法的 `return null;` 可作为引用型方法自有 `T` 的值，仅当同轮完整 Code、AST 和 SSA 证明它无效果且精确直接返回 `null` 时才能投影。不能由字节码源码输出猜测泛型；停止或证据不足 MUST 保留物理声明和报告且不得发布半个泛型头。

#### Scenario: No-argument generic instance method returns null
- **WHEN** 顶级普通 `Object` 子类的公有实例方法具有精确签名 `<T extends Number> T value()`、擦除 descriptor `()Number`、无参数及无异常，完整正文精确为无效果的 `aconst_null; areturn`，且同类调用/重载绑定无未证明冲突
- **THEN** 完整类源码 SHALL 写 `public <T extends java.lang.Number> T value()` 与 `return null;`；外部 `result.<Integer>value()` 消费者 SHALL 与原/JADX 源码一起通过 Java 8 重编，`-Xverify:all` 运行值及反射类型变量名称、界和返回类型 SHALL 相同

#### Scenario: Similar-looking return has unproved effects or incompatible type
- **WHEN** 真实 Code 在 null 返回前有额外效果、异常 handler、非 null 生成、错误的 `Signature` 擦除、primitive 返回或未证的本类同名调用绑定
- **THEN** 系统 MUST 拒绝这个泛型声明投影，保留按物理 descriptor 拼写的声明和拒绝来源；MUST NOT 仅凭最终 `return null;` 文本或泛型 Signature 省略真实指令

#### Scenario: Proof stops before declaration publication
- **WHEN** 候选提取、Signature 读取/解析、来源或输出计费遇到预算耗尽或取消
- **THEN** 系统 SHALL 传播现有停止状态，MUST NOT 写出部分 `<T>` 声明；独立方法恢复及可取得的物理身份 SHALL 继续可查询

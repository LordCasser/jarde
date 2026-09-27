## MODIFIED Requirements

### Requirement: Complete class source preserves proved method-local generic signatures

对于可完整解析且与物理 descriptor 擦除一致的方法 `Signature`，完整类源码 SHALL 只在正文与声明参数于同次恢复中保持 Java 合法性、来源及调用绑定时写出泛型语法。对顶级普通类中的静态单参数空 `void` 方法，系统 MAY 从已证的 `java.util.List` 通配符 Signature 恢复 `?`、`? extends` 或 `? super`；必须证明该唯一参数未被正文读取/写入且真实 Code 精确无效果返回。证据不足、预算耗尽或取消 MUST 保留物理报告且不得发布半个参数化声明。

#### Scenario: Wildcard parameter is preserved on an empty static method
- **WHEN** 完整 Java 8 类的静态 `void` 方法只有一个 `List` 参数，Signature 的唯一实参是 `?`、`? extends Number`、`? super String`、`? extends byte[]` 或 `? super int[]`，其擦除为真实 `(Ljava/util/List;)V`，同轮正文精确为无效果 `return;` 且参数槽未被使用
- **THEN** 完整源码 SHALL 写出对应的 `java.util.List<...>` 参数；与原始/JADX 完整源码一起 Java 8 重编后，`-Xverify:all` 的反射参数类型 SHALL 逐方法相同

#### Scenario: Raw List control remains raw
- **WHEN** 邻接静态空方法的 `List` 参数没有泛型 `Signature`
- **THEN** 输出 SHALL 保持原始 `List`，MUST NOT 从其它方法的通配符类型推测泛型实参

#### Scenario: Parameter use, effect or ambiguous source refuses projection
- **WHEN** 该参数在正文中被读取/写入、Code 有额外指令/调用/异常路径、Signature 擦除不匹配、界无法拼写、成员来源歧义或同名调用目标未被证明
- **THEN** 系统 MUST 拒绝相应方法的通配符投影并保留按 descriptor 拼写的物理声明，不得仅因最终文本看似 `return;` 而丢失效果

#### Scenario: Proof stops before method header commit
- **WHEN** 参数/Code/SSA/Signature 证明或声明输出遇到预算耗尽或取消
- **THEN** 系统 SHALL 传播停止状态并保留可取得的物理成员事实，MUST NOT 发布部分 `List<...>` 声明

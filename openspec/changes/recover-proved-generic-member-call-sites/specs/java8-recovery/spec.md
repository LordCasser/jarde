## ADDED Requirements

### Requirement: Proven generic member construction preserves its enclosing instance and source argument binding

当非泛型外层类与其公开的泛型非静态成员类在选定环境中唯一确定，成员关系、合成外层首参、成员类型变量作用域以及构造器源级参数对物理参数尾部的对应关系均获证明时，Java 8 恢复结果 SHALL 把调用写为 `outer.new Inner<>(args)` 或等价的已证明泛型限定构造。源码括号中的实参 MUST 不含物理外层首参；完整二进制目标、来源与可观察的外层空值检查、普通实参、构造调用顺序 MUST 保持一致。缺少任何证明时 SHALL 保留可追源拒绝，不得仅凭 `$` 名称、泛型签名或首参类型推断绑定。

#### Scenario: Generic member call site can be recompiled against its original dependency

- **WHEN** Java 8 调用方以 `Object` 返回类型创建非泛型 `Outer` 的 `Inner<V>`，选定输入包含唯一一致的外层及成员 class，构造器泛型签名仅描述已证明的源级普通参数
- **THEN** 只重编该调用方、以原始外层及成员 class 为依赖时 SHALL 成功；其验证运行的返回值、副作用及异常次序 MUST 与原调用方 class 相同

#### Scenario: Missing or incompatible generic evidence is refused

- **WHEN** 成员类的变量声明、构造器签名与物理参数尾部擦除、成员关系、构造器重载绑定或调用点接收者身份中的任一项无法证明
- **THEN** 构造站点 SHALL 报告拒绝并保留物理 BCI 来源，且 MUST NOT 发布泛型限定构造或已证明完整源码的结论

#### Scenario: Null receiver precedes ordinary argument effects

- **WHEN** 外层限定值为空，普通构造实参包含可观察效果
- **THEN** 重编调用方 MUST 在这些效果之前抛出同类空值异常；非空路径 MUST 使每个普通实参效果恰好发生一次且次序与原 class 相同

#### Scenario: Evidence selection and request stops remain honest

- **WHEN** 同一调用点分别请求必要、全部或 BCI 范围证据，或目标读取及证明期间发生预算耗尽或取消
- **THEN** 完整请求的源码决定 SHALL 一致，报告 MUST 同时保留泛型调用的源码位置与物理构造来源；停止请求 MUST 表明未完成且不得发布无证明的成功结果

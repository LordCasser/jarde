## MODIFIED Requirements

### Requirement: Complete class source preserves proved named member class families

对完整选定的 Java 8 命名非静态成员类家族，系统 SHALL 在双向成员身份、外层泛型词法作用域、捕获、构造调用、方法 Signature 擦除和完整正文均可证明时，按已证 Java 类型路径一次输出嵌套声明。物理根与 child 的成员、来源和执行状态 MUST 保持可查；证据不足或停止时 MUST 保留物理表达，不得把另一个物理类的同名类型变量当作当前作用域。

#### Scenario: Outer type variable flows into one direct member
- **WHEN** `Outer<T>` 有唯一直接非静态 `Inner`，child 的 `(TT;)TT;` 方法用同一外层 T 直接返回参数，根 `make()` 的 `()LOuter<TT;>.Inner;` 与准确构造调用、捕获和双向成员关系闭合
- **THEN** 完整类源码 SHALL 写出 `Outer<T>` 内的 `Inner` 与 `T id(T)` 以及可编译的成员返回/构造表达；原始、JADX、Jarde 源码与 `Outer<String>.Inner` consumer SHALL 通过 Java 8 重编并在 `-Xverify:all` 下输出相同结果

#### Scenario: Scope or family identity is not proved
- **WHEN** 根/child 的选定物理关系缺失或冲突，根 T 作用域不完整、重复或与物理 descriptor 擦除不符，构造/捕获或正文有额外未解释使用，或另一个 child 使家族超出首片边界
- **THEN** 系统 MUST 拒绝泛型嵌套家族投影，保留两个物理 class 及其拒绝来源，不得只补成员名、泛型头或删除捕获字段

#### Scenario: Family proof stops
- **WHEN** 家族发现、Signature/Code/SSA 证明、source-map 或输出遇到预算耗尽或取消
- **THEN** 系统 MUST 传播停止并保留已读物理前缀，不得发布部分 `Outer<T>.Inner` 声明或混合 root/child 的方法身份

## MODIFIED Requirements

### Requirement: Complete class source preserves proved generic class headers

当普通 Java 8 类的完整 `Signature` 明示直接参数化 superclass，系统 SHALL 仅在物理父名、签名擦除以及选定父定义的形参数量与作用域全部一致时，写出准确的 `extends` 实参。证据不足、来源歧义或停止时 MUST 保留物理 raw 类头、拒绝及物理报告，不得从方法体或最终文本猜泛型父类。

#### Scenario: One direct Parent<String>
- **WHEN** 无自有形参的普通 `Child` 的唯一 class `Signature` 是 `Ldt21parent/Parent<Ljava/lang/String;>;`，物理父类指向选定的 `Parent`，其真实签名证明唯一 `<T extends Object>`
- **THEN** 完整源码 SHALL 写 `Child extends Parent<String>`；原始、JADX 与 Jarde 完整源码连同同一 Runner SHALL 通过 Java 8 重编和 `-Xverify:all`，`getGenericSuperclass()` SHALL 逐字一致

#### Scenario: Parent relation or signature is unproved
- **WHEN** 父定义缺失/歧义、无泛型形参或 arity 不符，child 签名/擦除错误，或出现本片不支持的接口、type-use 注解、嵌套/多段路径
- **THEN** 系统 MUST 拒绝参数化父类投影并保留原 raw 头，不得仅因 child 签名包含 `<String>` 就改变源码

#### Scenario: Proof stops before class header commit
- **WHEN** class Signature、选定父定义或输出遇到预算耗尽/取消
- **THEN** 系统 MUST 传播停止并保留已取得的物理事实，MUST NOT 发布半个参数化 `extends` 头

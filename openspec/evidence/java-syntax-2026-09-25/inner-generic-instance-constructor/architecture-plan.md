# 成员类构造与泛型声明的分层推进

本地 JADX 1.5.6 的结果依赖上下文，不能用一个“支持泛型内类”的标签概括：[双层泛型样例](analysis.md)的 `A<String>.B<Integer>` 调用丢掉封闭实例，完整源码不能重编；[非泛型外层、泛型成员样例](non-generic-outer-generic-member/analysis.md)的参数化返回调用能发出 `outer.new Inner<>(...)`，但同一成员类改为 `Object` 返回时又写成不能重编的 `new Outer.Inner(outer, ...)`。对 Jarde，已经独立验收的[非泛型成员调用](../../../changes/recover-proved-member-inner-construction/verification-root.md)只解决调用点和选定类事实的交接，不构成嵌套声明或泛型类作用域的证明。

| 阶段 | 具体 Java 8 形状 | 可复用的证据与落点 | 完成口径 |
| --- | --- | --- | --- |
| 已验收 | `Outer` + 非泛型 `Inner`，`outer.new Inner(args)` | `InnerClasses` 双向关系、合成外层字段/prologue；`new@1` 的 SSA 身份、早空值检查、参数依赖闭包；现有 `New` AST | 调用方对原始目标 class 重编，正常/null/效果轨迹与原 JVM 相同；错关系、错身份、迟检查拒绝 |
| 已验收 | 非泛型 `Outer` + `Inner<V>`，`outer.new Inner<>(args)` | 在同一成员证明后读取类/构造器 `Signature`，将源级参数与已证物理首参后的 descriptor 尾部对齐；现有 AST 加明确 diamond 拼写 | [recover-proved-generic-member-call-sites](../../../changes/recover-proved-generic-member-call-sites/verification-root.md)已对原始依赖完成 caller-only 重编及负例验收；不据此宣称完整类族输出 |
| 当前实施 | `Outer.A<T>` + 非泛型或泛型非静态成员，在 raw/参数化调用方创建 | [四形状矩阵](shape-matrix/analysis.md)把外层泛型、成员泛型和返回签名拆开；复用已有目标/SSA/null 证明，新增已选成员关系的源级类型路径 | [recover-generic-enclosing-member-call-sites](../../../changes/recover-generic-enclosing-member-call-sites/design.md)要求四个调用方分别对原始类族重编、执行一致；JADX 1.5.6 四个调用方均不能重编 |
| 后续类声明债务 | `Outer.A<T>` 与 `B<V>` 的两层声明及字段/方法 `T`、`V` | reader 已有 class/method/field `Signature` 树和擦除证明；`class_source::project_generic_signature` 只发布顶层作用域且拒绝 `$`，`source_text` 尚不装配嵌套声明 | 先证明每层 `InnerClasses` 关系、外层变量词法作用域和物理成员 erasure，再按所属外层一次装配完整声明；完整源码 Java 8 重编与反射签名对齐 |

JADX 的 `ClassModifier` 标记可跳过合成首参，`InsnGen.addOuterClassInstance` 以首参静态类型、非 `this` 和内部类标志决定是否发出限定实例。这给出了**源级参数与限定表达式的输出位置**，但其形状门没有证明调用点的两个 SSA 值是否相同，也不保证空值检查先于实参副作用。[已冻结的身份错配字节码](simple-member/invalid-controls/byte-variants/analysis.md)中，JADX 重编结果少执行一次实参效果，说明这一证明不可省。Jarde 应只在现有物理目标和 SSA 站点证明成立后复用其发射思路；不必新增通用语法 pass。

进一步优化的顺序是：先补真实输入的编译与执行闭环，再扩大一种独立可证的签名/声明组合。`-g:none` 的局部泛型变量原始拼写通常无从证明，目标是等价且可编译的 Java 源码，而不是逐字还原。匿名/局部类、构造器重载、复杂泛型界与多级捕获需要各自的反例和证明门，不能在这个调用点 change 中顺带放宽。

完整类族不能靠把子类文本拼进根类的 `text` 字段蒙混过关。当前 `ClassSourceReport` 的 `class`、`declaration`、`fields`、`methods`、`coverage` 和 `execution` 都只属于一个选定 class；`source_text` 也忽略已有的 `assembly_context`。若根类源码包含 `A<T>` 与 `B<V>`，报告必须逐一说明子类的物理定义身份、成员来源、预算消耗和停止状态。后续 OpenSpec 应先规定“以根类请求装配一个编译单元”以及子声明的报告结构，再实施受环境约束的类树；不能在现有单类报告仍声称完整时暗中追加子类。`T`、`V` 的绑定需按声明类身份组成词法作用域，重名/缺失绑定先拒绝，构造器只在已证捕获前缀下删去源级不可见首参。这是目前确需新装配结构的边界，而不是新的 Signature 解析器或通用反编译 pass。

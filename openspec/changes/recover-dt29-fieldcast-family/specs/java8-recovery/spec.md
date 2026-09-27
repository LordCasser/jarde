## ADDED Requirements

### Requirement: 已证明的跨接收者父字段与方法转换保持物理绑定

系统 SHALL 仅在选中类层级、目标声明、物理引用、实际接收者或实参及逐点来源唯一吻合时，为 `B` 值生成访问 `A` 字段或调用接收 `A` 的方法所需的 Java 8 引用转换。生成源码 MUST 写入物理目标字段、调用物理目标方法且每个效果执行一次。证明缺失时 MUST 保留带来源的拒绝，不得靠同名字段、重载或文本 cast 猜测绑定。

#### Scenario: C 与 D 的四字段 setter 和根类调用
- **WHEN** 固定 `FieldCast` 类族中的 `C.set`、`D.set` 通过 `B`/`T extends B` 接收者写入 `A` 声明的 public、protected、包可见和 private 字段，并由根类三次以 `B` 调用私有 `bits(A)`
- **THEN** Jarde 的完整源码 SHALL 保持四个 `A` 字段和私有目标调用的物理绑定；即使隔离负例另有 `B` 同名隐藏字段也不得误绑，相关方法没有 `@bytecode`，且字段写入、调用的物理 BCI 均可追溯

#### Scenario: 不唯一或不合法的转换
- **WHEN** 所选父类、目标字段/方法的 owner、name、descriptor 或 flags、SSA 接收者/实参与物理指令不符，跨包源码访问不合法，或另一个重载会改变源码目标
- **THEN** 系统 MUST 不发布相应转换及依赖它的完整方法；原始 BCI 与拒绝原因仍可查询

### Requirement: 受证有正文的泛型 void 方法保留声明元数据

当方法 `Signature` 的参数化声明与物理 descriptor 的擦除一致，且完整正文的同 run AST/SSA 证明参数槽不被写入、所有参数使用均能以相同物理擦除类型安全呈现时，系统 SHALL 在 Java 8 源码中恢复该泛型方法声明。系统 MUST 不因泛型外观重写物理 SSA 类型或丢失源级反射元数据。若 class bound 含 `$`，系统仅可复用已选物理 descriptor 提供的相同参数源码拼写；当前选中 class 自己的唯一 `InnerClasses` 记录须证明当前类与 bound 名称具有相同 outer 和合法的直系 inner 名，且相关源码名均为合法 Java 标识符。该属性给出 nesting 声明事实，不代替 bound 定义的独立读取；不得仅凭字符串替换推导 nested source name。预算或取消停止 MUST 原子地保留物理声明，不得发布半截泛型头。

#### Scenario: D 的有副作用泛型 setter
- **WHEN** `D.set` 具有 `<T extends B> void set(T, boolean)` 签名，擦除为 `(B,Z)V`，且同 run 正文证书覆盖四次已证明的父类字段写入和 accessor 调用
- **THEN** 完整源码 SHALL 可重编、运行，反射所得类型参数、上界、泛型参数及物理参数类型与原 class 一致

#### Scenario: 签名或使用不闭合
- **WHEN** 类型变量未绑定、上界与擦除不符、正文把参数用在不兼容位置、重载目标无法唯一确定，或预算/取消中断证明
- **THEN** 系统 MUST 不发布看似完整但元数据或绑定错误的泛型声明，并保留可检查的拒绝或停止状态

### Requirement: 跨块条件拼接保持每次求值和结果顺序

当四段条件字符串沿唯一、无别名的构造链按次序追加时，系统 SHALL 将各条件、字段读取和最终字符串作为一个语义完整的 Java 8 方法恢复。每个条件及其值 MUST 在原有执行位置求值一次，不能吞掉调用、异常或其它可观察效果。

#### Scenario: bits 的四位字符串
- **WHEN** 固定 `bits(A)` 从 `A` 的四个 boolean 字段得到四个条件值并按顺序拼接
- **THEN** Jarde 完整源码 SHALL 可重编，原 class、固定 JADX、Jarde 对 `1111`、`0000`、混合位型产生相同字符串，四次读取和最终返回均有来源

#### Scenario: 构造链或条件值不唯一
- **WHEN** 构造器结果有别名、条件汇合值复用/交换、追加重载变化、独立效果或异常边进入链，或证明所需预算耗尽
- **THEN** 系统 MUST 保持拒绝或停止，不得输出改变求值次数或顺序的拼接

### Requirement: 固定 FieldCast 完整类族经整体语义验收

系统 SHALL 以固定 `TestFieldCast` 导出的全部物理类及所需接口为单一恢复对象验收，不以单个方法能打印代替类族可编译与运行。

#### Scenario: 原始、JADX、Jarde 的 Java 8 对照
- **WHEN** 固定源码以 `javac --release 8 -g:none` 编译，并分别从原 class、固定 JADX 和 Jarde 的完整源码重编后运行相同 Runner
- **THEN** 三方 `java -Xverify:all` 的标准输出 MUST 精确为 `runnable:1111:0000:1111:ClassCastException`，Jarde 对本类族相关方法无 `@bytecode` 或缺失返回，`D.set` 泛型反射信息与原 class 一致

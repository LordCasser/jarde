## ADDED Requirements

### Requirement: Structured parameterized List bodies are projected as one source decision

对于完整、无异常处理器、参数局部未重赋值的结构化void方法，若方法Signature给出确切 `java.util.List<S>`，且所有这些参数的真实源级用途均被有限平台契约证明，系统 MUST 同时恢复参数化声明和合法调用实参。系统 MUST 支持固定CF11的嵌套迭代、条件、元素add、对象remove、size及clear，以及至少一种不同元素类型的同类正文。不能仅因正文含多个块、循环Phi而退化为raw参数，也 MUST NOT 单凭Signature擦除相同或参数未重赋值发布未证明的泛型声明。

投影 MUST 保持物理检查转换、对象/索引重载选择、循环条件和副作用执行位置、值/异常及可观察集合状态。仅已核准的源级擦除参数包装可去除，未证明来源、参数重赋值、raw别名escape、wildcard capture、不完整结构、handler或未支持平台调用 MUST 沿既有泛型refusal保持可靠原正文，不能全局删cast或猜测未知层级。声明和正文 MUST 在同次有界投影中原子发布；中止不发布部分泛型源码或来源。optional evidence选择 MUST 不影响正文决定。

#### Scenario: Nested String lists retain their generic declaration and behavior

- **WHEN** 固定CF11方法迭代两个List<String>并有String检查转换、add、remove、size和clear，正文及所有参数用途证明完整
- **THEN** 未手修完整生成类 MUST 保留List<String>参数、add实参可赋值，双JDK编译运行的8条值/异常/集合状态同原类，真实String检查转换和remove(Object)绑定不改变

#### Scenario: The rule follows element types instead of fixture names

- **WHEN** 不同类名、参数名、至少一种非String元素的同类合法嵌套迭代/添加正文满足相同证明
- **THEN** 系统 MUST 恢复它自己的确切List<S>声明及合法调用，不能固定成String或依据名称匹配

#### Scenario: Real casts and overloads cannot borrow presentation elision

- **WHEN** 字节码含真实checkcast或对象remove与索引remove近邻调用
- **THEN** 系统 MUST 保留物理检查转换及准确调用目标；当前投影未证明的cast MUST 不被删去，完整控制行为同原

#### Scenario: Unsupported uses refuse generic projection precisely

- **WHEN** List<S>参数重赋值、逃逸至raw别名、wildcard写、元素实参不兼容、未知接收者层级、handler或结构/访问证据不完整
- **THEN** 系统 MUST 记录准确泛型投影拒绝并保持原可靠正文；MUST NOT 错称泛型已恢复，也不能借拒绝一个控制关闭整个语法单元

#### Scenario: Request selection and stop preserve atomicity

- **WHEN** default、all或不含RuleDetails的请求运行，或者候选/用途/AST编辑/重发射中预算耗尽或取消
- **THEN** 完整请求 MUST 得到同正文及物理来源；中止 MUST 发布既有Stopped，无部分声明、正文或来源，新增工作受原预算维度限制

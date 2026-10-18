## ADDED Requirements

### Requirement: Proved indexed List bodies preserve generic parameter declarations

对已完整恢复、结果仍为原物理 void 或精确 primitive 的方法，若 List 参数的全部用途与其合法 Signature、元素转换及源级调用均已证明一致，系统 SHALL 在完整方法及类源码中恢复相应 `List<E>` 参数声明。系统 MUST 保持原方法选择、返回类型、读取次数、求值顺序和异常时序；MUST NOT 仅凭 Signature 或一个结果转换猜测安全的泛型正文。

#### Scenario: Fixed indexed String loop retains its generic signature

- **WHEN** 固定 TestNestedLoops2 的 List<String>、size/get 调用、String 元素转换和 boolean 方法体均已证明
- **THEN** 完整方法和类源码 SHALL 输出 `java.util.List<java.lang.String>` 参数，保留已恢复的外层索引循环、内层循环及返回表达式；完整源码重编译后的正常和异常结果 SHALL 与原始 class 一致

#### Scenario: Physical get conversion remains observable

- **WHEN** 已证明的 get(int) 结果通过真实元素 CHECKCAST 被使用，输入含 null、错误元素、越界或会抛错的 List 实现
- **THEN** 系统 SHALL 保留该真实转换及其原有执行位置，完整生成源码的 size/get 事件、返回值和异常 SHALL 与原始 class 一致；MUST NOT 把该转换当展示包装删除

#### Scenario: Primitive result remains exact

- **WHEN** 泛型参数声明被恢复且方法结果为原物理 boolean、int、long 或其它 primitive
- **THEN** 输出 SHALL 保留同一精确结果类型及正文；MUST NOT 因若干类型共享 JVM return opcode 而互换其源类型，也 MUST NOT 据此接纳未证明的引用、数组或类型变量结果

#### Scenario: Missing body or element proof keeps reliable erased output

- **WHEN** 缺少唯一的参数用途、get/转换关联或源节点证据，存在额外写入、别名逃逸、未支持的成员/签名、错元素转换或无法闭合的值合流
- **THEN** 系统 SHALL 保留原先可靠的循环、正文、物理转换和擦除声明，并准确报告泛型投影未证明；MUST NOT 发布部分改写或凭同名/相邻位置匹配其它值

#### Scenario: Evidence selection and execution stops preserve output contracts

- **WHEN** 同一输入以 default/all/essential 请求完整源码，或证明和提交遇到预算、取消、期限停止
- **THEN** 成功请求的代码决定 SHALL 一致，raw 方法文本及对应 source map SHALL 保持同一恢复平面的真实性；真实 Stop MUST 原样传播，不能转为普通缺证后发布半完成声明

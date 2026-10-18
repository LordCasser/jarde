## ADDED Requirements

### Requirement: Proved List body calls omit redundant Object presentation wrappers

在已有可靠结构化 List 方法中，对精确 `String.equals(Object)` 和 `List.remove(Object)` 调用，若实参为物理元素转换、局部存取和来源均闭合的引用局部值，且去掉 Object 展示包装后源级绑定仍与原调用一致，系统 SHALL 在完整方法及类源码中省略该包装。实参求值次数、顺序、异常和方法选择 MUST 保持一致。remove 的实参类型 MUST NOT 用作 List 元素类型推断依据。

#### Scenario: Fixed nested String loops match their source calls

- **WHEN** 固定 JADX TestNestedLoops 的两层 String foreach、局部绑定和调用均已证明
- **THEN** 完整方法及类源码 SHALL 输出 `s1.equals(s2)` 和 `l1.remove(s2)`，继续保留 `l2.add(s1)`、两层 typed foreach 和原名；完整源码重编译后的运行结果 SHALL 与原始 class 一致

#### Scenario: Reference Integer removal keeps the Object overload

- **WHEN** 同范围内已证明的局部值类型为 Integer，物理调用为 List.remove(Object)，且已有其它事实独立支持泛型声明
- **THEN** 系统 SHALL 允许省略 Object 展示包装并保持 remove(Object) 选择；MUST NOT 将该引用实参误当 primitive int，也 MUST NOT 据此推断 List<Integer>

#### Scenario: Unknown optional proof preserves reliable recovery

- **WHEN** 可选清理缺少唯一声明、赋值、物理值链、精确调用或唯一 AST 包装证据
- **THEN** 系统 SHALL 保留该包装及已有可靠泛型、typed foreach 和原名；MUST NOT 因可选清理缺证让完整方法退回原始擦除形式

#### Scenario: Overloads and physical conversions remain observable

- **WHEN** 调用接收者具有 equals(String)/equals(Object) 重载，实参为需要保持选择的 primitive int 装箱表达式，或实参含真实 CHECKCAST
- **THEN** 系统 MUST 保留原重载和物理转换语义；真实 CHECKCAST 的错误类型异常和 null 行为 SHALL 与原 class 一致；MUST NOT 用通用 Object 包装删除规则处理这些表达式

#### Scenario: Evidence planes and stops remain intact

- **WHEN** 同一输入通过 default/all/essential evidence 请求输出，或证明及投影发生预算、期限或取消停止
- **THEN** 成功的完整源码 SHALL 一致，raw RecoveryReport 文本、原始 source map 及物理事实 SHALL 保持不变；Stop SHALL 沿已有契约传播，MUST NOT 发布半完成方法或部分投影

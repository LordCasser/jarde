## Why

主线 `790579e2` 已证明泛型字段的写入源类型，但 `Hold<T>(T)` 的构造参数仍呈现为 Object，字段因此安全擦除；完整类可编译，构造参数和字段的泛型反射仍丢失。根因是现有构造器 Signature 分流要求方法级形式参数，正文候选仅支持空体或原参数 super 转发，未消费类作用域参数和初始化后的直接字段写。

## What Changes

- 对已发布类泛型作用域中的构造参数恢复 Signature，不为类变量发明构造器形式类型参数。
- 扩展既有构造器候选，证明 Object() 初始化后按原序执行的 `this.field = 原参数` 直线正文；覆盖引用、数组、bound、多类变量与 long/double 槽前缀。
- 构造器只依赖其完整正文、物理参数/字段和已发布类作用域，字段继续依赖实际发布的构造器参数，保持单向发布顺序。不同 binder 不因同一擦除被合并。
- 保持已有方法级泛型空体/转发构造器和成员/匿名类消费者的证明范围，未证明正文、参数改写、非 Object 父类、this 委派、异常处理及预算/取消可靠拒绝。
- 前置条件：完整 Java 正文、单次 AST/Code/SSA、InitRecord、现有 Signature 解析与擦除证明、类作用域以及同类物理成员表可用。非目标：任意构造器正文的泛型推断、补源级 cast、raw receiver 实例化、嵌套/枚举泛型构造器、throws/注解投影，以及整个泛型单元的完成。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 恢复已证明的类作用域构造参数，并约束构造器与字段的泛型发布依赖。

## Impact

`crates/jarde-java/src/report.rs` 的 GenericConstructorCandidate 提取、`src/class_source.rs` 的构造器声明分流，以及 `src/facade.rs` 的物理字段传递/既有候选消费者。复用当前 SSA、InitRecord、参数槽、Signature parser 和字段写证明；不新增 pass、IR、fixpoint 或依赖。双真实 JDK8/JDK23、debug/no-debug 三方对照及完整类重编、验证执行、作用域身份反射作为验收。

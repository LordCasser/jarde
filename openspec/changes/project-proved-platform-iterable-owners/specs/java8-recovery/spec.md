## ADDED Requirements

### Requirement: Proven Java 8 platform collection loops use enhanced for

对源类型可确定为 Java 8 平台 `java.util.List` 或 `java.util.Collection`、且迭代行为与 Java 8 增强 `for` 完全等价的循环，系统 SHALL 呈现增强 `for`；系统 MUST 保持容器求值次数、元素转换、循环转移、异常处理及来源。仅有同名 `iterator()` 或缺少源类型/执行证明时，系统 MUST NOT 输出增强 `for`，而 SHALL 保留已可用的普通循环或可定位的保守引用。

#### Scenario: List and Collection with and without debug information

- **WHEN** Java 8 class 的 `List` 或 `Collection` 参数产生自身接口 owner 的 `iterator()` 调用，iterator 仅由匹配的 `hasNext()`/`next()` 消费，取值是每轮首个可观察动作，而且相关调用的异常处理范围一致
- **THEN** 无论 class 有无调试局部变量表，完整类 SHALL 输出可用 `javac --release 8` 重编的增强 `for`，并在空、多元素、null 与异常路径上保持原 class 的值、调用次数和异常顺序

#### Scenario: Raw type keeps explicit element cast

- **WHEN** `List` 或 `Collection` 只能以原始源码类型呈现，原循环在 `next()` 后以显式 cast 取得较窄元素类型
- **THEN** 增强 `for` SHALL 使用可编译的 `Object` 元素绑定，且 SHALL 在循环体原位置保留 cast；坏元素的 `ClassCastException` 与其前后副作用次序 SHALL 与原 class 相同

#### Scenario: Other owners and extra consumers

- **WHEN** `iterator()` 的 owner 是用户自定义子接口、仅有同名方法的非 `Iterable` 类型，或 iterator/元素另有消费者、逃逸、前置效果或不一致的异常处理范围
- **THEN** 系统 MUST NOT 据本平台规则把循环输出为增强 `for`；已恢复的普通循环或保守引用 SHALL 保留，且停止、来源及默认/完整证据正文契约 SHALL 不变

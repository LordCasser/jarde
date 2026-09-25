## Why

直接 `Iterable.iterator()` 的增强 `for` 已经独立验收，但 Java 8 对 `List`、`Collection` 类型的合法循环会把调用 owner 分别写成 `java/util/List`、`java/util/Collection`，当前精确 owner 门槛因此保留 `while`。JADX 1.5.6 只在有调试表时恢复其中两种语法；本变更用已知平台类型事实追平这部分正确输出，同时维持现有执行与异常证明。

## What Changes

- 仅为 Java 8 平台接口 `java/util/List`、`java/util/Collection` 的精确 `iterator()Ljava/util/Iterator;` 调用增加增强 `for` 候选，要求接收者的已呈现源码类型与该 owner 一致，并复用已验收的 iterator 独占消费、首动作、cast、handler、来源和原子提交证明。
- 原始或无法证明泛型的接收者仍以 `Object` 元素绑定并在循环体保留原 `checkcast`；不为了得到 `String` 元素名修改推断类型或删除转换。
- 固定有／无调试表的 `List`、`Collection`、自定义子接口、同名非 `Iterable` 的三方完整类对照，并覆盖空/null、坏元素、异常、额外消费和预算停止。
- 不新增通用子类型求解器；自定义子接口需要来自请求环境的类层级定义，跨异常区 explanation-only 和泛型签名呈现也另行处理。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：将已证明属于 Java 8 平台 `Iterable` 子接口的 `List`、`Collection` 加入增强 `for` 准入，保持行为等价和保守拒绝。

## Impact

局限于 `jarde-java` 的现有 `iterable_for_each_candidate`、相邻集成测试及 OpenSpec 证据。不改变 classfile 解析、SSA、Region、对外 API、CLI 或依赖；直接 `Iterable` 子切片和数组子切片须保持原验收结果。依据为[接口 owner 冻结对照](../../evidence/java-syntax-2026-09-24/iterable-subtype-owners/analysis.md)及[直接 `Iterable` 独立验收](../project-proved-enhanced-for-loops/verification-root-iterable.md)。

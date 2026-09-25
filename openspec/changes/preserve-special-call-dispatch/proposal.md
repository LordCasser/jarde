## Why

自写 `SpecialProbe` 的三方执行对照确认：jarde 将 `super.value()+1` 以及 `DefaultProbe.super.value()` 都写成 `this.value()`，重编译后触发 StackOverflowError，原 class 分别返回 8 和 11。jadx 1.5.6 对接口调用也误写成 class-super、返回 7，因此这项以原 class 行为为验收依据。

## What Changes

- 非构造器特殊调用保留 class-super、interface-super 和本类 private 三种不同的选择语义。
- 由当前类同次读取的直接父类/接口事实证明 super 目标；无法证明时明确引用，不默认为虚调用。
- 本类 private 调用保留其实际接收者，包括另一个同类对象。
- 保持现有 `super()`/`this()` 构造调用与普通 virtual/interface/static 调用。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增特殊调用目标和实际接收者的可观察正确性要求。

## Impact

前置是现有 `MethodIr` 已持有同次读取的 `Arc<ClassFacts>`、SSA 和 `ClassMembers`。实现限于借用已有类头事实、调用目标的池项种类、Java 接收者表示与消费构造，不新增 classpath 展开、callee Body 请求、resolver、pass、依赖或预算种类。

本 change 替代 `present-proved-java-structure` 4.3 中不完整的“属主不同就写 super”设计，并明确同类 private 调用可接受其它实例接收者。非目标：任意祖先查找、嵌套类 `Outer.super`、别名接收者证明、bridge/overload 还原、非法 class 的完整校验。超出当前声明证据的特殊调用保留缺口，不伪装成正确 Java。

## Why

Java 8 的 `Outer.super.method()` 在命名成员类中可经 Outer 的 synthetic 静态桥和桥内 `invokespecial` 编码。当前 Jarde 保留桥调用；直接把它内联成普通 `Outer.method()` 会改变分派。[分派消歧样本](../../evidence/java-syntax-2026-09-26/named-member-outer-receiver/variants/analysis.md)的四段结果使这种错误可执行地显现。

## What Changes

- 在已证明、可装配的命名成员类家族上，识别准确的 Outer 直接父类方法桥及其全部可见使用点，仅对接收者为该成员捕获的词法 Outer 的调用投影 `Outer.super.method()`。
- 只有桥 body、接收者、目标分派、求值/异常顺序和完整引用闭包都成立时，才从家族**源码文本**省略物理 helper；物理方法、调用点与来源仍保留在报告中。
- 对相同类型的显式 `other`、普通 `this.method()`、不同父类目标、额外调用者或不透明 method-handle/bootstrap 使用保持原语义并拒绝猜测。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在已证类家族中呈现词法外层的限定 `super` 调用，并在证明不足时保留桥及明确拒绝。
- `source-maps`：被源码投影掉的方法桥及其全部调用点保留双方完整物理身份、BCI 和派生关系。

## Impact

前置条件是 [assemble-proved-member-class-family](../assemble-proved-member-class-family/tasks.md) 的家族身份、捕获值、嵌套 writer 和来源报告完成验收。本变更只补跨类的 `access$` 方法桥证明及家族源码投影；不创建通用内联 pass，不处理 `Outer.this` 字段 getter、local/anonymous、任意继承链 `super` 或没有完整输入闭包的项目级隐藏。JADX 的 `ClassModifier`/inline visitor/`InsnGen.callSuper` 是算法对照，不进入生产依赖。

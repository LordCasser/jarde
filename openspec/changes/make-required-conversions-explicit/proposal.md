## Why

恢复文本今天靠**打印上下文**隐式完成一部分类型转换：同一个表达式在不同位置写出不同宽度，而值语义只在部分位置被保住。已登记的反例（T5）：

```java
// 原文：sb.append((int) c) 形式，c 为 char
// 当 c == 'A' 时：原方法返回 "65!"，恢复文本返回 "A!"；两者都能编译
```

`crates/jarde-java/src/build.rs::concat_expr` 已经保存了 append 的参数类型，但只对 boolean 转换做了适配。`javac` 通过、四个报告平面全绿，都不能证明值正确——两段文本都能编译而结果不同。

本变更独立登记（不混入 `add-demand-driven-core-results`），只负责"表达式实际呈现类型与消费位置要求类型不一致时，必须生成显式转换节点"。

## What Changes

- 表达式的类型决定 MUST 在**构建**时做出（结合消费位置要求），而不是交给打印器按上下文隐式决定；需要窄化/加宽时 MUST 生成显式转换节点（例如 `(int) arg0`），打印器只保留该决定。
- 先修现有 concat 接受范围（append 参数类型已保存），再逐项复用到调用实参、返回与赋值；**不同时**引入完整子类型系统。
- 无法用现有证据证明转换合法时，按既有 refusal 契约拒绝该区域并保留 bytecode 与 origin，MUST NOT 发布 javac 拒绝或值不同的文本。
- 验收必须用受控编译执行对照，不以编译通过或平面状态代替。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增"消费位置要求的类型必须由表达式自身满足，必要时显式转换"的可观察要求与场景；既有布尔上下文、拼接片段转换与拒绝契约继续成立。

## Impact

实施涉及 `jarde-java` 的表达式构建与打印（`build.rs` 的 concat 路径、调用/返回/赋值位置）、以及 `jarde-jvm` 在需要时提供的呈现类型事实。不新增依赖、不改报告 schema、不引入通用类型系统。

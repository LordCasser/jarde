## Context

一个描述符同时被三件事消费：读取路径判结构、JVM 层算槽位、Java 层拼写。今天这三件事各有自己的实现，于是"数组占几个槽"这类事实在每个实现里各错一次：

| 位置 | 现状 | 后果 |
| --- | --- | --- |
| `src/class_source.rs::descriptor_type` | 数组沿用元素槽宽 | `long[]` 参数被当成占两个槽，签名与正文错位 |
| frame / lambda 的槽位推导 | 各自实现 | 同一事实多处维护，修正无法一次生效 |
| `jarde-java` 的拼写 | 部分自行解析描述符 | 拼写与事实可能分叉 |

JVMS 2.6.1 / 4.3.2 给的是确定规则：基本类型按宽度（`long`/`double` 两槽，其余一槽），引用（数组与类）一律一槽，`void` 只在返回位置合法。

## Goals / Non-Goals

**Goals:** 描述符事实只有一份、由读取层提供；槽位由 JVM 规则计算；Java 层只做拼写；上述反例在文本与受控执行两层都被钉住。

**Non-Goals:** 不改报告 schema、不引入类型推断命名、不引入泛型/签名属性解析、不改变拒绝与 fallback 契约。

## Decisions

### 1. 事实的形状与所有者

`jarde-reader` 提供一次解析的描述符事实：基本类型（含 `void` 的合法位置）、原始类名、数组维数、参数序号，以及由 JVM 规则计算的**槽占用**。形状示例：`DescriptorFacts { kind, class_name, dimensions, parameter_index, slots }`。所有消费者（frame、lambda、声明槽位、class-source 拼写）都从它取值；读取层是唯一解析者。

### 2. 槽位计算留在 JVM 层，拼写留在 Java 层

事实本身不携带"Java 拼写"，只携带类型身份与宽度；`jarde-jvm` 用它算 slot；`jarde-java` 用同一事实拼 `long[]`、`double[][]` 这类源型（数组维度与基本类型名都是事实的投影，不是重新解析）。这样"槽位对但拼写错"与"拼写对但槽位错"都不可能各自悄悄发生。

### 3. 反例先于重构入场

先加两个失败用例（`f(long[], int)`、实例方法 `h(long[], int)`），再替换实现，确保重构不是"看起来更整齐"而已。行为级验收复用既有受控 JDK 编译执行 harness。

## Risks / Trade-offs

- 替换分叉实现会触碰 frame/lambda 的既有断言 → 逐条核对"本次有意改变"与回归，不整体重录期望值。
- 数组槽位修好后，类内变量编号会整体左移一位（这正是缺陷的表现），既有 golden 中受影响的文本要按可逆规则确认。

## Migration Plan

按 tasks 分两步提交：先加反例与统一事实（读者层），再把 JVM/Java 消费点切过来并修断言；每步都保持 workspace 全绿。

## Open Questions

无。剩余分叉点的精确清单由实施首步的失败用例枚举得出。

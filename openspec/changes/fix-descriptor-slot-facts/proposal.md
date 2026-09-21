## Why

描述符的槽占用在仓库里有多份各自实现的解析，其中"数组沿用元素槽宽"是错的：数组本身只占一个槽（JVMS 2.6.1 / 4.3.2）。后果不是拼写问题，而是**签名与正文互相矛盾**的可复现错误：

```java
// javac --release 8 -g:none；源码 static int f(long[] xs, int n) { return n; }
static int f(long[] arg0, int arg2) {   // 签名把第二个参数拼成 arg2
    return arg1;                         // 正文读的是 arg1
}
static long g(double[][] arg0, long arg2) { return arg1; }
int h(long[] arg1, int arg3) { return arg2; }   // 实例方法同样错位
```

`long[]`、`double[]`、`long[][]` 都错，`int[]` 正常（与元素同宽时错误不可见）。独立复核（`9a2f4ce`）与主 Agent 的本机复现一致，锚点 `src/class_source.rs::descriptor_type`。

本变更独立于 `add-demand-driven-core-results`（后者负责普通 prepared 交接、可选证据与增量查询）与 `add-parallel-bulk-recovery`（后者负责 worker/背压/总账），只负责"描述符事实只有一份、槽位由 JVM 规则计算"。

## What Changes

- **一行事实一次解析**：`jarde-reader` 提供描述符事实（基本类型、原始类名、数组维数、参数位置、槽占用），读取路径共用同一实现。
- **槽位由 JVM 层计算**，Java 层只负责源码拼写；数组占一个槽，`long`/`double` 占两个。
- **替换分叉实现**：frame、lambda、class-source 等今天各自按描述符推槽位或拼写的位置改到同一事实，不留两套。
- 既有语义不变：`MethodDeclaration`/frame 的公开语义、prepared 与直接路径等价、receiver 拼写（`ACC_STATIC` 判定，实例方法写 `this`）、无调试信息时的序数命名。
- 独立登记：本变更只修这一条；append 参数转换（T5）等其它既有边界各自立项。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `classfile-inspection`：新增"描述符事实由读取层一次解析并供所有消费者使用"的可观察要求（数组槽占用、参数位置、拼写与事实分离）。
- `java8-recovery`：明确源码拼写只消费事实、不得自行推算槽位；实例 receiver 与参数命名规则继续成立。

## Impact

实施涉及 `jarde-reader` 的描述符解析、`jarde-jvm` 的 frame/lambda/声明槽位计算、`jarde-java` 与根 facade 的拼写消费点，以及相关测试与受控 JDK 编译执行对照。不新增依赖、不改报告 schema、不引入类型推断命名。

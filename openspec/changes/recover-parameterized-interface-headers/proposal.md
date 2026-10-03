## Why

[桥准入判据实证](../../evidence/java-syntax-2026-10-04/bridge-method-patrol/README.md) 与 root 2026-10-04 的 javac 对照确立一条**跨切面不变量**：隐藏 javac 合成成员（桥、accessor、lambda 伴生）的前提是**源码自身能让 javac 重新生成同一合成物**；而擦除桥的可重建性来自类头的类型参数投影，不来自平台事实。实测：

```java
// 裸 implements + 隐藏桥 → 硬编译错误
public class RawI implements java.lang.Comparable { public int compareTo(RawI o) { return 0; } }
// javac: 错误: RawI不是抽象的, 并且未覆盖Comparable中的抽象方法compareTo(Object)

// 参数化 implements + 隐藏桥 → javac 自行重建桥（javap 实测 ACC_BRIDGE 计数=1）
public class ParamI implements java.lang.Comparable<ParamI> { public int compareTo(ParamI o) { return 0; } }
// javac exit=0
```

主线对 `implements Comparable<Impl>` 的类只投影裸类型（`class BR$Impl … implements java.lang.Comparable`），因为 `recover-proved-direct-parameterized-superclass`（6/6 全验收）的边界只覆盖**父类** `extends Parent<T>`，未覆盖**接口**。后果：`Comparable<T>` 实现、`Iterable<T>`/`Iterator<T>`、`Function<…>` 等泛型接口实现类既丢契约信息，又使桥投影无法安全进行。

## What Changes

- 类头 `implements` 子句按类自身 `Signature` 属性投影类型实参（`implements Comparable<Impl>`），判据与机制复用 `recover-proved-direct-parameterized-superclass` 的既有通道（同一选定环境、擦除与物理 `interfaces` 一致、唯一选定、不嵌套）——**边界从父类扩到接口列表**，不新建第二套 Signature 解析。
- 接口定义不可解析（无 JRE image、非选定环境、arity 不符、擦除不一致）时保留裸类型与物理来源，**并拒绝该类的桥投影**（见下）。
- **确立消隐前置不变量**：桥成员投影仅在"该擦除契约所属父类型在类头文本中已带类型实参"时进行；类头留在裸类型时 SHALL 保持桥可见（现行为），不得产出"桥已隐藏但类头裸类型"的中间态（javac 会报 missing-override）。该前置在 `recover-bridge-admission-gates` 内实现，本片提供其依赖的类头投影。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：泛型接口在类头按 Signature 投影类型实参；擦除桥的隐藏获得可重建性前提。

## Impact

`src/facade.rs`（类头 Signature 投影处，与 `recover-proved-direct-parameterized-superclass` 同段）与 `src/class_source.rs`；`crates/jarde-java` 无需改动。既有父类参数化投影、成员声明参数化（`recover-ordinary-parameterized-signatures` 9/9）、裸类型回退路径零回退。

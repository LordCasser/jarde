## Why

[泛型静态字段初始化巡查](../../evidence/java-syntax-2026-10-05/generic-static-field-init-patrol/README.md)实证（呈现缺陷级——比响亮拒绝更严重的一类）：静态泛型字段的初始化表达式被渲染为**不可编译的损坏文本**：

```java
static Hold<String> f1 = new Hold<>("a");   // 源
static Hold f1 = new MN$Holdava.lang.Object) "a");   // jarde 渲染（逐字）
```

丢左括号、丢类型实参、类名与池形和 `java.lang.Object` 纠缠（嵌套形出现 `Hold.lang.Object` 片段）。渲染源集 `javac` **exit 1**（"需要 '(' 或 '['"）。**jadx 完整恢复同形**（`static MN.Hold<java.lang.String> f1 = new MN.Hold<>("a")`）——有解，非语言层限制。

**判别（已实测）**：菱形与显式类型实参**同样坏**（与 diamond 无关）；仅**静态**字段受影响（实例字段退化为构造器赋值，保守合法）；伴生 `Hold<T>` 自身的 `T v` 投影正常。触发链 = 静态字段 + 泛型 Signature 投影被拒（注释 `field_generic_body_unproved`，常量在 `src/class_source.rs` 约 5891 行一带）+ 初始化含泛型类构造调用。

## What Changes

**MVP（本片）**：把损坏文本修为**合法呈现**——两条路按取证选一（实现者以现有代码结构定，均无需新机制）：
- 路径 A：初始化表达式呈现为**裸类型正确形**——`static Hold f1 = new Hold((java.lang.Object) "a");`（构造器按擦除描述符呈现，与实例字段 f3 的既有退化路径同构）；
- 路径 B：若静态字段初始化的证明链可复用 f3 的构造器赋值通道（把 `f1` 的初始化并入 `<clinit>` 呈现），则与 f3 完全同构。

**判据要求**：无论哪条路，**不得再输出损坏文本**；呈现必须可编译或响亮引注（核心不变量：从不产出可编译但行为不同的文本——现状连"可编译"都不满足且非引注形）。

## Impact

- **代码**：`src/class_source.rs` 的字段初始化呈现路径（`field_generic_body_unproved` 拒绝后的回退分支）。**不触碰** `crates/jarde-java`。
- **测试**：`MN` fixture（巡查已冻结，真 javac23 `--release 8` 与真 javac 8 双腿各一份入 `tests/fixtures/`）+ 渲染断言（无损坏文本、`javac --release 8` exit 0 或引注）+ 行为对照（`a b 5` 输出一致）。
- **账本**：summary.md 的 2026-10-05 呈现缺陷行关闭。

## Non-Goals

- **不**做静态字段泛型 Signature 投影（那是后续独立片——本片只修损坏文本）；
- **不**改实例字段退化路径（f3 现状合法）；
- **不**处理方法级同类纠缠（RG 探针的 `pick`/`newInner` 注释是既有拒绝域，非本片）。

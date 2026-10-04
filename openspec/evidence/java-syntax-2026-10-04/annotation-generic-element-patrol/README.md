# 注解泛型元素巡查（2026-10-04，root）——低危保真度缺口 + 一处保护性门的架构警示

按 Goal "持续巡查各个反编译语法分析模块，构造各种 Java 语法场景，编译，然后对比源码、jadx、jarde" 巡查**注解元素带泛型返回类型**（DT-22 "注解类型声明及元素默认值" 域内此前未验证的子形）。主线 HEAD 二进制（`b595bc6b`）。

**结论：不立 spec（低危保真度缺口，且队列已满）。但记录一处重要的架构警示——现有拒绝门是 *保护性* 的，naive 放宽会造成比现状更糟的静默语义回归。**

固定转录见 [fixture](fixture/) 与 [results](results/)。

## 判别（单变量：泛型元素**有无默认值**）

[fixture/A2.java](fixture/A2.java)（全部**无**默认值）与 [fixture/A3.java](fixture/A3.java)（四形对照）实测：

| 形 | 源码 | jarde 呈现 | 判定 |
| --- | --- | --- | --- |
| 泛型元素无默认 | `Class<?> noDefault();` | `public abstract java.lang.Class<?> noDefault();` | ✓ 正确 |
| 通配符上界无默认 | `Class<? extends Number> bound();` | `Class<? extends java.lang.Number> bound()` | ✓ 正确 |
| 具体实参无默认 | `Class<String> exact();` | `Class<java.lang.String> exact()` | ✓ 正确 |
| **泛型 + 默认值** | `Class<?> withDefault() default Object.class;` | **`java.lang.Class withDefault() default java.lang.Object.class;`**（丢 `<?>`） | ✗ 保真度损失 |
| **通配符上界 + 默认** | `Class<? extends Number> boundDefault() default Integer.class;` | `java.lang.Class boundDefault() default java.lang.Integer.class;` | ✗ 同上 |
| **泛型数组 + 默认** | `Class<String>[] arrayDefault() default {};` | `java.lang.Class[] arrayDefault() default {};` | ✗ 同上 |

判别变量是 **`AnnotationDefault` 属性的存在**，与通配符种类、是否数组、默认值形态均无关（A2 三形全对、A3 三个带默认的全丢泛型）。这也解释了先前注解域巡查中 `A9$Meta` 的 `Class<?> k() default Object.class;` 被拒。

## 严重度实测（关键：仅反射保真度，无行为差）

- 渲染文本 `javac --release 8` **exit 0**（可编译）；
- 注解读取行为**逐字一致**：原类与重编版均输出 `String/Long/Double/0`（[results/original.out](results/original.out)、[results/recompiled-annotation.out](results/recompiled-annotation.out)）；
- 唯一偏差：反射 `getGenericReturnType()` 由原类 `java.lang.Class<?>` 变为 `class java.lang.Class`（[results/reflection-compare.out](results/reflection-compare.out)），而**无默认的对照元素仍保持 `java.lang.Class<?>`**。

即与同日登记的 `newRet` 形（[generic-declaration-patrol](../generic-declaration-patrol/README.md)）同属"可编译、行为一致、反射元数据降级"的低危保真度缺口，**不触发**核心不变量（非"可编译且行为不同"）。

## 根因与架构警示（本巡查最重要的产出）

**根因**：`src/class_source.rs` 的 `ordinary_parameterized_declaration` 门（约 3895 起）在约 **3934 行**有一条 blanket 拒绝：

```rust
|| attributes.default.is_some()
```

即**任何带 `AnnotationDefault` 属性的成员**都不允许泛型 Signature 投影。拒绝文本因此是 `ordinary_generic_source_unproved` / "member flags, annotation positions, or source name cannot be preserved"。

**这条门是保护性的，不是遗漏**：root 核实泛型投影路径（`ordinary_parameterized_declaration` 及其下游）**完全不拼写 default**（该函数内除这条拒绝判据外无任何 default 处理）。故：

| 方案 | 结果 | 评价 |
| --- | --- | --- |
| 现状（拒绝投影） | `Class withDefault() default java.lang.Object.class;` — 丢 `<?>`，**保留 default** | 语义正确、反射元数据有损 |
| **naive 删掉该判据** | `Class<?> withDefault();` — 保住泛型，**但 default 整个消失** | **更糟**：注解元素从"有默认"变成"必填"，是静默语义回归（重编后使用者若不提供该元素即编译失败，或既有 `@Meta(...)` 使用点语义改变） |
| 正确修法 | `Class<?> withDefault() default java.lang.Object.class;` | 需投影路径**学会承载 default 值** |

**给将来实现者的警示**：不要把 3934 行的 `attributes.default.is_some()` 当作过严判据直接删——它是当前唯一防止"泛型投影吞掉注解默认值"的护栏。正确修法是先让泛型投影路径能拼写 `AnnotationDefault`（复用既有非泛型路径的 default 拼写实现，见 `recover-nested-annotation-defaults`/`recover-floating-annotation-defaults` 两片的 default 处理），再移除该拒绝条件，并以"带默认的泛型元素 default 值逐字不变"为验收锚。

## 查重与归属

- inventory **DT-22**（"注解类型声明及元素默认值"）状态"部分已测、仍待扩验"——本子形（泛型元素 + 默认值）在其范围内但未见专门证据。
- `recover-nested-annotation-defaults`(7/7)、`recover-floating-annotation-defaults`(7/7) 处理的是 default **值形态**（嵌套注解、浮点），均未触及"泛型返回类型的元素"，故不重复。
- 无既有 change 覆盖此形（`grep -rln "field_generic_body_unproved\|annotation.*generic.*default" openspec/changes/` 无命中）。

## 处置

**登记不立片。** 理由：(1) 严重度低（可编译、行为一致、仅反射元数据）；(2) 正确修法需扩展投影路径的 default 承载能力，属中等颗粒，而队列已有 5 项且 5.3 大颗粒里程碑在飞；(3) 按 Goal "优先完成大颗粒语法的里程碑，以 MVP 思维推进" 不应插队。将来若推进 DT-22 扩验，本证据可直接作为其一个子形的基线与判据起点（含上面的架构警示，避免踩 naive 放宽的坑）。

原 class 为行为基准（`String/Long/Double/0`；反射 `withDefault genericReturn=java.lang.Class<?>`）。

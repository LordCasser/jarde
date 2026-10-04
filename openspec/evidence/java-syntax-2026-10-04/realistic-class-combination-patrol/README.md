# 真实类组合巡查（2026-10-04，root）——单类混合九个语法族，全部 gap 均可归属

按 Goal "持续巡查各个反编译语法分析模块，构造各种 Java 语法场景，编译，然后对比源码、jadx、jarde" 构造**一个真实风格的单类**，把九个常见语法族混在一起——因为真实代码是混合的，单形探针会漏掉**交互**问题。主线 HEAD 二进制（`c226413c` 之后重建，环 3 合入态）。

**结论：整类 `javac --release 8` exit 1（响亮失败，非静默偏离）；四个方法未恢复。逐个归属后确认——没有一个是"未知新缺口"，全部落在已登记域内；其中枚举形是 root 本会话刚立的 [recover-enum-string-field-name-generalization](../../../changes/recover-enum-string-field-name-generalization/) 缺陷的独立复现。**

固定转录见 [fixture](fixture/) 与 [results](results/)。

## 探针（[fixture/C1.java](fixture/C1.java)，`javac --release 8`，4 个 class）

单个类内含九个语法族：泛型嵌套静态字段 + diamond、泛型方法（有界类型参数 + 通配符参数化实参）、枚举（常量专属体 + `super.tag()`）、匿名类 + 捕获、try-with-resources（单资源 + catch）、Stream 链（多语句 lambda + 方法引用 + 装箱）、String switch（fallthrough + default）、三元 + 位运算 + 复合赋值 + 后自增。

**原 class 行为基线**（[results/original.out](results/original.out)，7 行，对应 7 个 `println`）：`3` / `Ss/f` / `xy2` / `5` / `[n3, n4]` / `ABC?` / `21`。

## 呈现与逐个归属

| 成员 | jarde 呈现 | 归属 | 严重度 |
| --- | --- | --- | --- |
| `INDEX`（`static Map<String,List<Integer>>` + diamond 初始化） | **raw** `java.util.Map`（丢 `<String,List<Integer>>`） | 同类字段读残留（`field_generic_body_unproved`，见 [generic-declaration-patrol](../generic-declaration-patrol/README.md) 归属一：`recover-same-class-generic-bindings` 11/11 的已登记残留） | 反射元数据降级，响亮（渲染 raw 可编译但反射丢失） |
| `maxOf`（`<T extends Comparable<? super T>> T maxOf(List<? extends T>)`） | `not recovered`（explanation only）+ `generic_source_shape_unproved` | **`recover-generic-method-signatures`（5/8，未勾 2.3/3.1/3.3）**——正是"复杂合法形状"的类级方法头投影，其 1.2 已冻结"有界类型参数"反例，本片属其剩余范围 | 整方法未恢复，响亮 |
| `count`（单资源 TWR + `catch(IOException)` + 返回值体） | `not recovered`（explanation only） | **TWR 域的窄残留**：`recover-enclosing-named-catch`(7/7，已合入 `5644fc85`) 的锚 T1 是 **void 调用语句体 + 用户类资源**；本片是**值返回体（含三元）+ JDK `BufferedReader` 资源**，落在其锚之外。失败响亮（方法体被 explanation-only marker 替换，不可编译） | 整方法未恢复，响亮 |
| `chain`（Stream + 多语句 lambda + 方法引用 + 装箱） | 部分：方法引用降级为显式 lambda + 伴生原名，多语句 lambda 体走基线写法 | 方法引用见 [method-reference-patrol](../method-reference-patrol/README.md)（DT-27 已登记残留）；多语句 lambda 体见 5.2（需 AST 扩展，`present-proved-java-structure` tasks 已量化） | 行为正确，源码形态降级，响亮可编译 |
| `mk`（匿名 `Runnable` + 捕获） | 未投影（引注） | 匿名接口捕获形，属既有匿名域边界 | 响亮 |
| `pick`（String switch + fallthrough） | 恢复（无引注） | 已覆盖（`recover-string-switch` 8/8） | — |
| `arith`（三元 + 位运算 + 复合赋值 + **后自增作返回值**） | `return b++` 的返回位被引注（后自增旧值语义） | **已登记残留**：`recover-compound-lvalue-updates` proposal 第 11 行明写"后置递增返回旧值单列后续工作"；inventory EM-23 覆盖 `++`/`--` 返回值 | 该返回语句响亮引注，其余恢复 |
| `C1$Mode`（枚举，常量专属体 + `super.tag()`，**字段名 `t`**） | 常量退化为 `public static final C1$Mode FAST;` 字段声明（非法 Java） | **枚举 `op` 硬编码缺陷的独立复现**：其字段名是 `t` 不是 `op`，故不投影——与 [enum-string-field-name-hardcode](../enum-string-field-name-hardcode/README.md) 的 C2/C4/C5/C6 同因。这是该缺陷**首次在真实混合类中自然出现**（而非定向探针） | 响亮（非法 Java → `javac` exit 1 `此处需要枚举常量`） |

## 价值

1. **独立复现了枚举缺陷**：`C1$Mode` 用字段名 `t`（真实代码里 `op` 极罕见），在一个与枚举专项探针完全无关的混合类里自然触发了 [recover-enum-string-field-name-generalization](../../../changes/recover-enum-string-field-name-generalization/) 要修的缺陷——印证该缺陷"现实输入基本不生效"的判断，且证明**组合巡查能发现单形定向探针之外的同类问题**。
2. **验证了核心不变量未被破坏**：整类失败是**响亮的**（`javac` exit 1，未恢复方法带 explanation-only marker，枚举退化为非法字段声明），**无一处"可编译但行为不同"**。
3. **全部 gap 可归属**：九个语法族中，`pick` 已覆盖，其余八个的缺口**逐一映射到已立项或已登记的域**（generic-method-signatures 5/8、TWR 窄残留、method-reference DT-27、5.2 多语句 lambda、compound-lvalue 后自增、enum `op` 硬编码、same-class-generic 字段读、匿名接口捕获）——**没有发现需要新立项的未知缺口**，说明当前主线在真实混合类上的边界是清楚且已登记的。

## 处置

**不立新 spec**（无未知缺口）。本记录作为**真实类组合基准**留档，将来各域推进后可重放同一 `C1.java` 验证覆盖面是否扩大。枚举形直接支撑已在飞的 [recover-enum-string-field-name-generalization](../../../changes/recover-enum-string-field-name-generalization/)。

原 class 为行为基准（[results/original.out](results/original.out)）。

## Context

动机与实测见 [proposal](proposal.md)。已知事实（来自 [完成复核](../../completion-review.md) 的“新确认”一节，本 change 按下列代码位置复核过一次）：

- `crates/jarde-java/src/build.rs::spell_reference` 是「descriptor → Java 源拼写」的既有入口：它去掉 `L…;` 包装后把所有 `/` 换成 `.`，对 `[` 开头的 descriptor 不做任何处理，于是 `[B`、`[Ljava/lang/String;` 原样进入文本。
- 数组 descriptor 会到达类型位置，是因为帧对引用值的命名就是 descriptor：`jarde-jvm` 的 `RefType::Named { name, .. }` 保存 `parse_field_type` 读到的名字，而 `parse_field_type` 对 `[` 分支返回整段 descriptor（`[B`、`[Ljava/lang/String;`、`[[I`）。`value_type` 把该名字交给 `spell_reference`，产物于是写出 `Type::Reference("[B")`。
- 同一 crate 已有正确的数组拼写：`lambda.rs::parse_type` 对 `[` 递归解析元素、再用 `"[]".repeat(dimensions)` 拼出 `Type::Reference("int[]")`、`Type::Reference("java.lang.String[][]")`，元素名经 `source_name`（`/` → `.`）拼写。`spell_reference` 的文档注释本身写着「a second spelling of "internal form to source form" is exactly the kind of duplicate that drifts」。
- `facts.rs::parameter_types` 里 `[` 分支给出 `Type::Reference("Object")`：那是**参数布尔判定**用的保守事实，不是类型位置拼写，修改时 MUST NOT 把它当成第二个泄漏点或顺手改成数组拼写。

## Goals / Non-Goals

**Goals:**

- 类型位置上的数组 descriptor MUST 拼成合法 Java 数组类型（原始、引用与多维形态），元素拼写复用既有对象名拼写。
- 不能拼成合法 Java 类型的 descriptor MUST 让该区域拒绝；MUST NOT 原样输出。
- 以 javac 编译对照与精确文本把缺陷、拒绝路径与正向对照永久化。

**Non-Goals:**

- 不做泛型、擦除、数组协变或 `Object` 上转的推理；不新增类型模型、`Type` 变体或公共 API。
- 不改 `value_type` 对非数组类型的决定，不改 `emit.rs` 的打印、AST 或任何报告平面。
- 不重开 R8/R9、已归档的递归界与打印修正；不声称一般语义等价或覆盖整类类型缺陷。
- 不做性能工作（`optimize-demand-workloads` 保持 0/22）；不修 body 解码重新解析类的债务。

## Decisions

### 1. 数组按递归元素拼写，对象名拼写逐字不变

`[` 前缀计数维度、对剩余部分递归拼写元素、再补 `[]`。对象元素沿用现有规则（`L…;` → 去掉包装并把 `/` 换成 `.`），因此 `[B` → `byte[]`、`[Ljava/lang/String;` → `java.lang.String[]`、`[[I` → `int[][]`、`[[Ljava/lang/String;` → `java.lang.String[][]`。`lambda.rs` 的同一拼写与 `build.rs` 的这份 MUST 合并为一条（或由测试固定两处逐字一致），MUST NOT 保留两份会漂移的拼写。

### 2. 类型位置清单

实施者 MUST 从当前代码枚举所有把 descriptor/类型写成 Java 文本的位置，逐条记录可达性与覆盖：

| 位置（写入点） | 本次处置 |
| --- | --- |
| 局部变量的声明类型（`declare` → `Type::spell`，经 `value_type` → `spell_reference`） | **已实测的缺陷点**；受控 fixture + 精确文本 + javac 编译对照 |
| `ExprKind::New { ty }`（构造的类名） | 复核可达性：数组的构造是否到达该位置；可达则进 fixture，不可达则给出代码证据 |
| `ExprKind::Path(owner)`（静态 owner/实现类名） | 同上 |
| 其它由 `spell_reference` 写出的文本 | 按调用点清单逐条复核 |

不得只改声明路径就声明整类关闭；也不得把「声明位置已修」当成其它位置已受检。

### 3. 拒绝路径

`spell_reference` 收到的输入若不是可解析的字段 descriptor（畸形、截断、方法 descriptor、空串等），该区域 MUST 拒绝：保留 bytecode 与 origin、`representation=Mixed`、`quality=Fallback`、诊断点名 BCI 与物理方法。MUST NOT 把原字符串写进文本（现状），也 MUST NOT 用 `Object` 之类的占位符替换一个已读到的类型。

### 4. 受控 fixture 与「声称 Java」的验收

fixture 放在 `tests/fixtures/p3-array-types/`：

- `ArrayTypes.java`：受控源码，含缺陷形状与正向对照；
- `v8/ArrayTypes.class`：`javac --release 8 -g:none -d v8 ArrayTypes.java` 的真实输出；
- `README.md`：编译器版本与确切命令、字节数、SHA-256、逐成员字节码（沿用 `p3-nested-arithmetic` 格式），并记录修正前 javac 在 `[B` 处的拒绝信息；
- `Baseline.java`：原 class 的驱动，打印输入集合的原值。

规划期候选成员（成员名、正文与精确文本由实施者按实测记录，下表用于验收对照，不得据此反推；数组参数在对照里以 `null` 调用）：

| 成员 | 形状 | 修正前 | 修正后 |
| --- | --- | --- | --- |
| `echoed([B)[B` | `byte[] local = copy(value); return local;` | `[B local1 = copy(arg0); return local1;` | `byte[] local1 = copy(arg0); return local1;` |
| `named([Ljava/lang/String;)[Ljava/lang/String;` | `String[] local = value; return local;` | `[Ljava.lang.String; local1 = arg0; return local1;` | `java.lang.String[] local1 = arg0; return local1;` |
| `grid([[I)[[I` | `int[][] local = value; return local;` | `[[I local1 = arg0; return local1;` | `int[][] local1 = arg0; return local1;` |
| `table([[Ljava/lang/String;)[[Ljava/lang/String;` | `String[][] local = value; return local;` | `[[Ljava.lang.String; local1 = arg0; return local1;` | `java.lang.String[][] local1 = arg0; return local1;` |
| `text(Ljava/lang/String;)Ljava/lang/String;`（对照） | `String local = value; return local;` | `java.lang.String local1 = arg0;` | 逐字不变 |

（`copy([B)[B` 是 fixture 内的静态 helper，用来让第一个成员走「调用结果填进局部」的形状。）

验收沿用 `tests/p3_execution_comparison.rs` 的既有流程：从本次运行事实派生成员声明（descriptor、access flags、参数槽类型）、`javac --release 8` 编译呈现文本、两侧执行同一输入集合并逐行比较 trace。该文件已支持数组拼写（`java_type` 的 `[]` 分支）；数组参数的样本值走 `sample_values` 的兜底分支（`null`），`default_value` 对非基本类型给 `null`，所以「产物声称 Java 就必须被 javac 接受」这条判据在该路径上可直接执行。稳定回归（精确文本与拒绝边界）放在新的 `tests/p3_array_types.rs`。

### 5. 复用与依赖

不需要新库：`lambda.rs::parse_type`/`source_name` 与 `build.rs::spell_reference` 已覆盖所需拼写，本次是让唯一入口正确并让两处一致。`javac` 仍是测试侧外部 oracle（固定版本、缺席如实报告）。新增 fixture 会改变 `tests/fixtures/corpus-fingerprint.json` 与 reader census 计数，两者都按既有流程显式再生成/更新并记录实测值。

## Risks / Trade-offs

- **只修声明路径** → 类型位置清单与逐条可达性记录；未覆盖的位置必须给出代码证据。
- **顺手改了对象名拼写** → `text(Ljava/lang/String;)Ljava/lang/String;` 对照 MUST 逐字不变。
- **多维与引用元素拼错（`java.lang.String[][]` 与 `String[][]` 混用）** → 元素拼写由既有规则决定并在 README/verification 记录实测文本；同一元素类型的两种合法写法不得混进同一产物。
- **把不可解析的输入原样输出** → 拒绝路径验收必须存在，且不得以「没有生成 Java」通过。
- **两处拼写漂移** → 合并成一条，或由测试固定两处逐字一致；不得只改一处。
- **语料变更被静默接受** → fingerprint 再生成器与 reader census 都要求记录实测计数。

## Migration Plan

1. 在固定基线上先记录反例：恢复正文、报告平面、把正文放进正确签名的 wrapper 后 javac 的拒绝信息（修正前完成）。
2. 枚举类型位置并记录可达性；确认与 `lambda.rs` 拼写的一致策略。
3. 实施最小修正，保持对象名与非数组类型逐字不变。
4. 提交 fixture 与来源 README，执行 fingerprint 再生成与 census 更新，加入精确文本回归、javac 编译对照、拒绝边界与变异。
5. 跑固定提交门禁并写 verification；同步 delta 与状态引用后归档。

与 `group-call-receivers`、`type-boolean-contexts` 串行实施（同一 crate，本 change 第 3 个）。回退按本 change 的独立提交进行；回退后必须恢复「产物在 `[B` 处无法编译」的公开事实。

## Open Questions

无。类型位置的可达性与两处拼写的合并方式由实施者按当前代码查证并记录，这不是需要上游裁决的设计歧义。

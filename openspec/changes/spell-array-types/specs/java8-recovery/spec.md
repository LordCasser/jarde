## ADDED Requirements

### Requirement: A type position is spelled as a legal Java type

类型位置上的类型 MUST 以合法 Java 源类型拼写，而不是原样输出 class file 的 descriptor：数组 descriptor MUST 拼成 `元素类型[]`（原始、引用与多维形态，例如 `byte[]`、`java.lang.String[]`、`int[][]`、`java.lang.String[][]`），元素名的拼写 MUST 复用既有的对象名拼写（`L…;` 去掉包装、`/` 换成 `.`）；对象类型与基本类型的既有拼写 MUST 逐字不变。层无法把某个 descriptor 拼成合法 Java 类型时 MUST 拒绝该区域并保留 bytecode 与 origin，MUST NOT 把 descriptor 原样写进文本，也 MUST NOT 用占位类型替换一个已读到的类型。representation/quality/content/execution 等结构平面 MUST NOT 被当作本要求的证据。

#### Scenario: A primitive array in a type position

- **WHEN** 局部声明由数组参数或返回数组的调用填充，帧事实给出的 descriptor 是 `[B`（源码形状 `byte[] local = value;`）
- **THEN** 呈现 MUST 是该位置的合法 Java 数组类型（`byte[] local1 = …;`），MUST NOT 含 `[B`；产物声称 Java 时 javac MUST 接受该声明

#### Scenario: A reference array in a type position

- **WHEN** 同一位置上的 descriptor 是 `[Ljava/lang/String;`
- **THEN** 呈现 MUST 是元素名按既有对象名规则拼写后的数组类型（`java.lang.String[] local1 = …;`），MUST NOT 含 `[Ljava.lang.String;`；javac MUST 接受

#### Scenario: Multi-dimensional arrays in a type position

- **WHEN** 同一位置上的 descriptor 是 `[[I` 或 `[[Ljava/lang/String;`
- **THEN** 呈现 MUST 分别是 `int[][]` 与 `java.lang.String[][]`，MUST NOT 含 `[[I` 或 `[[Ljava.lang.String;`；javac MUST 接受

#### Scenario: A legal non-array type is unchanged

- **WHEN** 类型位置上是对象类型或基本类型（例如 `Ljava/lang/String;` → `java.lang.String`）
- **THEN** 呈现 MUST 与修正前逐字相同；本要求 MUST NOT 用普遍改写拼写换取反例通过

#### Scenario: An unspellable descriptor is refused

- **WHEN** 类型位置上的 descriptor 不能被拼成合法 Java 类型（畸形、截断或非字段 descriptor 形态）
- **THEN** 该区域 MUST 拒绝并保留 bytecode 与 origin（representation=Mixed、quality=Fallback、拒绝诊断点名相关 BCI 与物理方法），MUST NOT 原样输出该 descriptor；拒绝产物按既有 refusal 契约可定位

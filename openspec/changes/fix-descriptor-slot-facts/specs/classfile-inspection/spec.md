## ADDED Requirements

### Requirement: Descriptor facts are parsed once, by the reader

读取层 SHALL 提供唯一的描述符解析：一次调用给出基本类型（含 `void` 的合法位置判定）、原始类名、数组维数、参数序号与按 JVMS 计算的槽占用。该事实 SHALL 供 JVM 层的槽位计算与 Java 层的源码拼写共同消费；MUST NOT 存在第二份按描述符自行推槽宽或维数的实现。畸形描述符 MUST 沿用既有拒绝路径与错误码，不因统一解析而放宽或改写。

#### Scenario: Every consumer reads the same facts

- **WHEN** frame 推导、lambda 槽位、方法声明与 source 拼写各自需要一个描述符的类型与宽度
- **THEN** 它们 MUST 从同一次解析的事实取值；替换其中任何一个消费者 MUST NOT 需要改动另一处的解析实现（F02）

#### Scenario: An array's width is its own

- **WHEN** 描述符是 `long[]`、`double[][]` 或任意维数的引用类型
- **THEN** 事实 MUST 报告它的槽占用为**一个**槽，并单独报告维数与元素类型身份；该报告 MUST NOT 依赖元素宽度（F01）

#### Scenario: A malformed descriptor is refused as before

- **WHEN** 描述符在读取层被判定为畸形（截断、非法字符、非法返回类型）
- **THEN** 请求 MUST 以既有错误码拒绝，MUST NOT 由统一解析引入新的宽松解释（F03）

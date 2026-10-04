## Why

**已验收切片的能力实际上只在它自己的 fixture 字段名上生效**——这是本片要修的缺陷，root 2026-10-04 实测确立。

`recover-proved-string-arg-enum-constant-bodies`（8/8，2026-09-27 提交 `7e49c75f`）的 Requirement 声称的能力边界是"每个常量只带一个可无损拼写的 ASCII 字符串字面量源实参"，**其 spec 中没有任何字段名条件**（root 实测：`grep -niE "op\b|字段名|field name"` 于该 change 的 `specs/java8-recovery/spec.md` 无命中）。但实现把**字段名硬编码为 `op`**，共五处（行号随主线漂移，以锚点名为准）：

| 落点 | 内容 |
| --- | --- |
| `src/facade.rs:14094` | 构造器 BCI 6 的 `putfield` 必须 `name == b"op"`，否则 `Err("the String constructor does not preserve Enum and op semantics")` |
| `src/facade.rs:15906` | 查找源字段时要求 `field.item.name.raw().0 == b"op"` |
| `src/facade.rs:15920` | 唯一性检查按 `== b"op"` 计数 |
| `src/facade.rs:15923` | 再核对 `op_field.item.name.raw().0 != b"op"` |
| `src/facade.rs:15928` | **发射文本字面量** `"    private {}(java.lang.String arg0) {{\n        this.op = arg0;\n    }}\n"` —— 构造器体内硬写 `this.op` |

而 `op` **正是该切片验收锚 `TestEnums2a/DoubleOperations` 的字段名**（`private final String op;`）。故该验收是**自我实现的**：锚能通过不代表能力成立。

**root 实测判别（六形对照，仅字段名不同，结构逐字节同构）**：

| 探针 | 字段名 / getter | 结果 |
| --- | --- | --- |
| C1 | `op` / `getOp` | 投影成功（常量列表 `TIMES("*") { … }`） |
| C3 | `op` / `getT` | **投影成功**（getter 名无关） |
| C2 | `t` / `getT` | 不投影（常量退化为 `public static final demo.C2 TIMES;` 字段声明） |
| C4 | `t` / `getOp` | 不投影 |
| C5 | `value` / `getValue` | 不投影 |
| C6 | `x` / `getX` | 不投影 |

即**决定因素是字段名本身**，与 getter 名、包名、常量名、常量体方法的参数个数与是否 `@Override` 均无关（root 另测：`A/B` 与 `TIMES/DIVIDE` 常量名、默认包与 `demo` 包、0 参与 2 参常量体方法、是否实现接口——均不改变结论）。

**严重度（root 实测，非推断）**：不投影时枚举被呈现为"字段声明式常量"，这是**非法 Java**（`javac` 报 `此处需要枚举常量`），完整源集 `javac --release 8` **exit 1** —— 属**响亮失败，非静默偏离**，故不触发本会话确立的核心不变量。但它是**真实能力缺口**：现实代码几乎不会把枚举字段命名为 `op`，故该切片声称的"带一个 String 实参的枚举常量体"能力在真实输入上基本不生效。

固定转录见 [fixture](../../evidence/java-syntax-2026-10-04/enum-string-field-name-hardcode/fixture/) 与 [results](../../evidence/java-syntax-2026-10-04/enum-string-field-name-hardcode/results/)。

## What Changes

- 把五处 `op` 字面量改为**由字节码证明的字段名**：构造器 BCI 6 的 `putfield` 目标名（`EnumCodeReference::Field { name, … }`，该绑定在 `facade.rs:14094` 已经存在、但被 `matches!` 丢弃）即为唯一权威来源；把它随证明结果传递到发射处，`this.op` 改为 `this.<已证字段名>`。
- 唯一性判据从"名为 `op` 的字段恰一个"改为"**被构造器 `putfield` 写入的 String 字段**恰一个"，其余既有属性检查（描述符 `Ljava/lang/String;`、owner == 本枚举定义、非 static/synthetic/enum-implicit、`ACC_PRIVATE`、有 `declaration`、无 markers）逐字保留。
- 发射的构造器文本用已证字段名拼写；**不得**继续硬写 `op`。

## Capabilities

### Modified Capabilities

- `java8-recovery`：带一个 String 源实参的枚举常量体投影不再依赖字段名恰为 `op`；其能力边界回归 `recover-proved-string-arg-enum-constant-bodies` 的 Requirement 原文所声称的范围（"一个可无损拼写的 ASCII 字符串字面量源实参"），不含隐含的字段名条件。

## Impact

`src/facade.rs`（五处；可能需在 `PendingEnumConstructorEdge` 或等价的证明载体上增加已证字段名字段——该结构当前只有 `caller`/`call_bci`/`target_owner`/`target_descriptor` 四项，**不携带字段名**，故名字在 `matches!` 之后即丢失，这是本片必须打通的数据通路）。`crates/jarde-java` 预期无改动。

**冻结 fixture 与既有验收**：`TestEnums2a/DoubleOperations`（字段名恰为 `op`）的呈现必须**逐字节不变**——它是本片最重要的零回退锚，因为它同时是"名字恰为 `op`"与"名字由字节码证明"两种实现的共同正例。`recover-proved-string-arg-enum-constant-bodies` 的 8 项任务与全部既有测试零回退。

**Non-Goals（不得越界）**：不放宽 String 实参的可拼写性判据（非 ASCII / 含转义仍拒绝）；不支持**多个** String 源实参；不处理 int/long 等其它描述符的常量实参（属 `project-proved-enum-switch-labels` 等其它域）；不改常量体的匿名子类投影判据；不泛化接口匿名路径的 `b"D"` 特化（属 `recover-anonymous-parameterized-root` 的登记后续）。

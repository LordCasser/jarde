## ADDED Requirements

### Requirement: 静态泛型字段初始化 SHALL 呈现为合法文本

当静态字段的泛型 Signature 投影被拒（`field_generic_body_unproved`）且其初始化表达式含泛型类构造调用时，系统 SHALL 把该初始化呈现为**合法 Java 文本**：或为裸类型正确形（构造器按擦除描述符呈现实参 cast，与既有实例字段退化路径同构，如 `new Hold((java.lang.Object) "a")`），或为响亮引注（拒绝 + bytecode 标记）。系统 SHALL **不得**输出丢失括号/类型实参或类名与类型文本纠缠的损坏中间态。

泛型 Signature 投影本身的拒绝注释 SHALL 保留（投影能力是后续独立域）；实例字段的既有退化路径 SHALL 逐字不变。

#### Scenario: 修复后可编译且行为一致

- **WHEN** `MN`（`static Hold<String> f1 = new Hold<>("a")` 与显式实参 `f2`）经 `class-source` 呈现（javac23 `--release 8` 与真 javac 8 双腿产物）
- **THEN** 渲染不含损坏文本（无 `Holdava` 类拼接残片）、渲染源集 `javac --release 8` exit 0、`main` 输出 `a b 5` 与原 class 一致；泛型投影拒绝注释仍在（本能力不修投影）

#### Scenario: 实例字段零回退

- **WHEN** 同类中实例泛型字段（`Hold<Integer> f3 = new Hold<>(5)`）经呈现
- **THEN** 其既有构造器赋值退化路径逐字不变

#### Scenario: 无损坏中间态

- **WHEN** 审查任何含静态泛型字段初始化的渲染输出
- **THEN** 每个字段声明的初始化文本要么可编译、要么带引注——不存在第三态

## ADDED Requirements

### Requirement: Class source projects proved interface field initialization

对于 Java 8 普通接口，类源码 SHALL 仅在接口自身所有字段的初始化来源、赋值目标、求值顺序、一次执行与重编译后初始化阶段均可证明时，将 `<clinit>` 中的运行时赋值恢复为字段声明初始化式。原字段只有 `<clinit>` 写入时，投影 MUST NOT 使生成源码把它变成编译期常量；不能证明 RHS 在 Java 源码中仍是非常量表达式则 SHALL 拒绝整组。生成文本 MUST 是该范围内合法的接口源码；重编译后的初始化结果、调用次数、顺序与异常观察 MUST 和原 class 一致。投影 MUST 保留物理字段及 `<clinit>` 的身份、原始恢复报告与被搬移效果的来源；字段表顺序、字段名或恢复后的 Java 文本本身 MUST NOT 被当作执行顺序证明。若整组无法忠实投影，系统 SHALL 明确保留未恢复工作或停止原因，MUST NOT 把部分初值写成貌似完整的接口源码，也 MUST NOT 声称已编译或验证。

#### Scenario: Several effectful field initializers

- **WHEN** 普通接口有一项 `ConstantValue` 字段及三项无 `ConstantValue` 字段，后者各由 `<clinit>` 按顺序唯一赋值且值的计算有可观察调用
- **THEN** 类源码 SHALL 给三项各写一个初始化式，不写接口 `static {}`，并保留常量字段原初值；完整 Java 8 重编译、`-Xverify:all` 及初始化轨迹 SHALL 与原 class 相同

#### Scenario: Physical field table differs from execution order

- **WHEN** 两个无 `ConstantValue` 字段的物理字段表顺序与 `<clinit>` 的赋值顺序不同，但完整轨迹仍满足安全投影条件
- **THEN** 源字段声明的求值顺序 SHALL 服从已证 `<clinit>` 顺序；报告中的原始字段表索引与声明归属 MUST 仍可对照，生成源码执行结果 MUST 与原 class 相同

#### Scenario: Qualified forward field read observes a default value

- **WHEN** 较早初始化式以限定名读取同接口较晚才写入、且不是编译期常量的字段，即使物理字段表把被读字段排在前面
- **THEN** 投影后的执行 SHALL 保留该读取在较晚写入之前的事实及其默认值；若无法证明源声明顺序与名称绑定会保留此观察，SHALL 拒绝整组，MUST NOT 输出可编译但错值的字段初值

#### Scenario: Constant fields and initialization source remain distinct

- **WHEN** 一些字段有真实 `ConstantValue` 属性，另一些只有 `<clinit>` 赋值
- **THEN** 同一字段 MUST NOT 获得两个初始化式或把运行时赋值伪称为 `ConstantValue`；原 `<clinit>` 的物理身份、报告与投影来源 SHALL 保留

#### Scenario: Runtime constant write must not become a compile-time constant

- **WHEN** 合法接口 class 的 `EARLY` 在 `<clinit>` 先读取尚无 `ConstantValue` 的 `LATE` 默认值，之后 `LATE` 才由常数指令写入 9，原 JVM 观察为 `0|9`
- **THEN** 投影 SHALL 保留 `0|9` 的初始化阶段观察，或拒绝整组；MUST NOT 输出会使 javac 生成 `ConstantValue`、将较早读取内联为 9 并运行成 `9|9` 的貌似完整源码

#### Scenario: Unproved or additional initialization effects

- **WHEN** 接口初始化含额外副作用、重复/遗漏字段赋值、跨块或异常处理流、数组或值的来源无法呈现、字段身份或求值位置无法证明
- **THEN** 系统 SHALL 拒绝整组投影并保留相关来源与明确原因；MUST NOT 删除效果、重排异常或只将部分赋值搬到字段

#### Scenario: Stop and unaffected class kinds

- **WHEN** 初始化的读取、分析或输出遇到预算拒绝/取消，或输入不是本要求支持的普通接口
- **THEN** 执行状态 SHALL 如实记录停止或沿用该类种原有行为；MUST NOT 把停止当成空初始化，也 MUST NOT 将普通类、枚举或注解类型的 `<clinit>` 套用本接口投影

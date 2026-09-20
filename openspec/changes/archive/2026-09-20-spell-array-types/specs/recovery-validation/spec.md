## ADDED Requirements

### Requirement: A type-position artifact compiles or states its refusal

类型位置拼写的验收 SHALL 以「用本次运行自己的事实派生成员声明、把产物交给 javac 编译」为准（沿用既有 `tests/p3_execution_comparison.rs` 的包装器路径）。产物声称 Java（representation=Java）时其文本 MUST 被 javac 接受；修正前 `[B`、`[[I`、`[Ljava.lang.String;` 在类型位置被 javac 拒绝的事实 MUST 作为实测边界记录，MUST NOT 因为该成员没有执行而被算作通过。选择拒绝的区域 MUST 以拒绝范围、被引用 BCI 与物理方法映射作为其边界证据。MUST NOT 用 representation/quality/content/coverage 任一平面代替编译结果。验收 MUST 覆盖原始数组、引用数组、多维数组（含二维引用数组）以及一个非数组类型的逐字对照。

#### Scenario: The artifact is accepted by javac

- **WHEN** 原始数组、引用数组与多维数组成员的呈现文本包进由本次事实派生的声明（参数与返回类型来自运行自己的 descriptor）
- **THEN** javac MUST 接受每个 wrapper；修正前的 `[B`/`[[I`/`[Ljava.lang.String;` MUST 在 verification 里作为实测拒绝记录（其拒绝信息原样保存），不得作为通过

#### Scenario: Mutation restores the descriptor text

- **WHEN** 变异恢复「数组 descriptor 原样通过」的行为后重跑已提交检查
- **THEN** 精确文本断言 MUST 失败，且编译对照 MUST 在该形态（`[B` 等）的 javac 拒绝上变红；MUST NOT 用其它用例的红代替；变异恢复后不遗留调试改动

#### Scenario: Non-array types stay as they were

- **WHEN** 对象类型与基本类型的样本在修正后运行
- **THEN** 文本 MUST 逐字不变并继续通过编译对照；本修正 MUST NOT 用普遍改写拼写换取反例通过（验收 A13）

#### Scenario: A refused type position passes only with its boundary

- **WHEN** 某个类型位置按「不能拼成合法 Java 类型」选择了拒绝
- **THEN** 验收核对拒绝范围、被引用 BCI 与物理方法映射；仅「没有可执行文本」不构成通过，也没有产物可以在无证明的情况下声称结构化（验收 A13）

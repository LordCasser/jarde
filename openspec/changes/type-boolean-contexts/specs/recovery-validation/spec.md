## ADDED Requirements

### Requirement: A boolean context's artifact compiles and agrees when executed

boolean 上下文的验收 SHALL 以「用本次运行自己的事实派生成员声明、把产物交给 javac 编译、并以同一输入集合执行两侧」为准（沿用既有 `tests/p3_execution_comparison.rs` 的包装器路径）。`Z` 返回与 boolean 条件样本的呈现文本 MUST 能在方法自己的 descriptor 下编译；两侧返回值 MUST 逐项相同。修正前的 `return 1;`/`return 0;` 与 `if (flag() != 0)` MUST 被记录为 javac 的拒绝，MUST NOT 因为该成员没有执行而被算作边界通过。MUST NOT 用 representation/quality/content/execution 任一平面或「文本看起来像 Java」代替编译与执行。选择拒绝的区域 MUST 以拒绝范围、被引用 BCI 与物理方法映射作为其边界证据。

#### Scenario: The artifact compiles under the member's own descriptor

- **WHEN** 两个缺陷形状的呈现文本包进由本次事实派生的成员声明（返回 `Z` 的 `isZero`、条件在 `parity`）
- **THEN** javac MUST 接受；修正前的 `return 1;`/`return 0;`（`int cannot be converted to boolean`）与 `if (flag() != 0)`（`incomparable types: boolean and int`）MUST 在 verification 里作为实测边界记录，不得作为通过

#### Scenario: Both sides return the same values

- **WHEN** 两侧以同一输入集合执行（至少含 `isZero(0)`/`isZero(1)` 与 `parity` 的正、负输入）
- **THEN** 每个输入的返回值 MUST 逐项相同；差异 MUST 使验收失败并指名输入、成员与对应 BCI（验收 A13）

#### Scenario: Mutation restores the untyped presentation

- **WHEN** 变异恢复「按值渲染、不按返回 descriptor 定型」或「条件只识别 boolean 参数」的行为后重跑已提交检查
- **THEN** 至少一个检查 MUST 变红，且失败现象是该形态在 javac 下的拒绝或精确文本不符；MUST NOT 用其它用例的红代替；变异恢复后不遗留调试改动

#### Scenario: Int-shaped controls stay int-shaped

- **WHEN** 真正 `int` 返回的方法、int 比较以及既有 boolean 参数路径在修正后运行
- **THEN** 它们 MUST 保持 `1`/`0`、`!= 0` 与 `if (arg0)` 的既有呈现并继续通过编译执行对照；本修正 MUST NOT 用普遍改写成 boolean 换取反例通过（验收 A13）

#### Scenario: A boolean context's refusal passes only with its boundary

- **WHEN** 某个 boolean 上下文区域按证据不足选择了拒绝
- **THEN** 验收核对拒绝范围、被引用 BCI 与物理方法映射；仅「没有生成可执行文本」不构成通过，也没有产物可以在无证明的情况下声称结构化（验收 A13）

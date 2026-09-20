## ADDED Requirements

### Requirement: A local's decided type compiles and agrees when executed

局部类型决定的验收 SHALL 以「用本次运行自己的事实派生成员声明、把呈现文本交给 `javac --release 8` 编译、并以同一输入集合分别运行原 class 与呈现文本、逐项比较返回值/可观察行为」为准（沿用既有 `tests/p3_execution_comparison.rs` 的包装器路径）。提升声明与就地声明对同一个值的决定 MUST 被同一份样本覆盖；MUST NOT 用 representation/quality/content/execution 任一平面、「文本看起来像 Java」、或某条文本断言代替编译与执行。修正前的 `int local3; … local3 = <boolean 值>; … if (local3 != 0)` 一类正文（`boolean cannot be converted to int` 一类拒绝）MUST 在 verification 里作为实测边界记录，不得作为通过。验收 MUST 覆盖：交换分支顺序后仍得同一结论、经另一个 boolean 局部的复制链、纯 `int` 对照与既有的 descriptor 证据局部；选择拒绝的区域 MUST 以拒绝范围、被引用 BCI 与物理方法映射作为其边界证据。变异让提升路径退回 descriptor-only 证据、或让一致性检查只读第一次写入时，至少一个已提交检查 MUST 变红；MUST NOT 用其它用例的红代替。

#### Scenario: The hoisted shape compiles under the member's own descriptor

- **WHEN** 提升形状（`boolean a = b; boolean c; if (n == 0) c = a; else c = b; if (c) …`）的呈现文本包进由本次事实派生的成员声明
- **THEN** javac MUST 接受；修正前的 `int local3; … local3 = local2; …`（`boolean cannot be converted to int` 一类）MUST 在 verification 里作为实测拒绝记录（拒绝信息原样保存），不得作为通过（验收 A13）

#### Scenario: Both sides return the same values

- **WHEN** 两侧以同一输入集合执行（覆盖 `n == 0` 与 `n != 0`、`b` 的两个取值，以及跨分支复制链的输入）
- **THEN** 每个输入的返回值 MUST 逐项相同；差异 MUST 使验收失败并指名输入、成员与对应 BCI（验收 A13）

#### Scenario: Swapping the branches keeps one conclusion

- **WHEN** 同一形状以交换后的分支顺序再次恢复并对照
- **THEN** 该变量的类型与它在使用处的拼写 MUST 与未交换的样本一致（只有分支文本本身按源码位置不同），两侧执行仍逐项相同；两个顺序得出不同类型即验收失败（验收 A13）

#### Scenario: Mutation restores the descriptor-only hoisting path

- **WHEN** 变异把提升路径退回「只读 descriptor 证据（`boolean_proof`）」后重跑已提交检查
- **THEN** 至少一个检查 MUST 变红，失败现象是该形状的精确文本不符（`int local3` 一类）或 javac 拒绝；MUST NOT 用其它用例的红代替；变异恢复后不遗留调试改动（验收 A13）

#### Scenario: Mutation checks only the first write

- **WHEN** 变异让一致性检查退回「只看第一次写入」后重跑已提交检查
- **THEN** 至少一个检查 MUST 变红，失败现象是某个分支的写入按决定拼不出文本（或该冲突未被拒绝）；MUST NOT 用其它用例的红代替；变异恢复后不遗留调试改动（验收 A13）

#### Scenario: A refused structure passes only with its boundary

- **WHEN** 某个局部因未知或冲突按既有 refusal 契约被拒绝
- **THEN** 验收核对拒绝范围、被引用 BCI 与物理方法映射；仅「没有生成可执行文本」不构成通过，也没有产物可以在无证明的情况下声称结构化（验收 A13）

#### Scenario: The controls keep their text and their execution

- **WHEN** 修正后运行由 boolean 参数或返回 `Z` 的调用提升的局部、只被 `0`/`1` 填充的 `int` 局部、以及 `p3-boolean-contexts` 的既有样本
- **THEN** 它们的文本 MUST 逐字不变并继续通过编译执行对照；literal-armed 形状按记录的边界验收（自洽文本 + 两侧执行一致），MUST NOT 被写成类型已与源码一致；本修正 MUST NOT 用普遍改写成 boolean 或普遍拒绝换取反例通过（验收 A13）

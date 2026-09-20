## ADDED Requirements

### Requirement: A presented concatenation is compared by execution

拼接转换的验收 SHALL 以受控执行对照为准：用本次运行自己的事实派生成员声明，把呈现文本交给 `javac --release 8` 编译，并以同一输入集合分别运行原 class 与呈现文本、逐项比较返回值/可观察行为（沿用既有 `tests/p3_execution_comparison.rs` 的包装器路径）。MUST NOT 用「两侧都能编译」、representation/quality/content/execution 任一平面、或「该链被记录为 presented」代替该对照——已知反例正是两侧都能编译而值不同。反例 MUST 在修正前记录：`append(a).append(b).append("!")` 的呈现是 `arg0 + arg1 + "!"`，输入 `(1, 2)` 时原 class 返回 `"12!"`、呈现文本返回 `"3!"`，两侧都编译通过；MUST NOT 以「文本看起来像 Java」或结构平面代替这条记录。对照 MUST 覆盖多个数值/字符串顺序（至少：两个数值在前、需要转换的片段在最后、全字符串），MUST 覆盖 boolean、`null` 与对象片段的转换，MUST 覆盖「片段本身是加法」与「两个独立片段」这一对分组形态（两者在同一输入上的值不同，任何合并或拆分都会让对照变红），并 MUST 包含一个可能抛异常/可观察的片段用于比较求值次数与顺序。变异恢复「从第一个原始值起左折叠、不建立字符串上下文」的行为后，对照 MUST 在至少一个输入上以值不同变红；MUST NOT 用其它用例的红代替，变异恢复后不遗留调试改动。选择拒绝的区域 MUST 以拒绝范围、被引用 BCI 与物理方法映射作为其边界证据。

#### Scenario: The counterexample's two values are recorded

- **WHEN** 修正前的基线上运行该反例（`append(int).append(int).append(String)`，输入 `(1, 2)`）
- **THEN** verification MUST 记录原 class 的 `"12!"`、呈现文本的 `"3!"` 与「两侧都编译」的事实，并指名该成员与对应 BCI；MUST NOT 以「文本看起来像 Java」或结构平面代替这条记录（验收 A13）

#### Scenario: Several orders are compared

- **WHEN** 对照运行受控样本的输入集合与全部顺序成员（两个数值在前、数值在最后、全字符串）
- **THEN** 每个输入的返回值/可观察行为 MUST 在两侧逐项相同；差异 MUST 使验收失败并指名输入、成员与对应 BCI（验收 A13）

#### Scenario: Merged and split parts are both caught

- **WHEN** 对照运行「一个片段本身是加法」（`append(a + b).append("!")`）与「两个独立片段」（`append(a).append(b).append("!")`）两个成员
- **THEN** 两者在 `(1, 2)` 上 MUST 分别得到 `"3!"` 与 `"12!"` 的同一值；把前者拆开或把后者合并的实现 MUST 被该对照以值不同捕获（验收 A13）

#### Scenario: Boolean, null and object fragments are compared

- **WHEN** 对照运行 `append(boolean)`、`append((Object) null)` 与 `append(object)` 形态的成员
- **THEN** 每个输入的返回值 MUST 在两侧逐项相同；`append(true)` 被写成 `1` 一类差异 MUST 被该对照捕获（`"true"` 对 `"1"`）（验收 A13）

#### Scenario: The evaluation order and count are compared

- **WHEN** 样本里有一个可能抛异常或可观察的片段夹在需要转换的片段之间
- **THEN** 两侧的调用次数、顺序与异常 MUST 逐项相同；把转换移到别处（例如对求和结果转换）而改变求值形态的文本 MUST 被该对照捕获（验收 A12、A13）

#### Scenario: Mutation restores the lost conversion

- **WHEN** 变异恢复「从第一个原始值起左折叠、不建立字符串上下文」的行为后重跑该对照
- **THEN** 对照 MUST 在至少一个输入上以值不同变红（`(1, 2)` 的 `"12!"` 对 `"3!"` 即该失败）；MUST NOT 用其它用例的红代替；变异恢复后不遗留调试改动（验收 A13）

#### Scenario: The unchanged chains stay unchanged

- **WHEN** 修正后运行全字符串链与「需要转换的片段在最后」的链
- **THEN** 两者的文本 MUST 逐字不变并继续通过编译执行对照；本修正 MUST NOT 用普遍插入空字符串或普遍改写转换换取反例通过（验收 A13）

#### Scenario: A refused chain passes only with its boundary

- **WHEN** 某条链因某个片段的转换无法表达而被拒绝
- **THEN** 验收核对拒绝范围、被引用 BCI 与物理方法映射；仅「没有生成可执行文本」不构成通过，也没有产物可以在无证明的情况下声称结构化（验收 A13）

## MODIFIED Requirements

### Requirement: A signal is not an acceptable answer

恢复验收 SHALL 把「修正前以进程 signal 结束」的输入纳入永久回归，并在公开入口核对修正后得到的是报告。该回归 MUST 在独立进程内运行这个输入，因为 signal 是进程级事实，进程内的断言捕获不到它。验收证据 MUST 是报告的 `outcome`、`execution`、诊断与 CLI 退出状态；MUST NOT 以「stderr 没有 stack overflow」「进程存活」或「日志里没有 abort」单独算作通过，也 MUST NOT 通过扩大预算、在测试外重跑或降低输入来绕过。修正前该回归 MUST 失败，且失败现象是 signal/无报告，而不是断言缺失或编译错误。

对**深拼接链**这一族，输入 MUST 由受控、确定性的构造给出并直接产出字节：链长是本检查的参数（复核区间 N = 1536/2048 与更大规模都要跑），生成器能确定地给出它，而 `javac -J-Xss64m` 一类「放大编译器栈」的产出手段 MUST NOT 进入回归。生成器 MUST 点名它镜像的复现（同一 `StringBuilder` 链、同一 overload、同一直线、同一区间），库入口与进程入口 MUST 使用同一份字节。检查 MUST 在**子进程级**运行，并 MUST 覆盖 debug 与 release 两个构建：两个构建都 MUST 返回报告，正常完成与预算停止后的清理都 MUST 被覆盖，MUST NOT 出现 signal、空 stdout 或 abort。界与完成性 MUST NOT 依赖线程栈：检查 MUST 在默认栈上运行，MUST NOT 通过 `RUST_MIN_STACK`、大栈线程或等价手段取得结论——放大线程栈不是界的替代。修正前的 signal MUST 在 verification 里记录一次（命令、exit 状态、stderr 的栈溢出行与空 stdout 的事实），MUST NOT 变成常备断言——常备断言的对象是「返回报告」，不是崩溃。debug 与 release 两个构建边界的原始测量 MUST 分开记录：MUST NOT 用 release 对更深输入完成替代 debug 的 signal 证据，也 MUST NOT 用任一构建的完成深度代替界的依据。

#### Scenario: Counterexample fails on the pre-fix behaviour

- **WHEN** 在修正前的行为基线上运行该永久回归
- **THEN** 用例失败，失败现象是进程以栈溢出 signal 结束或没有可读报告；不得把「测试当时没跑」或「断言未写」当作这条证据（验收 A13、A14）

#### Scenario: Mutation restores the unbounded recursion

- **WHEN** 移除显式界（恢复无界递归）后重跑回归
- **THEN** 回归变红并复现 signal 现象；红必须来自该受控输入的进程层断言，不得用其他用例的红代替

#### Scenario: The post-fix stop is deterministic

- **WHEN** 同一受控 fixture 与默认预算重复运行修正后的实现
- **THEN** 停止的分类、位置与原因稳定可复核，不因线程栈大小、机器负载或进程内先后顺序改变；要求稳定的是界决定的停止，而不是修正前的 signal 时机

#### Scenario: Both entries answer

- **WHEN** 同一受控输入分别经库入口与 CLI 运行
- **THEN** 两侧都得到同源的非 Complete 报告与一致的停止分类；只检验库或只检验进程都不满足本要求（验收 A13、A14）

#### Scenario: The deep chain runs as a subprocess in both builds

- **WHEN** 深链输入经 CLI 在子进程内分别以 debug 与 release 构建运行
- **THEN** 两个构建 MUST 都返回报告：能呈现时是产物（CLI 的成功状态），不能呈现时是既有停止（CLI 的「执行未完整」状态）；MUST NOT 出现 signal 终止、exit 134、空 stdout 或 `fatal runtime error: stack overflow, aborting`。只检验其中一个构建 MUST NOT 被认为满足本要求（验收 A13、A14）

#### Scenario: Normal completion and a budget stop are both covered

- **WHEN** 同一深链输入在默认预算下运行一次，并在一个会让构建/发射中途停止的预算下再运行一次
- **THEN** 前者的报告 MUST 是产物或既有停止，后者的报告 MUST 是既有停止形态（`content = not_produced`、text 与段表为空）；两次运行都 MUST 正常退出、MUST NOT abort，后者的清理不遗留崩溃（验收 A13、A14）

#### Scenario: Mutation restores the chain's depth in the layer's representation

- **WHEN** 变异把片段表示退回左深 `Binary` 折叠（或任一使链长重新加深构造/发射/释放嵌套的实现）后重跑深链检查
- **THEN** debug 构建的子进程检查 MUST 变红，失败现象是 signal/abort 或 `stack overflow`（该变异恢复的就是修正前的形态）；MUST NOT 用其它用例的红代替；变异恢复后不遗留调试改动（验收 A13、A14）

#### Scenario: The check does not enlarge a thread stack

- **WHEN** 深链检查在门禁中运行
- **THEN** MUST 在默认线程栈上取得结论，MUST NOT 设置 `RUST_MIN_STACK`、把工作放到大栈线程或使用等价手段；「在放大栈下不 abort」MUST NOT 被当作本要求的证据（验收 A14）

#### Scenario: The pre-fix abort and the two build boundaries are one-time records

- **WHEN** change 的 verification 记录该输入修正前的失败
- **THEN** 记录 MUST 是命令、exit 状态、stderr 的 `has overflowed its stack`／`stack overflow, aborting` 行与空 stdout 的一次性事实，并分别记录 debug 的 signal 区间（1536/2048，1024 完成）与 release 的完成范围（2048–15000）；MUST NOT 把它写成常备断言，也 MUST NOT 用「进程仍然存活」或「stderr 没有溢出字样」代替修正后的报告断言（验收 A13、A14）

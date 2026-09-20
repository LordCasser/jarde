## ADDED Requirements

### Requirement: A signal is not an acceptable answer

恢复验收 SHALL 把「修正前以进程 signal 结束」的输入纳入永久回归，并在公开入口核对修正后得到的是报告。该回归 MUST 在独立进程内运行这个输入，因为 signal 是进程级事实，进程内的断言捕获不到它。验收证据 MUST 是报告的 `outcome`、`execution`、诊断与 CLI 退出状态；MUST NOT 以「stderr 没有 stack overflow」「进程存活」或「日志里没有 abort」单独算作通过，也 MUST NOT 通过扩大预算、在测试外重跑或降低输入来绕过。修正前该回归 MUST 失败，且失败现象是 signal/无报告，而不是断言缺失或编译错误。

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

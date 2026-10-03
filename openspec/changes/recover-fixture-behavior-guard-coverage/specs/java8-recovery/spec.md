## ADDED Requirements

### Requirement: 冻结行为 fixture 由 CI 断言守卫

系统 SHALL 为 `tests/fixtures/proved-java-structure/` 中每个已冻结的行为 fixture 提供至少一条 CI 默认套件内的呈现断言（关键文本锚），并在其有可运行基线时提供一条执行对照断言（原 class 输出为 golden；可重编者加重编运行逐行对照）。当前不可编译的形 SHALL 被断言为不可编译并记录其诊断，不得被当作通过。`run.sh` SHALL NOT 被算作守卫（其为手动复现工具）。新增冻结行为 fixture SHALL 同时新增引用它的 CI 测试。

#### Scenario: 构造期虚分派回归在 CI 可见

- **WHEN** 任何改动使 `anonymous-super-dispatch/AnonymousSuperDispatch$1` 的呈现把捕获写入移到 `super()` 之后
- **THEN** 默认套件中的呈现锚断言失败（该形正是本要求设立前静默通过 CI 的真实回归）

#### Scenario: 不可编译形不被当作通过

- **WHEN** fixture 的完整源集当前 `javac --release 8` 退出非 0（如 `anonymous-super-args`）
- **THEN** 测试断言其退出非 0 且诊断与记录一致，而非跳过或视为成功

#### Scenario: 守卫可判伪

- **WHEN** 以等价扰动临时破坏被锚定的呈现（自检）
- **THEN** 对应断言失败，证明锚具备判伪能力；自检产物不留在代码库

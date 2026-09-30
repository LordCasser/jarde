# Catches 参数槽复用巡查（2026-10-01）

[CF-17b 已知边界](../../java-syntax-2026-09-30/cf17-twrcatch-patrol/enclosing-17b/README.md)的展开取证（主线 `fdedd36f`）。固定转录 [fixture](fixture/)（S1，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），基线 JSON 与恢复文本在 [results](results/)。

## 形态与结果

`twrHelper`：`try (S1 r = new S1()) { touch(r); } catch (IllegalStateException e) { return tag(e); }`——javac 让 catch 参数**复用资源槽 0**（绑定 `astore_0` 覆写资源 store）。对照 `plainHelper`（普通 try/catch，参数槽无复用）。

| 场景 | 主线 Jarde |
| --- | --- |
| twrHelper（槽复用） | catch 子句头正确拼 `IllegalStateException local0`，但 handler 体内 `tag(e)` 实参被引："declared `java.lang.IllegalStateException` presents `S1` but the invocation requires `java.lang.IllegalStateException` … no safe reference conversion evidence"（BCI 40） |
| plainHelper（对照） | 完整恢复 `return tag(local0);` |

## 根因

`decide_types` 对槽 0 的类型决策被资源 store 赢得（`S1`——单槽两定义取一处）；handler 体内对该槽的读取在实参呈现时沿用**槽决策**，而该读取的 SSA 值定义实际是 catch 绑定 store（`Definition::Caught`，行类型 `IllegalStateException`）。即：呈现类型按槽归属而非按值归属——参数槽复用（值流正确、槽身被两个不同类型所有者先后使用）暴露了该归属规则。正常非复用场景（对照）槽只有一个类型所有者，规则差异不可见。

## 处置方向

读取的呈现类型跟随**值定义**：handler 体内读到的 SSA 值若定义为 catch 绑定 store（Caught），呈现该绑定值的行类型（子句头已正确拼写的类型），不再回退槽决策。这是呈现归属修正而非新机制——`Definition::Caught` 在 build.rs 多处已有类型/拼写事实（行 22953 一带已把 caught 拼为子句类型），缺的是实参呈现路径对复用槽读取取值定义类型。

原 class 为行为基准；JADX 参照不作为语义正例。

## 处置结果（2026-10-01，`recover-caught-value-argument-typing`）

方向已实施：实参/读取呈现对 handler 入口 store 的值按入口引用自身的类型拼写（单行子句即行
类型），其余读取保持槽决策。诊断与落点见 [typing-diagnosis.md](typing-diagnosis.md)；变体前后
输出见 [variants/](variants/)（S1V.java / S1V.before.txt / S1V.after.txt）；恢复输出与三方对照
见 [results/S1-typing.after.java](results/S1-typing.after.java)、
[results/typing-threeway.md](results/typing-threeway.md)。回归测试钉在
`tests/p3_caught_value_argument_typing.rs`（fixture 同字节）。

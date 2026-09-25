## 1. 冻结现状与边界

- [x] 1.1 重放 `shape-matrix/replay.py`，核对 Java 8 `-g`/`-g:none` 原始正常/null/效果轨迹、四个 caller-only 原始重编、JADX 独立重编诊断及 SHA 清单，并在分析文档记录实际版本与结果。
- [x] 1.2 以当前 Jarde 对同一冻结 jar 输出四个调用方和 `Outer/A/Plain/Generic` 的 class-source JSON，记录每个拒绝所在层与物理 Signature/descriptor；验收不把单类展示当成完整类族输出。

## 2. 类关系与源类型路径

- [x] 2.1 在现有成员构造证明中受预算核验泛型外层 `Outer.A<T>` 的已选定义、双向静态成员关系、唯一类 Signature 及擦除，且仅对完整证明放开非静态成员目标；用签名/关系错形、缺失与歧义定义控制确认拒绝或停止。
- [x] 2.2 基于已选类族关系为本次调用方建立受限源类型路径，支持原始 `Outer.A`、`Outer.A<String>` 和 `Outer.A<String>.Generic<Integer>` 的每段名称与类型实参数量/擦除核验；用合法 `$` 标识符、错成员边与错实参数量控制确认不做字符串猜测。

## 3. 调用点与方法头

- [x] 3.1 复用现有 `new@1` 外层 SSA 身份、早空值检查、普通实参顺序与泛型成员构造尾部对齐，让矩阵四个调用点输出已证 `outer.new Plain(...)` 或 `outer.new Generic<>(...)`；定向测试核对报告仍含物理外层首参和各 BCI，错身份/迟检查仍拒绝。
- [x] 3.2 仅在同一运行已证成员创建返回 body 上投影调用方方法头的嵌套类型及同一已选 `A` 的 `mark(...)` 静态限定符：原始参数、`A<String>` 参数及 `A<String>.Generic<Integer>` 返回；由 Java 8 caller-only `javac` 验证，其他普通泛型方法回归不放宽。

## 4. 独立验收

- [x] 4.1 对 `-g` 与 `-g:none` 的冻结原始目标 jar 各自重编四个 Jarde caller，并运行替换后的 Runner；与原 class 对比完整 stdout、null 异常先后及实参效果次数，再记录与 JADX 四个失败 caller 的差异。
- [x] 4.2 验证完整类入口必要/全部证据的正文一致性、BCI 范围请求的既有明确停止、预算/取消停止、定向 Rust 回归、`cargo fmt` 与 `openspec validate --strict`；清理私有 Cargo target，并由 Root 写 `verification-root.md` 后勾选实际完成项。
